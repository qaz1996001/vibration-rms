use ndarray::Array1;
use ndarray_stats::SummaryStatisticsExt;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Time-domain feature enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TdFeature {
    Max,
    Min,
    Mean,
    Std,
    SqrtAmplitude,
    Rms,
    PeakPeak,
    Skewness,
    Kurtosis,
    Crest,
    Clearance,
    Shape,
    Impulse,
}

impl TdFeature {
    pub const ALL: &[Self] = &[
        Self::Max,
        Self::Min,
        Self::Mean,
        Self::Std,
        Self::SqrtAmplitude,
        Self::Rms,
        Self::PeakPeak,
        Self::Skewness,
        Self::Kurtosis,
        Self::Crest,
        Self::Clearance,
        Self::Shape,
        Self::Impulse,
    ];

    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "max" | "t_max" => Some(Self::Max),
            "min" | "t_min" => Some(Self::Min),
            "mean" | "t1" => Some(Self::Mean),
            "std" | "t2" => Some(Self::Std),
            "sqrt_amplitude" | "t3" => Some(Self::SqrtAmplitude),
            "rms" | "t4" => Some(Self::Rms),
            "peak_peak" | "t5" => Some(Self::PeakPeak),
            "skewness" | "t6" => Some(Self::Skewness),
            "kurtosis" | "t7" => Some(Self::Kurtosis),
            "crest" | "t8" => Some(Self::Crest),
            "clearance" | "t9" => Some(Self::Clearance),
            "shape" | "t10" => Some(Self::Shape),
            "impulse" | "t11" => Some(Self::Impulse),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Frequency-domain feature enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FdFeature {
    Mean,
    Variance,
    Skewness,
    Kurtosis,
    Centroid,
    RmsSpread,
    RmsFreq,
    FourthMoment,
    Regularity,
    SpreadRatio,
    WeightedSkew,
    WeightedKurt,
}

impl FdFeature {
    pub const ALL: &[Self] = &[
        Self::Mean,
        Self::Variance,
        Self::Skewness,
        Self::Kurtosis,
        Self::Centroid,
        Self::RmsSpread,
        Self::RmsFreq,
        Self::FourthMoment,
        Self::Regularity,
        Self::SpreadRatio,
        Self::WeightedSkew,
        Self::WeightedKurt,
    ];

    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "mean" | "f12" => Some(Self::Mean),
            "variance" | "f13" => Some(Self::Variance),
            "skewness" | "f14" => Some(Self::Skewness),
            "kurtosis" | "f15" => Some(Self::Kurtosis),
            "centroid" | "f16" => Some(Self::Centroid),
            "rms_spread" | "f17" => Some(Self::RmsSpread),
            "rms_freq" | "f18" => Some(Self::RmsFreq),
            "fourth_moment" | "f19" => Some(Self::FourthMoment),
            "regularity" | "f20" => Some(Self::Regularity),
            "spread_ratio" | "f21" => Some(Self::SpreadRatio),
            "weighted_skew" | "f22" => Some(Self::WeightedSkew),
            "weighted_kurt" | "f23" => Some(Self::WeightedKurt),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Selective time-domain feature computation
// ---------------------------------------------------------------------------

pub fn time_domain_features_select(signal: &[f64], features: &[TdFeature]) -> Vec<f64> {
    let requested: HashSet<TdFeature> = features.iter().copied().collect();
    let n = signal.len() as f64;

    // Determine which intermediates are needed
    let need_max = requested.contains(&TdFeature::Max) || requested.contains(&TdFeature::PeakPeak);
    let need_min = requested.contains(&TdFeature::Min) || requested.contains(&TdFeature::PeakPeak);
    let need_mean = requested.contains(&TdFeature::Mean) || requested.contains(&TdFeature::Std);
    let need_rms = requested.contains(&TdFeature::Rms)
        || requested.contains(&TdFeature::Crest)
        || requested.contains(&TdFeature::Shape);
    let need_sqrt_amp = requested.contains(&TdFeature::SqrtAmplitude)
        || requested.contains(&TdFeature::Clearance);
    let need_abs_max = requested.contains(&TdFeature::Crest)
        || requested.contains(&TdFeature::Clearance)
        || requested.contains(&TdFeature::Impulse);
    let need_abs_sum = requested.contains(&TdFeature::Shape)
        || requested.contains(&TdFeature::Impulse);

    // Single pass for basic intermediates
    let mut v_max = f64::NEG_INFINITY;
    let mut v_min = f64::INFINITY;
    let mut sum = 0.0_f64;
    let mut sum_sq = 0.0_f64;
    let mut abs_max = 0.0_f64;
    let mut abs_sum = 0.0_f64;
    let mut sqrt_abs_sum = 0.0_f64;

    for &x in signal {
        if need_max && x > v_max {
            v_max = x;
        }
        if need_min && x < v_min {
            v_min = x;
        }
        if need_mean || need_rms {
            sum += x;
            sum_sq += x * x;
        }
        if need_abs_max || need_abs_sum {
            let ax = x.abs();
            if need_abs_max && ax > abs_max {
                abs_max = ax;
            }
            if need_abs_sum {
                abs_sum += ax;
            }
        }
        if need_sqrt_amp {
            sqrt_abs_sum += x.abs().sqrt();
        }
    }

    let mean = if need_mean { sum / n } else { 0.0 };
    let rms = if need_rms { (sum_sq / n).sqrt() } else { 0.0 };
    let sqrt_amp = if need_sqrt_amp {
        (sqrt_abs_sum / n).powi(2)
    } else {
        0.0
    };

    // Conditional: std
    let std_val = if requested.contains(&TdFeature::Std) {
        let var = signal.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        var.sqrt()
    } else {
        0.0
    };

    // Conditional: skewness & kurtosis (expensive, only when needed)
    let skew = if requested.contains(&TdFeature::Skewness) {
        let arr = Array1::from_vec(signal.to_vec());
        arr.skewness().unwrap()
    } else {
        0.0
    };

    let kurt = if requested.contains(&TdFeature::Kurtosis) {
        let arr = Array1::from_vec(signal.to_vec());
        arr.kurtosis().unwrap() - 3.0
    } else {
        0.0
    };

    // Map requested features to values in request order
    features
        .iter()
        .map(|feat| match feat {
            TdFeature::Max => v_max,
            TdFeature::Min => v_min,
            TdFeature::Mean => mean,
            TdFeature::Std => std_val,
            TdFeature::SqrtAmplitude => sqrt_amp,
            TdFeature::Rms => rms,
            TdFeature::PeakPeak => 0.5 * (v_max - v_min),
            TdFeature::Skewness => skew,
            TdFeature::Kurtosis => kurt,
            TdFeature::Crest => abs_max / rms,
            TdFeature::Clearance => abs_max / sqrt_amp,
            TdFeature::Shape => (n * rms) / abs_sum,
            TdFeature::Impulse => abs_max / (abs_sum / n),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Selective frequency-domain feature computation
// ---------------------------------------------------------------------------

pub fn freq_domain_features_select(pl: &[f64], f: &[f64], features: &[FdFeature]) -> Vec<f64> {
    let requested: HashSet<FdFeature> = features.iter().copied().collect();
    let k = pl.len() as f64;
    let x = f;
    let y = pl;

    // Determine which intermediates are needed
    let need_f12 = requested.contains(&FdFeature::Mean)
        || requested.contains(&FdFeature::Variance)
        || requested.contains(&FdFeature::Skewness)
        || requested.contains(&FdFeature::Kurtosis);

    let need_f13 = requested.contains(&FdFeature::Variance)
        || requested.contains(&FdFeature::Skewness)
        || requested.contains(&FdFeature::Kurtosis);

    let need_sum_y = requested.contains(&FdFeature::Centroid)
        || requested.contains(&FdFeature::RmsFreq)
        || requested.contains(&FdFeature::Regularity);

    let need_sum_xy = requested.contains(&FdFeature::Centroid);

    let need_sum_x2y = requested.contains(&FdFeature::RmsFreq)
        || requested.contains(&FdFeature::FourthMoment)
        || requested.contains(&FdFeature::Regularity);

    let need_sum_x4y = requested.contains(&FdFeature::FourthMoment)
        || requested.contains(&FdFeature::Regularity);

    let need_f16 = requested.contains(&FdFeature::Centroid)
        || requested.contains(&FdFeature::RmsSpread)
        || requested.contains(&FdFeature::SpreadRatio)
        || requested.contains(&FdFeature::WeightedSkew)
        || requested.contains(&FdFeature::WeightedKurt);

    let need_f17 = requested.contains(&FdFeature::RmsSpread)
        || requested.contains(&FdFeature::SpreadRatio)
        || requested.contains(&FdFeature::WeightedSkew)
        || requested.contains(&FdFeature::WeightedKurt);

    // Compute intermediates conditionally

    // f12: mean(y)
    let f_12 = if need_f12 {
        y.iter().sum::<f64>() / k
    } else {
        0.0
    };

    // f13: var(y) ddof=0
    let f_13 = if need_f13 {
        y.iter().map(|&v| (v - f_12).powi(2)).sum::<f64>() / k
    } else {
        0.0
    };

    // sum_y, sum_xy, sum_x2y, sum_x4y — single pass
    let mut sum_y = 0.0_f64;
    let mut sum_xy = 0.0_f64;
    let mut sum_x2y = 0.0_f64;
    let mut sum_x4y = 0.0_f64;

    if need_sum_y || need_sum_xy || need_sum_x2y || need_sum_x4y || need_f16 {
        for (i, &yi) in y.iter().enumerate() {
            let xi = x[i];
            if need_sum_y || need_f16 {
                sum_y += yi;
            }
            if need_sum_xy || need_f16 {
                sum_xy += xi * yi;
            }
            if need_sum_x2y {
                sum_x2y += xi * xi * yi;
            }
            if need_sum_x4y {
                sum_x4y += xi.powi(4) * yi;
            }
        }
    }

    // f16: centroid
    let f_16 = if need_f16 { sum_xy / sum_y } else { 0.0 };

    // f17: RMS spread around centroid
    let f_17 = if need_f17 {
        (x.iter()
            .zip(y.iter())
            .map(|(&xi, &yi)| (xi - f_16).powi(2) * yi)
            .sum::<f64>()
            / k)
            .sqrt()
    } else {
        0.0
    };

    // Map requested features to values in request order
    features
        .iter()
        .map(|feat| match feat {
            FdFeature::Mean => f_12,
            FdFeature::Variance => f_13,
            FdFeature::Skewness => {
                let f13_sqrt = f_13.sqrt();
                y.iter()
                    .map(|&v| (v - f_12).powi(3))
                    .sum::<f64>()
                    / (k * f13_sqrt.powi(3))
            }
            FdFeature::Kurtosis => {
                y.iter()
                    .map(|&v| (v - f_12).powi(4))
                    .sum::<f64>()
                    / (k * f_13.powi(2))
            }
            FdFeature::Centroid => f_16,
            FdFeature::RmsSpread => f_17,
            FdFeature::RmsFreq => (sum_x2y / sum_y).sqrt(),
            FdFeature::FourthMoment => (sum_x4y / sum_x2y).sqrt(),
            FdFeature::Regularity => sum_x2y / (sum_y * sum_x4y).sqrt(),
            FdFeature::SpreadRatio => f_17 / f_16,
            FdFeature::WeightedSkew => {
                x.iter()
                    .zip(y.iter())
                    .map(|(&xi, &yi)| (xi - f_16).powi(3) * yi)
                    .sum::<f64>()
                    / (k * f_17.powi(3))
            }
            FdFeature::WeightedKurt => {
                x.iter()
                    .zip(y.iter())
                    .map(|(&xi, &yi)| (xi - f_16).powi(4) * yi)
                    .sum::<f64>()
                    / (k * f_17.powi(4))
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Original all-features functions — delegate to _select
// ---------------------------------------------------------------------------

/// 13 time-domain features matching `time_domain_fea()` in feature.py.
///
/// Returns: [max, min, mean, std(ddof=1), square-root amplitude, RMS,
///           peak-peak, skewness, kurtosis(Fisher), crest factor,
///           clearance factor, shape factor, impulse factor]
pub fn time_domain_features(signal: &[f64]) -> Vec<f64> {
    time_domain_features_select(signal, TdFeature::ALL)
}

/// 12 frequency-domain features matching `freq_domain_fea()` in feature.py.
///
/// `pl`: FFT magnitude array, `f`: frequency array.
/// Returns: [f12..f23] per Lei et al. (2007) naming.
pub fn freq_domain_features(pl: &[f64], f: &[f64]) -> Vec<f64> {
    freq_domain_features_select(pl, f, FdFeature::ALL)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signal() -> Vec<f64> {
        (0..256)
            .map(|i| {
                let t = i as f64 / 256.0;
                (2.0 * std::f64::consts::PI * 10.0 * t).sin()
                    + 0.5 * (2.0 * std::f64::consts::PI * 30.0 * t).cos()
                    + 0.1 * t
            })
            .collect()
    }

    // -- Original tests (backward compat) --

    #[test]
    fn test_time_domain_returns_13() {
        let sig = make_signal();
        let feats = time_domain_features(&sig);
        assert_eq!(feats.len(), 13);
    }

    #[test]
    fn test_time_domain_max_min() {
        let sig = vec![1.0, -2.0, 3.0, -4.0, 5.0];
        let feats = time_domain_features(&sig);
        assert!((feats[0] - 5.0).abs() < 1e-12);
        assert!((feats[1] - -4.0).abs() < 1e-12);
    }

    #[test]
    fn test_time_domain_mean_std() {
        let sig = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let feats = time_domain_features(&sig);
        assert!((feats[2] - 5.0).abs() < 1e-12);
        let expected_std = (32.0_f64 / 7.0).sqrt();
        assert!((feats[3] - expected_std).abs() < 1e-12);
    }

    #[test]
    fn test_time_domain_rms() {
        let sig = vec![1.0, -1.0, 1.0, -1.0];
        let feats = time_domain_features(&sig);
        assert!((feats[5] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_time_domain_peak_peak() {
        let sig = vec![-3.0, 0.0, 5.0, 1.0];
        let feats = time_domain_features(&sig);
        assert!((feats[6] - 4.0).abs() < 1e-12);
    }

    #[test]
    fn test_freq_domain_returns_12() {
        let pl: Vec<f64> = (1..=100).map(|i| i as f64 * 0.01).collect();
        let f: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let feats = freq_domain_features(&pl, &f);
        assert_eq!(feats.len(), 12);
    }

    #[test]
    fn test_freq_domain_mean() {
        let pl = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let f = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let feats = freq_domain_features(&pl, &f);
        assert!((feats[0] - 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_freq_domain_centroid() {
        let pl = vec![1.0, 1.0, 1.0];
        let f = vec![10.0, 20.0, 30.0];
        let feats = freq_domain_features(&pl, &f);
        assert!((feats[4] - 20.0).abs() < 1e-12);
    }

    // -- Selective computation tests --

    #[test]
    fn test_td_select_all_matches_original() {
        let sig = make_signal();
        let all = time_domain_features(&sig);
        let sel = time_domain_features_select(&sig, TdFeature::ALL);
        assert_eq!(all.len(), sel.len());
        for (a, b) in all.iter().zip(sel.iter()) {
            assert!((a - b).abs() < 1e-12, "mismatch: {} vs {}", a, b);
        }
    }

    #[test]
    fn test_fd_select_all_matches_original() {
        let pl: Vec<f64> = (1..=100).map(|i| i as f64 * 0.01).collect();
        let f: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let all = freq_domain_features(&pl, &f);
        let sel = freq_domain_features_select(&pl, &f, FdFeature::ALL);
        assert_eq!(all.len(), sel.len());
        for (a, b) in all.iter().zip(sel.iter()) {
            assert!((a - b).abs() < 1e-12, "mismatch: {} vs {}", a, b);
        }
    }

    #[test]
    fn test_td_select_subset_correct_order() {
        let sig = make_signal();
        let all = time_domain_features(&sig);
        // Request [rms, kurtosis, max] → should return [all[5], all[8], all[0]]
        let sel = time_domain_features_select(
            &sig,
            &[TdFeature::Rms, TdFeature::Kurtosis, TdFeature::Max],
        );
        assert_eq!(sel.len(), 3);
        assert!((sel[0] - all[5]).abs() < 1e-12); // rms
        assert!((sel[1] - all[8]).abs() < 1e-12); // kurtosis
        assert!((sel[2] - all[0]).abs() < 1e-12); // max
    }

    #[test]
    fn test_fd_select_subset_correct_order() {
        let pl: Vec<f64> = (1..=50).map(|i| i as f64 * 0.1).collect();
        let f: Vec<f64> = (1..=50).map(|i| i as f64 * 10.0).collect();
        let all = freq_domain_features(&pl, &f);
        // Request [centroid, mean] → should return [all[4], all[0]]
        let sel = freq_domain_features_select(&pl, &f, &[FdFeature::Centroid, FdFeature::Mean]);
        assert_eq!(sel.len(), 2);
        assert!((sel[0] - all[4]).abs() < 1e-12); // centroid
        assert!((sel[1] - all[0]).abs() < 1e-12); // mean
    }

    #[test]
    fn test_td_from_name_english() {
        assert_eq!(TdFeature::from_name("rms"), Some(TdFeature::Rms));
        assert_eq!(TdFeature::from_name("kurtosis"), Some(TdFeature::Kurtosis));
        assert_eq!(TdFeature::from_name("peak_peak"), Some(TdFeature::PeakPeak));
    }

    #[test]
    fn test_td_from_name_paper_id() {
        assert_eq!(TdFeature::from_name("t4"), Some(TdFeature::Rms));
        assert_eq!(TdFeature::from_name("t7"), Some(TdFeature::Kurtosis));
        assert_eq!(TdFeature::from_name("t_max"), Some(TdFeature::Max));
    }

    #[test]
    fn test_fd_from_name_english() {
        assert_eq!(FdFeature::from_name("centroid"), Some(FdFeature::Centroid));
        assert_eq!(FdFeature::from_name("regularity"), Some(FdFeature::Regularity));
    }

    #[test]
    fn test_fd_from_name_paper_id() {
        assert_eq!(FdFeature::from_name("f16"), Some(FdFeature::Centroid));
        assert_eq!(FdFeature::from_name("f20"), Some(FdFeature::Regularity));
    }

    #[test]
    fn test_from_name_unknown() {
        assert_eq!(TdFeature::from_name("nonexistent"), None);
        assert_eq!(FdFeature::from_name("f99"), None);
    }

    #[test]
    fn test_td_english_and_paper_id_produce_same_values() {
        let sig = make_signal();
        let by_english = time_domain_features_select(
            &sig,
            &[TdFeature::from_name("rms").unwrap(), TdFeature::from_name("kurtosis").unwrap()],
        );
        let by_paper = time_domain_features_select(
            &sig,
            &[TdFeature::from_name("t4").unwrap(), TdFeature::from_name("t7").unwrap()],
        );
        assert!((by_english[0] - by_paper[0]).abs() < 1e-12);
        assert!((by_english[1] - by_paper[1]).abs() < 1e-12);
    }
}
