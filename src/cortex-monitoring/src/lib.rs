//! # cortex-monitoring — Observability for the Fractal Mind
//!
//! Real-time metrics collection, performance tracking, anomaly detection,
//! and export for every Cortex mind. This is the observability substrate
//! that makes the self-improving loop data-driven.
//!
//! ## What it tracks
//!
//! - **ThinkLoop metrics**: iterations, tool calls, latency, stall kills
//! - **Tool metrics**: call counts, success rates, latency per tool
//! - **Delegation metrics**: spawn counts, completion times, success rates
//! - **Memory metrics**: searches, writes, depth scores, cache hits
//! - **Challenger metrics**: warnings fired, severity distribution
//! - **Fitness metrics**: composite scores per role over time
//! - **Model metrics**: which model was used, latency, token usage
//!
//! ## Export
//!
//! Metrics are written to:
//! - **JSONL files** in `data/metrics/{mind_id}/` for persistence
//! - **In-memory ring buffer** for real-time access
//! - **Anomaly alerts** written to `data/alerts/` when thresholds breached

pub mod metrics;
pub mod collector;
pub mod anomaly;
pub mod export;

pub use metrics::*;
pub use collector::MetricsCollector;
pub use anomaly::AnomalyDetector;
pub use export::MetricsExporter;
