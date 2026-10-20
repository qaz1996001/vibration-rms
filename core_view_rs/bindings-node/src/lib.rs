#[macro_use]
extern crate napi_derive;

use napi::{Error, Status};
use vibration_rms_core as core;

fn parse_td_features(names: Vec<String>) -> napi::Result<Vec<core::TdFeature>> {
    names
        .iter()
        .map(|name| {
            core::TdFeature::from_name(name).ok_or_else(|| {
                Error::new(
                    Status::InvalidArg,
                    format!("Unknown time-domain feature: '{}'", name),
                )
            })
        })
        .collect()
}

fn parse_fd_features(names: Vec<String>) -> napi::Result<Vec<core::FdFeature>> {
    names
        .iter()
        .map(|name| {
            core::FdFeature::from_name(name).ok_or_else(|| {
                Error::new(
                    Status::InvalidArg,
                    format!("Unknown frequency-domain feature: '{}'", name),
                )
            })
        })
        .collect()
}

#[napi]
fn acceleration_rms(
    data: Vec<f64>,
    sampling_rate: f64,
    freq_lower_bound: Option<f64>,
    freq_upper_bound: Option<f64>,
) -> Vec<f64> {
    let f_lo = freq_lower_bound.unwrap_or(10.0);
    let f_hi = freq_upper_bound.unwrap_or(1000.0);
    let (acc, vel, dis) = core::acceleration_rms(&data, sampling_rate, f_lo, f_hi);
    vec![acc, vel, dis]
}

#[napi]
fn acceleration_rms2(
    data: Vec<f64>,
    sampling_rate: f64,
    freq_lower_bound: Option<f64>,
    freq_upper_bound: Option<f64>,
) -> Vec<f64> {
    let f_lo = freq_lower_bound.unwrap_or(10.0);
    let f_hi = freq_upper_bound.unwrap_or(1000.0);
    let (acc, vel, dis) = core::acceleration_rms2(&data, sampling_rate, f_lo, f_hi);
    vec![acc, vel, dis]
}

#[napi]
fn get_fft_abs_data(data: Vec<f64>, sampling_rate: f64) -> Vec<Vec<f64>> {
    let (abs_data, freqs, angles) = core::get_fft_abs_data(&data, sampling_rate);
    vec![abs_data, freqs, angles]
}

#[napi]
fn time_domain_features(
    signal: Vec<f64>,
    features: Option<Vec<String>>,
) -> napi::Result<Vec<f64>> {
    let result = match features {
        None => core::time_domain_features(&signal),
        Some(names) => {
            let feats = parse_td_features(names)?;
            core::time_domain_features_select(&signal, &feats)
        }
    };
    Ok(result)
}

#[napi]
fn freq_domain_features(
    pl: Vec<f64>,
    f: Vec<f64>,
    features: Option<Vec<String>>,
) -> napi::Result<Vec<f64>> {
    let result = match features {
        None => core::freq_domain_features(&pl, &f),
        Some(names) => {
            let feats = parse_fd_features(names)?;
            core::freq_domain_features_select(&pl, &f, &feats)
        }
    };
    Ok(result)
}

#[napi(object)]
pub struct BothFeaturesResult {
    pub time_domain_feature: Vec<f64>,
    pub freq_domain_feature: Vec<f64>,
    pub axis: String,
    pub create_time: String,
}

#[napi]
fn both_features(
    signal: Vec<f64>,
    pl: Vec<f64>,
    f: Vec<f64>,
    axis: String,
    create_time: String,
    time_features: Option<Vec<String>>,
    freq_features: Option<Vec<String>>,
) -> napi::Result<BothFeaturesResult> {
    let td = match time_features {
        None => core::time_domain_features(&signal),
        Some(names) => {
            let feats = parse_td_features(names)?;
            core::time_domain_features_select(&signal, &feats)
        }
    };

    let fd = match freq_features {
        None => core::freq_domain_features(&pl, &f),
        Some(names) => {
            let feats = parse_fd_features(names)?;
            core::freq_domain_features_select(&pl, &f, &feats)
        }
    };

    Ok(BothFeaturesResult {
        time_domain_feature: td,
        freq_domain_feature: fd,
        axis,
        create_time,
    })
}
