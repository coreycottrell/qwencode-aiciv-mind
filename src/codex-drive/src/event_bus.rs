//! # EventBus — Dual-Channel Event Router
//!
//! External events (Hub, Telegram, Human, sub-mind results) have capacity-64 channel.
//! Drive events (idle detection, task discovery) have capacity-1 channel.
//!
//! `tokio::select! { biased }` ensures external events ALWAYS take priority.
//! Drive events only surface when there's nothing external to process.
//!
//! This implements InputMux (Principle A2) at the channel level:
//! "Using channel capacity as backpressure instead of runtime priority logic
//!  is the kind of design that never breaks because there's nothing to break."
//! — Corey, 2026-04-04

use std::sync::atomic::{AtomicBool, Ordering};

use codex_types::{ExternalEvent, DriveEvent, MindEvent};
use tokio::sync::mpsc;

/// Capacity of the external event channel.
const EXTERNAL_CAPACITY: usize = 64;

/// Capacity of the drive event channel. Capacity-1 = natural backpressure.
/// If the drive channel is full, DriveLoop's try_send() silently drops.
/// This means at most ONE drive event is pending at any time.
const DRIVE_CAPACITY: usize = 1;

/// Producer handle for external events.
#[derive(Clone)]
pub struct ExternalSender {
    tx: mpsc::Sender<ExternalEvent>,
    has_pending: AtomicBoolRef,
}

/// Producer handle for drive events.
#[derive(Clone)]
pub struct DriveSender {
    tx: mpsc::Sender<DriveEvent>,
}

impl DriveSender {
    /// Create a DriveSender from a raw mpsc::Sender.
    /// Primarily for testing — production code uses `event_bus::create()`.
    pub fn from_sender(tx: mpsc::Sender<DriveEvent>) -> Self {
        Self { tx }
    }
}

/// Shared atomic flag — true when external events are pending.
/// DriveLoop checks this to yield to external work.
#[derive(Clone)]
struct AtomicBoolRef(std::sync::Arc<AtomicBool>);

impl AtomicBoolRef {
    fn new() -> Self {
        Self(std::sync::Arc::new(AtomicBool::new(false)))
    }

    fn set(&self, val: bool) {
        self.0.store(val, Ordering::Release);
    }

    fn get(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// The EventBus consumer. Receives from both channels with biased priority.
pub struct EventBus {
    external_rx: mpsc::Receiver<ExternalEvent>,
    drive_rx: mpsc::Receiver<DriveEvent>,
    external_pending: AtomicBoolRef,
}

/// Create a new EventBus with its producer handles.
///
/// Returns (EventBus, ExternalSender, DriveSender).
pub fn create() -> (EventBus, ExternalSender, DriveSender) {
    let (ext_tx, ext_rx) = mpsc::channel(EXTERNAL_CAPACITY);
    let (drv_tx, drv_rx) = mpsc::channel(DRIVE_CAPACITY);
    let flag = AtomicBoolRef::new();

    let bus = EventBus {
        external_rx: ext_rx,
        drive_rx: drv_rx,
        external_pending: flag.clone(),
    };

    let ext_sender = ExternalSender {
        tx: ext_tx,
        has_pending: flag,
    };

    let drv_sender = DriveSender {
        tx: drv_tx,
    };

    (bus, ext_sender, drv_sender)
}

impl ExternalSender {
    /// Send an external event. Returns error if the bus is closed.
    pub async fn send(&self, event: ExternalEvent) -> Result<(), mpsc::error::SendError<ExternalEvent>> {
        self.has_pending.set(true);
        self.tx.send(event).await
    }
}

impl DriveSender {
    /// Try to send a drive event. Returns error if the channel is full or closed.
    /// This is intentional — capacity-1 means at most one drive event is pending.
    /// If the channel is full, the event is dropped (natural backpressure).
    pub fn try_send(&self, event: DriveEvent) -> Result<(), mpsc::error::TrySendError<DriveEvent>> {
        self.tx.try_send(event)
    }

    /// Check if the drive channel has capacity (for DriveLoop cooldown logic).
    pub fn has_capacity(&self) -> bool {
        self.tx.capacity() > 0
    }
}

impl EventBus {
    /// Receive the next event with biased priority.
    ///
    /// External events ALWAYS take priority over drive events.
    /// This ensures real work preempts self-prompted activity.
    ///
    /// Returns None when all senders are dropped (shutdown).
    pub async fn recv(&mut self) -> Option<MindEvent> {
        tokio::select! {
            biased;

            // External events always checked first
            Some(event) = self.external_rx.recv() => {
                // Check if more external events are pending
                if self.external_rx.is_empty() {
                    self.external_pending.set(false);
                }
                Some(MindEvent::External(event))
            }

            // Drive events only if no external events
            Some(event) = self.drive_rx.recv() => {
                Some(MindEvent::Drive(event))
            }

            // Both channels closed
            else => None,
        }
    }

    /// Check if external events are pending (for DriveLoop yield logic).
    pub fn external_pending(&self) -> bool {
        self.external_pending.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use codex_types::{EventSource, EventPriority};

    fn make_external(content: &str) -> ExternalEvent {
        ExternalEvent {
            source: EventSource::Human { name: "test".into() },
            content: content.into(),
            priority: EventPriority::Normal,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        }
    }

    #[tokio::test]
    async fn external_takes_priority() {
        let (mut bus, ext_tx, drv_tx) = create();

        // Send both events before receiving
        ext_tx.send(make_external("external")).await.unwrap();
        drv_tx.try_send(DriveEvent::IdleSuggestion {
            suggestion: "do something".into(),
        }).unwrap();

        // First receive should be external (biased)
        let first = bus.recv().await.unwrap();
        assert!(matches!(first, MindEvent::External(_)));

        // Second should be drive
        let second = bus.recv().await.unwrap();
        assert!(matches!(second, MindEvent::Drive(_)));
    }

    #[tokio::test]
    async fn drive_backpressure() {
        let (_bus, _ext_tx, drv_tx) = create();

        // First send should succeed (capacity = 1)
        let result1 = drv_tx.try_send(DriveEvent::IdleSuggestion {
            suggestion: "first".into(),
        });
        assert!(result1.is_ok());

        // Second send should fail (channel full)
        let result2 = drv_tx.try_send(DriveEvent::IdleSuggestion {
            suggestion: "second".into(),
        });
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn shutdown_returns_none() {
        let (mut bus, ext_tx, drv_tx) = create();

        // Drop all senders
        drop(ext_tx);
        drop(drv_tx);

        // Should return None (shutdown)
        let result = bus.recv().await;
        assert!(result.is_none());
    }
}
