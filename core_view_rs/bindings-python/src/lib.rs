use numpy::{PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use core_view_core::{self as core};

#[pyfunction]
#[pyo3(signature = (data, sampling_rate, freq_lower_bound=10.0, freq_upper_bound=1000.0))]
fn acceleration_rms<'py>(
    _py: Python<'py>,
    data: PyReadonlyArray1<'py, f64>,
    sampling_rate: f64,
    freq_lower_bound: f64,
    freq_upper_bound: f64,
) -> (f64, f64, f64) {
    let slice = data.as_slice().unwrap();
    core::acceleration_rms(slice, sampling_rate, freq_lower_bound, freq_upper_bound)
}

#[pyfunction]
#[pyo3(signature = (data, sampling_rate, freq_lower_bound=10.0, freq_upper_bound=1000.0))]
fn acceleration_rms2<'py>(
    _py: Python<'py>,
    data: PyReadonlyArray1<'py, f64>,
    sampling_rate: f64,
    freq_lower_bound: f64,
    freq_upper_bound: f64,
) -> (f64, f64, f64) {
    let slice = data.as_slice().unwrap();
    core::acceleration_rms2(slice, sampling_rate, freq_lower_bound, freq_upper_bound)
}

type PyArray1Tuple<'py> = (
    Bound<'py, PyArray1<f64>>,
    Bound<'py, PyArray1<f64>>,
    Bound<'py, PyArray1<f64>>,
);

#[pyfunction]
fn get_fft_abs_data<'py>(
    py: Python<'py>,
    data: PyReadonlyArray1<'py, f64>,
    sampling_rate: f64,
) -> PyArray1Tuple<'py> {
    let slice = data.as_slice().unwrap();
    let (abs_data, freqs, angles) = core::get_fft_abs_data(slice, sampling_rate);
    (
        PyArray1::from_vec(py, abs_data),
        PyArray1::from_vec(py, freqs),
        PyArray1::from_vec(py, angles),
    )
}

fn parse_td_features(names: Vec<String>) -> PyResult<Vec<core::TdFeature>> {
    names
        .iter()
        .map(|name| {
            core::TdFeature::from_name(name).ok_or_else(|| {
                pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown time-domain feature: '{}'",
                    name
                ))
            })
        })
        .collect()
}

fn parse_fd_features(names: Vec<String>) -> PyResult<Vec<core::FdFeature>> {
    names
        .iter()
        .map(|name| {
            core::FdFeature::from_name(name).ok_or_else(|| {
                pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown frequency-domain feature: '{}'",
                    name
                ))
            })
        })
        .collect()
}

#[pyfunction]
#[pyo3(signature = (signal, features=None))]
fn time_domain_features<'py>(
    py: Python<'py>,
    signal: PyReadonlyArray1<'py, f64>,
    features: Option<Vec<String>>,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let slice = signal.as_slice().unwrap();
    let result = match features {
        None => core::time_domain_features(slice),
        Some(names) => {
            let feats = parse_td_features(names)?;
            core::time_domain_features_select(slice, &feats)
        }
    };
    Ok(PyArray1::from_vec(py, result))
}

#[pyfunction]
#[pyo3(signature = (pl, f, features=None))]
fn freq_domain_features<'py>(
    py: Python<'py>,
    pl: PyReadonlyArray1<'py, f64>,
    f: PyReadonlyArray1<'py, f64>,
    features: Option<Vec<String>>,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let pl_slice = pl.as_slice().unwrap();
    let f_slice = f.as_slice().unwrap();
    let result = match features {
        None => core::freq_domain_features(pl_slice, f_slice),
        Some(names) => {
            let feats = parse_fd_features(names)?;
            core::freq_domain_features_select(pl_slice, f_slice, &feats)
        }
    };
    Ok(PyArray1::from_vec(py, result))
}

#[pyfunction]
#[pyo3(signature = (signal, pl, f, axis, create_time, time_features=None, freq_features=None))]
fn both_features<'py>(
    py: Python<'py>,
    signal: PyReadonlyArray1<'py, f64>,
    pl: PyReadonlyArray1<'py, f64>,
    f: PyReadonlyArray1<'py, f64>,
    axis: &str,
    create_time: &str,
    time_features: Option<Vec<String>>,
    freq_features: Option<Vec<String>>,
) -> PyResult<Bound<'py, PyDict>> {
    let sig_slice = signal.as_slice().unwrap();
    let pl_slice = pl.as_slice().unwrap();
    let f_slice = f.as_slice().unwrap();

    let td = match time_features {
        None => core::time_domain_features(sig_slice),
        Some(names) => {
            let feats = parse_td_features(names)?;
            core::time_domain_features_select(sig_slice, &feats)
        }
    };

    let fd = match freq_features {
        None => core::freq_domain_features(pl_slice, f_slice),
        Some(names) => {
            let feats = parse_fd_features(names)?;
            core::freq_domain_features_select(pl_slice, f_slice, &feats)
        }
    };

    let dict = PyDict::new(py);
    dict.set_item("time_domain_feature", PyArray1::from_vec(py, td))?;
    dict.set_item("freq_domain_feature", PyArray1::from_vec(py, fd))?;
    dict.set_item("axis", axis)?;
    dict.set_item("create_time", create_time)?;
    Ok(dict)
}

#[pymodule]
fn core_view(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(acceleration_rms, m)?)?;
    m.add_function(wrap_pyfunction!(acceleration_rms2, m)?)?;
    m.add_function(wrap_pyfunction!(get_fft_abs_data, m)?)?;
    m.add_function(wrap_pyfunction!(time_domain_features, m)?)?;
    m.add_function(wrap_pyfunction!(freq_domain_features, m)?)?;
    m.add_function(wrap_pyfunction!(both_features, m)?)?;
    Ok(())
}
