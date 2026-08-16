use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const BUCKET_UPPER_SECONDS: [f64; 11] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

static BUCKET_COUNTS: [AtomicU64; 11] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];
static DURATION_SUM_MICROS: AtomicU64 = AtomicU64::new(0);
static DURATION_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn record_duration(duration: Duration) {
    let seconds = duration.as_secs_f64();
    DURATION_COUNT.fetch_add(1, Ordering::Relaxed);
    DURATION_SUM_MICROS.fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    for (index, upper_bound) in BUCKET_UPPER_SECONDS.iter().enumerate() {
        if seconds <= *upper_bound {
            BUCKET_COUNTS[index].fetch_add(1, Ordering::Relaxed);
        }
    }
}

pub fn render_prometheus_histogram(
    metric_name: &str,
    service: &str,
    environment: &str,
    deployment_profile: &str,
) -> String {
    let mut output = format!(
        "# HELP {metric_name} HTTP request latency in seconds.\n\
         # TYPE {metric_name} histogram\n"
    );
    let mut cumulative = 0u64;
    for (index, upper_bound) in BUCKET_UPPER_SECONDS.iter().enumerate() {
        cumulative += BUCKET_COUNTS[index].load(Ordering::Relaxed);
        output.push_str(&format!(
            "{metric_name}_bucket{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\",le=\"{upper_bound}\"}} {cumulative}\n"
        ));
    }
    let total = DURATION_COUNT.load(Ordering::Relaxed);
    output.push_str(&format!(
        "{metric_name}_bucket{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\",le=\"+Inf\"}} {total}\n"
    ));
    let sum_seconds = DURATION_SUM_MICROS.load(Ordering::Relaxed) as f64 / 1_000_000.0;
    output.push_str(&format!(
        "{metric_name}_sum{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {sum_seconds}\n"
    ));
    output.push_str(&format!(
        "{metric_name}_count{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {total}\n"
    ));
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_duration_into_histogram_buckets() {
        record_duration(Duration::from_millis(50));
        let rendered = render_prometheus_histogram(
            "drive_http_request_duration_seconds",
            "test-service",
            "test",
            "local",
        );
        assert!(rendered.contains("drive_http_request_duration_seconds_bucket"));
        assert!(rendered.contains("le=\"0.1\""));
        assert!(rendered.contains("drive_http_request_duration_seconds_count"));
    }
}
