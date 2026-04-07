use std::f64::consts::PI;

/// Equivalent to `np.fft.rfftfreq(n, d)`.
/// Returns frequencies for an rfft of length n with sample spacing d.
pub fn rfftfreq(n: usize, d: f64) -> Vec<f64> {
    let m = n / 2 + 1;
    let factor = 1.0 / (n as f64 * d);
    (0..m).map(|i| i as f64 * factor).collect()
}

/// Index of first frequency >= f_lo in a monotonically increasing freq array.
pub fn highpass_start(freqs: &[f64], f_lo: f64) -> usize {
    freqs.partition_point(|&f| f < f_lo)
}

/// (start, end) indices for freqs where f_lo <= freq <= f_hi.
pub fn bandpass_range(freqs: &[f64], f_lo: f64, f_hi: f64) -> (usize, usize) {
    let start = freqs.partition_point(|&f| f < f_lo);
    let end = freqs.partition_point(|&f| f <= f_hi);
    (start, end)
}

/// Max of f64 slice (f64 lacks Ord).
pub fn f64_max(data: &[f64]) -> f64 {
    data.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

/// Min of f64 slice (f64 lacks Ord).
pub fn f64_min(data: &[f64]) -> f64 {
    data.iter().copied().fold(f64::INFINITY, f64::min)
}

pub const TWO_PI: f64 = 2.0 * PI;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfftfreq_even() {
        // np.fft.rfftfreq(8, 1/100) => [0, 12.5, 25, 37.5, 50]
        let freqs = rfftfreq(8, 1.0 / 100.0);
        assert_eq!(freqs.len(), 5);
        assert!((freqs[0] - 0.0).abs() < 1e-12);
        assert!((freqs[1] - 12.5).abs() < 1e-12);
        assert!((freqs[4] - 50.0).abs() < 1e-12);
    }

    #[test]
    fn test_rfftfreq_odd() {
        // np.fft.rfftfreq(7, 1/100) => [0, 14.285..., 28.571..., 42.857...]
        let freqs = rfftfreq(7, 1.0 / 100.0);
        assert_eq!(freqs.len(), 4);
        assert!((freqs[1] - 100.0 / 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_highpass_start() {
        let freqs = vec![0.0, 5.0, 10.0, 15.0, 20.0];
        assert_eq!(highpass_start(&freqs, 10.0), 2);
        assert_eq!(highpass_start(&freqs, 7.0), 2);
        assert_eq!(highpass_start(&freqs, 0.0), 0);
    }

    #[test]
    fn test_bandpass_range() {
        let freqs = vec![0.0, 5.0, 10.0, 15.0, 20.0, 25.0];
        let (s, e) = bandpass_range(&freqs, 10.0, 20.0);
        assert_eq!(s, 2);
        assert_eq!(e, 5); // indices 2,3,4 => freqs 10,15,20
    }

    #[test]
    fn test_f64_max() {
        assert!((f64_max(&[1.0, 3.0, 2.0]) - 3.0).abs() < 1e-15);
        assert!((f64_max(&[-5.0, -1.0, -3.0]) - -1.0).abs() < 1e-15);
    }

    #[test]
    fn test_f64_min() {
        assert!((f64_min(&[1.0, 3.0, 2.0]) - 1.0).abs() < 1e-15);
        assert!((f64_min(&[-5.0, -1.0, -3.0]) - -5.0).abs() < 1e-15);
    }

    #[test]
    fn test_bandpass_range_exact_boundaries() {
        let freqs = vec![0.0, 10.0, 20.0, 30.0];
        let (s, e) = bandpass_range(&freqs, 10.0, 20.0);
        assert_eq!(s, 1);
        assert_eq!(e, 3);
    }
}
