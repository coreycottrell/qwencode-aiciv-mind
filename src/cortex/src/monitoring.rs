//! Monitoring module — bridges cortex-monitoring crate into Cortex main binary.

pub use cortex_monitoring::MetricsCollector;
pub use cortex_monitoring::AnomalyDetector;
pub use cortex_monitoring::export::MetricsExporter;
pub use cortex_monitoring::metrics::*;

use std::path::Path;

/// Initialize monitoring for a mind.
pub fn init_monitoring(
    mind_id: &str,
    project_root: &Path,
) -> (MetricsCollector, AnomalyDetector, MetricsExporter) {
    let metrics_dir = project_root.join("data").join("metrics").join(mind_id);
    let alerts_dir = project_root.join("data").join("alerts");

    std::fs::create_dir_all(&metrics_dir).ok();
    std::fs::create_dir_all(&alerts_dir).ok();

    let collector = MetricsCollector::new(mind_id, &metrics_dir);
    let detector = AnomalyDetector::new(mind_id, &alerts_dir);
    let exporter = MetricsExporter::new(&metrics_dir);

    (collector, detector, exporter)
}
