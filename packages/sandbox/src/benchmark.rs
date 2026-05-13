#[cfg(target_os = "linux")]
use std::time::Instant;
#[cfg(target_os = "linux")]
use crate::linux::seccomp;

#[cfg(target_os = "linux")]
pub fn measure_overhead() -> (f64, f64, f64) {
    let iterations = 100;
    let mut total_ns = 0u128;
    let mut min_ns = u128::MAX;
    let mut max_ns = 0u128;

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = seccomp::build_seccomp_filter();
        let duration = start.elapsed();
        let ns = duration.as_nanos() as u128;

        total_ns += ns;
        if ns < min_ns { min_ns = ns; }
        if ns > max_ns { max_ns = ns; }
    }

    let avg_ns = total_ns / iterations;
    (avg_ns as f64 / 1_000_000.0,
     min_ns as f64 / 1_000_000.0,
     max_ns as f64 / 1_000_000.0)
}

#[cfg(not(target_os = "linux"))]
pub fn measure_overhead() -> (f64, f64, f64) {
    (0.0, 0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overhead_under_200ms() {
        let (avg, min, max) = measure_overhead();
        println!("Sandbox overhead: avg={:.2}ms, min={:.2}ms, max={:.2}ms", avg, min, max);
        assert!(avg < 200.0, "Average overhead {:.2}ms exceeds 200ms target", avg);
    }
}
