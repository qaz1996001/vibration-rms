"""
Cross-validation tests for the PyO3 Python bindings.

Validates that the Rust-backed `vibration_rms` module produces results
matching the original Python (NumPy/SciPy) implementation via golden data.
"""

import json
import os
import sys

import numpy as np
import numpy.testing as npt

# Import the Rust-backed module
import vibration_rms

GOLDEN_PATH = os.path.join(os.path.dirname(__file__), '..', 'scripts', 'golden_data.json')

with open(GOLDEN_PATH) as f:
    GOLDEN = json.load(f)

SIGNAL = np.array(GOLDEN['signal'], dtype=np.float64)
SR = GOLDEN['sampling_rate']
RTOL = 1e-6


def test_acceleration_rms():
    acc, vel, dis = core_view.acceleration_rms(SIGNAL, SR)
    py = GOLDEN['acceleration_rms']
    npt.assert_allclose(acc, py['acc_rms'], rtol=RTOL, err_msg='acc_rms')
    npt.assert_allclose(vel, py['vel_rms'], rtol=RTOL, err_msg='vel_rms')
    npt.assert_allclose(dis, py['dis_pkpk'], rtol=RTOL, err_msg='dis_pkpk')


def test_acceleration_rms_custom_band():
    acc, vel, dis = core_view.acceleration_rms(SIGNAL, SR, 20.0, 500.0)
    assert acc > 0, 'acc_rms should be positive'
    assert vel > 0, 'vel_rms should be positive'
    assert dis > 0, 'dis_pkpk should be positive'


def test_acceleration_rms_default_params():
    """Ensure default freq bounds (10, 1000) work."""
    r1 = core_view.acceleration_rms(SIGNAL, SR)
    r2 = core_view.acceleration_rms(SIGNAL, SR, 10.0, 1000.0)
    for a, b in zip(r1, r2):
        npt.assert_equal(a, b)


def test_acceleration_rms2():
    acc, vel, dis = core_view.acceleration_rms2(SIGNAL, SR)
    py = GOLDEN['acceleration_rms2']
    npt.assert_allclose(acc, py['acc_rms'], rtol=RTOL, err_msg='acc_rms')
    npt.assert_allclose(vel, py['vel_rms'], rtol=RTOL, err_msg='vel_rms')
    npt.assert_allclose(dis, py['dis_pkpk'], rtol=RTOL, err_msg='dis_pkpk')


def test_get_fft_abs_data():
    fft_abs, freqs, angles = core_view.get_fft_abs_data(SIGNAL, SR)
    py_fft_abs = np.array(GOLDEN['get_fft_abs_data']['fft_abs'])
    py_freqs = np.array(GOLDEN['get_fft_abs_data']['freqs'])
    py_angles = np.array(GOLDEN['get_fft_abs_data']['angles'])

    assert len(fft_abs) == len(py_fft_abs), 'fft_abs length mismatch'
    assert len(freqs) == len(py_freqs), 'freqs length mismatch'
    assert len(angles) == len(py_angles), 'angles length mismatch'

    npt.assert_allclose(freqs, py_freqs, rtol=1e-12, err_msg='freqs')
    npt.assert_allclose(fft_abs, py_fft_abs, rtol=RTOL, err_msg='fft_abs')


def test_get_fft_abs_data_returns_numpy():
    fft_abs, freqs, angles = core_view.get_fft_abs_data(SIGNAL, SR)
    assert isinstance(fft_abs, np.ndarray), 'fft_abs should be numpy array'
    assert isinstance(freqs, np.ndarray), 'freqs should be numpy array'
    assert isinstance(angles, np.ndarray), 'angles should be numpy array'
    assert fft_abs.dtype == np.float64
    assert freqs.dtype == np.float64
    assert angles.dtype == np.float64


def test_time_domain_features():
    td = core_view.time_domain_features(SIGNAL)
    py_td = np.array(GOLDEN['time_domain_features'])

    assert len(td) == 13, f'expected 13 features, got {len(td)}'
    npt.assert_allclose(td, py_td, rtol=RTOL, err_msg='time_domain_features')


def test_time_domain_features_returns_numpy():
    td = core_view.time_domain_features(SIGNAL)
    assert isinstance(td, np.ndarray)
    assert td.dtype == np.float64


def test_time_domain_features_selective():
    td_all = core_view.time_domain_features(SIGNAL)
    td_sel = core_view.time_domain_features(SIGNAL, ['rms', 'kurtosis', 'max'])

    assert len(td_sel) == 3
    npt.assert_allclose(td_sel[0], td_all[5], rtol=1e-12, err_msg='rms')
    npt.assert_allclose(td_sel[1], td_all[8], rtol=1e-12, err_msg='kurtosis')
    npt.assert_allclose(td_sel[2], td_all[0], rtol=1e-12, err_msg='max')


def test_time_domain_features_paper_ids():
    td_english = core_view.time_domain_features(SIGNAL, ['rms', 'kurtosis'])
    td_paper = core_view.time_domain_features(SIGNAL, ['t4', 't7'])
    npt.assert_array_equal(td_english, td_paper)


def test_time_domain_features_invalid_name():
    try:
        core_view.time_domain_features(SIGNAL, ['nonexistent'])
        assert False, 'Should have raised ValueError'
    except ValueError as e:
        assert 'nonexistent' in str(e)


def test_freq_domain_features():
    py_fft_abs = np.array(GOLDEN['get_fft_abs_data']['fft_abs'], dtype=np.float64)
    py_freqs = np.array(GOLDEN['get_fft_abs_data']['freqs'], dtype=np.float64)
    py_fd = np.array(GOLDEN['freq_domain_features'])

    fd = core_view.freq_domain_features(py_fft_abs, py_freqs)

    assert len(fd) == 12, f'expected 12 features, got {len(fd)}'
    npt.assert_allclose(fd, py_fd, rtol=RTOL, err_msg='freq_domain_features')


def test_freq_domain_features_selective():
    py_fft_abs = np.array(GOLDEN['get_fft_abs_data']['fft_abs'], dtype=np.float64)
    py_freqs = np.array(GOLDEN['get_fft_abs_data']['freqs'], dtype=np.float64)

    fd_all = core_view.freq_domain_features(py_fft_abs, py_freqs)
    fd_sel = core_view.freq_domain_features(py_fft_abs, py_freqs, ['centroid', 'mean'])

    assert len(fd_sel) == 2
    npt.assert_allclose(fd_sel[0], fd_all[4], rtol=1e-12, err_msg='centroid')
    npt.assert_allclose(fd_sel[1], fd_all[0], rtol=1e-12, err_msg='mean')


def test_freq_domain_features_paper_ids():
    py_fft_abs = np.array(GOLDEN['get_fft_abs_data']['fft_abs'], dtype=np.float64)
    py_freqs = np.array(GOLDEN['get_fft_abs_data']['freqs'], dtype=np.float64)

    fd_english = core_view.freq_domain_features(py_fft_abs, py_freqs, ['centroid'])
    fd_paper = core_view.freq_domain_features(py_fft_abs, py_freqs, ['f16'])
    npt.assert_array_equal(fd_english, fd_paper)


def test_both_features():
    fft_abs, freqs, _ = core_view.get_fft_abs_data(SIGNAL, SR)
    result = core_view.both_features(SIGNAL, fft_abs, freqs, 'X', '2026-03-18T00:00:00')

    assert isinstance(result, dict)
    assert 'time_domain_feature' in result
    assert 'freq_domain_feature' in result
    assert result['axis'] == 'X'
    assert result['create_time'] == '2026-03-18T00:00:00'

    td = result['time_domain_feature']
    fd = result['freq_domain_feature']
    assert isinstance(td, np.ndarray)
    assert isinstance(fd, np.ndarray)
    assert len(td) == 13
    assert len(fd) == 12


def test_both_features_selective():
    fft_abs, freqs, _ = core_view.get_fft_abs_data(SIGNAL, SR)
    result = core_view.both_features(
        SIGNAL, fft_abs, freqs, 'Y', '2026-01-01',
        time_features=['rms', 'max'],
        freq_features=['centroid'],
    )
    assert len(result['time_domain_feature']) == 2
    assert len(result['freq_domain_feature']) == 1


def test_end_to_end_pipeline():
    """Full pipeline: signal -> FFT -> features, all through Rust bindings."""
    fft_abs, freqs, angles = core_view.get_fft_abs_data(SIGNAL, SR)
    acc, vel, dis = core_view.acceleration_rms(SIGNAL, SR)
    td = core_view.time_domain_features(SIGNAL)
    fd = core_view.freq_domain_features(fft_abs, freqs)
    result = core_view.both_features(SIGNAL, fft_abs, freqs, 'Z', '2026-03-18')

    # Cross-check: td from both_features should match standalone td
    npt.assert_array_equal(result['time_domain_feature'], td)
    npt.assert_array_equal(result['freq_domain_feature'], fd)


# ─── Run ──────────────────────────────────────────────────────────────

if __name__ == '__main__':
    tests = [v for k, v in sorted(globals().items()) if k.startswith('test_')]
    passed = 0
    failed = 0
    for test_fn in tests:
        name = test_fn.__name__
        try:
            test_fn()
            print(f'  PASS  {name}')
            passed += 1
        except Exception as e:
            print(f'  FAIL  {name}: {e}')
            failed += 1

    print(f'\n{passed + failed} tests: {passed} passed, {failed} failed')
    sys.exit(1 if failed else 0)
