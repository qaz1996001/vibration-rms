import numpy as np
import numpy.typing as npt

def acceleration_rms(
    data: npt.NDArray[np.float64],
    sampling_rate: float,
    freq_lower_bound: float = 10.0,
    freq_upper_bound: float = 1000.0,
) -> tuple[float, float, float]:
    """Returns (acc_rms, vel_rms, dis_pkpk) via inverse FFT. g = 9.8."""
    ...

def acceleration_rms2(
    data: npt.NDArray[np.float64],
    sampling_rate: float,
    freq_lower_bound: float = 10.0,
    freq_upper_bound: float = 1000.0,
) -> tuple[float, float, float]:
    """Returns (acc_rms, vel_rms, dis_pkpk) via PSD. g = 9.81."""
    ...

def get_fft_abs_data(
    data: npt.NDArray[np.float64],
    sampling_rate: float,
) -> tuple[npt.NDArray[np.float64], npt.NDArray[np.float64], npt.NDArray[np.float64]]:
    """Returns (fft_abs, freq_ticks, angle_data)."""
    ...

def time_domain_features(
    signal: npt.NDArray[np.float64],
) -> npt.NDArray[np.float64]:
    """Returns 13 time-domain features."""
    ...

def freq_domain_features(
    pl: npt.NDArray[np.float64],
    f: npt.NDArray[np.float64],
) -> npt.NDArray[np.float64]:
    """Returns 12 frequency-domain features."""
    ...

def both_features(
    signal: npt.NDArray[np.float64],
    pl: npt.NDArray[np.float64],
    f: npt.NDArray[np.float64],
    axis: str,
    create_time: str,
) -> dict[str, npt.NDArray[np.float64] | str]:
    """Returns dict with time_domain_feature, freq_domain_feature, axis, create_time."""
    ...
