use num_complex::Complex64;
use realfft::RealFftPlanner;

use crate::fft_utils::{bandpass_range, f64_max, f64_min, highpass_start, rfftfreq, TWO_PI};

// ─── Internal FFT helpers ────────────────────────────────────────────

/// Forward real FFT, equivalent to `np.fft.rfft(data)`.
/// Returns complex spectrum of length N/2+1.
fn rfft(data: &[f64]) -> Vec<Complex64> {
    let n = data.len();
    let mut planner = RealFftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(n);

    let mut input = data.to_vec();
    let mut spectrum = fft.make_output_vec();
    fft.process(&mut input, &mut spectrum).unwrap();
    spectrum
}

/// Inverse real FFT, equivalent to `np.fft.irfft(spectrum)`.
/// The caller must provide `n` = original signal length.
/// Output is normalized by dividing by n (matching numpy convention).
fn irfft(spectrum: &[Complex64], n: usize) -> Vec<f64> {
    let mut planner = RealFftPlanner::<f64>::new();
    let ifft = planner.plan_fft_inverse(n);

    let mut spec = spectrum.to_vec();
    let mut output = ifft.make_output_vec();
    ifft.process(&mut spec, &mut output).unwrap();

    // realfft inverse is unnormalized; divide by n to match numpy
    let inv_n = 1.0 / n as f64;
    for v in output.iter_mut() {
        *v *= inv_n;
    }
    output
}

// ─── Public API ──────────────────────────────────────────────────────

/// Port of `Vibration.acceleration_rms()`.
///
/// Returns `(acc_rms, vel_rms, dis_pkpk)` in (G, m/s, m).
/// Uses inverse FFT for peak-peak displacement. g = 9.8.
pub fn acceleration_rms(data: &[f64], sampling_rate: f64, f_lo: f64, f_hi: f64) -> (f64, f64, f64) {
    let g = 9.8;
    let n = data.len();

    let fft = rfft(data);
    let mut freq_ticks = rfftfreq(n, 1.0 / sampling_rate);

    // Filtered copy for displacement iFFT
    let fft_filtered: Vec<Complex64> = fft
        .iter()
        .zip(freq_ticks.iter())
        .map(|(&c, &f)| {
            if f < f_lo || f > f_hi {
                Complex64::new(0.0, 0.0)
            } else {
                c
            }
        })
        .collect();

    // Avoid division by zero — mutate freq_ticks[0] = 1.0 (matches Python exactly)
    freq_ticks[0] = 1.0;

    // displacement spectrum: FFT_filtered / (2*pi*f)^2
    let mut tmp: Vec<Complex64> = fft_filtered
        .iter()
        .zip(freq_ticks.iter())
        .map(|(&c, &f)| c / (TWO_PI * f).powi(2))
        .collect();
    tmp[0] = Complex64::new(0.0, 0.0); // DC = 0

    // irfft and negate
    let dis_time: Vec<f64> = irfft(&tmp, n).iter().map(|&v| -v).collect();
    let dis_pkpk = g * (f64_max(&dis_time) - f64_min(&dis_time));

    // Normalize FFT magnitude by spectrum length (len(FFT) in Python = N/2+1)
    let spec_len = fft.len() as f64;
    let fft_abs: Vec<f64> = fft.iter().map(|c| c.norm() / spec_len).collect();

    // acc_rms: high-pass only
    let hp_start = highpass_start(&freq_ticks, f_lo);
    let fft_high_pass = &fft_abs[hp_start..];
    let acc_rms = (fft_high_pass.iter().map(|&v| v * v).sum::<f64>()).sqrt() * 2.0_f64.powf(-0.5);

    // vel_rms: bandpass
    let (bp_start, bp_end) = bandpass_range(&freq_ticks, f_lo, f_hi);
    let fft_bp = &fft_abs[bp_start..bp_end];
    let freq_bp = &freq_ticks[bp_start..bp_end];

    let vel_sum: f64 = fft_bp
        .iter()
        .zip(freq_bp.iter())
        .map(|(&a, &f)| a * a / (f * TWO_PI).powi(2))
        .sum();
    let vel_rms = g * vel_sum.sqrt() * 2.0_f64.powf(-0.5);

    (acc_rms, vel_rms, dis_pkpk)
}

/// Port of `Vibration.acceleration_rms2()`.
///
/// Returns `(acc_rms, vel_rms, dis_pkpk)` via PSD approach. g = 9.81.
pub fn acceleration_rms2(
    data: &[f64],
    sampling_rate: f64,
    f_lo: f64,
    f_hi: f64,
) -> (f64, f64, f64) {
    let n = data.len();
    let fft = rfft(data);
    let freq_ticks = rfftfreq(n, 1.0 / sampling_rate);
    let spec_len = fft.len() as f64;

    // FFT_abs = |fft| / len(fft)
    let fft_abs: Vec<f64> = fft.iter().map(|c| c.norm() / spec_len).collect();

    // PSD = fft_abs^2 / 2
    let psd: Vec<f64> = fft_abs.iter().map(|&a| a * a / 2.0).collect();

    // acc_rms from high-passed PSD (in G units)
    let hp_start = highpass_start(&freq_ticks, f_lo);
    let acc_rms = psd[hp_start..].iter().sum::<f64>().sqrt();

    // Convert to SI: multiply PSD by g^2
    let g = 9.81;
    let g2 = g * g;
    let psd_si: Vec<f64> = psd.iter().map(|&p| p * g2).collect();

    // Bandpass slice
    let (bp_start, bp_end) = bandpass_range(&freq_ticks, f_lo, f_hi);
    let psd_bp = &psd_si[bp_start..bp_end];
    let freq_bp = &freq_ticks[bp_start..bp_end];

    let vel_rms = psd_bp
        .iter()
        .zip(freq_bp.iter())
        .map(|(&p, &f)| p / (f * TWO_PI).powi(2))
        .sum::<f64>()
        .sqrt();

    let dis_rms = psd_bp
        .iter()
        .zip(freq_bp.iter())
        .map(|(&p, &f)| p / (f * TWO_PI).powi(4))
        .sum::<f64>()
        .sqrt();

    let dis_pkpk = dis_rms * 2.0 * 2.0_f64.sqrt();

    (acc_rms, vel_rms, dis_pkpk)
}

/// Port of `Vibration.get_fft_abs_data()`.
///
/// Returns `(fft_abs, freq_ticks, angle_data)`.
/// Normalization: divide by `fft_abs.len()` (= shape[0] in Python).
pub fn get_fft_abs_data(data: &[f64], sampling_rate: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = data.len();
    let fft = rfft(data);
    let spec_len = fft.len() as f64;

    let fft_abs: Vec<f64> = fft.iter().map(|c| c.norm() / spec_len).collect();
    let angle_data: Vec<f64> = fft.iter().map(|c| c.arg()).collect();
    let freq_ticks = rfftfreq(n, 1.0 / sampling_rate);

    (fft_abs, freq_ticks, angle_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn make_sine(freq: f64, n: usize, sr: f64) -> Vec<f64> {
        (0..n)
            .map(|i| (2.0 * PI * freq * i as f64 / sr).sin())
            .collect()
    }

    #[test]
    fn test_rfft_irfft_roundtrip() {
        let data: Vec<f64> = (0..128).map(|i| (i as f64 * 0.1).sin()).collect();
        let spectrum = rfft(&data);
        let recovered = irfft(&spectrum, data.len());
        for (a, b) in data.iter().zip(recovered.iter()) {
            assert!((a - b).abs() < 1e-10, "roundtrip failed: {} vs {}", a, b);
        }
    }

    #[test]
    fn test_rfft_length() {
        let data = vec![0.0; 100];
        let spectrum = rfft(&data);
        assert_eq!(spectrum.len(), 51); // N/2 + 1
    }

    #[test]
    fn test_get_fft_abs_data_lengths() {
        let data = make_sine(50.0, 256, 1000.0);
        let (abs_data, freqs, angles) = get_fft_abs_data(&data, 1000.0);
        assert_eq!(abs_data.len(), 129);
        assert_eq!(freqs.len(), 129);
        assert_eq!(angles.len(), 129);
    }

    #[test]
    fn test_get_fft_abs_data_peak_at_frequency() {
        let data = make_sine(50.0, 1000, 1000.0);
        let (abs_data, freqs, _) = get_fft_abs_data(&data, 1000.0);
        // Find index of 50 Hz
        let idx = freqs.iter().position(|&f| (f - 50.0).abs() < 0.01).unwrap();
        // Peak should be at 50 Hz
        let max_idx = abs_data
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;
        assert_eq!(max_idx, idx);
    }

    #[test]
    fn test_acceleration_rms_returns_positive() {
        let data = make_sine(100.0, 2048, 4096.0);
        let (acc, vel, dis) = acceleration_rms(&data, 4096.0, 10.0, 1000.0);
        assert!(acc > 0.0, "acc_rms should be positive");
        assert!(vel > 0.0, "vel_rms should be positive");
        assert!(dis > 0.0, "dis_pkpk should be positive");
    }

    #[test]
    fn test_acceleration_rms2_returns_positive() {
        let data = make_sine(100.0, 2048, 4096.0);
        let (acc, vel, dis) = acceleration_rms2(&data, 4096.0, 10.0, 1000.0);
        assert!(acc > 0.0, "acc_rms should be positive");
        assert!(vel > 0.0, "vel_rms should be positive");
        assert!(dis > 0.0, "dis_pkpk should be positive");
    }

    #[test]
    fn test_acceleration_rms_zero_outside_band() {
        // Signal entirely below lower bound -> near-zero
        let data = make_sine(5.0, 4096, 4096.0);
        let (acc, _, _) = acceleration_rms(&data, 4096.0, 10.0, 1000.0);
        assert!(acc < 0.01, "signal below band should give small acc_rms");
    }

    #[test]
    fn test_acceleration_rms2_zero_outside_band() {
        let data = make_sine(5.0, 4096, 4096.0);
        let (acc, _, _) = acceleration_rms2(&data, 4096.0, 10.0, 1000.0);
        assert!(acc < 0.01, "signal below band should give small acc_rms");
    }
}
