#[macro_use]
extern crate napi_derive;

use vibration_rms_core as core;

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
fn time_domain_features(signal: Vec<f64>) -> Vec<f64> {
    core::time_domain_features(&signal)
}

#[napi]
fn freq_domain_features(pl: Vec<f64>, f: Vec<f64>) -> Vec<f64> {
    core::freq_domain_features(&pl, &f)
}
