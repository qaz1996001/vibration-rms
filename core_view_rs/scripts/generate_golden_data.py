"""
Generate golden reference data for Rust cross-validation.

Usage:
    python scripts/generate_golden_data.py

Output:
    scripts/golden_data.json
"""

import json
import sys
import os

import numpy as np

# Add parent directory to import the Python modules
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from vibration import Vibration
from feature import time_domain_fea, freq_domain_fea

np.random.seed(42)

# Test signal: 1024 samples, 4096 Hz sampling rate
N = 1024
SR = 4096.0
t = np.arange(N) / SR
signal = (
    0.5 * np.sin(2 * np.pi * 100 * t)
    + 0.3 * np.cos(2 * np.pi * 250 * t)
    + 0.1 * np.random.randn(N)
)

# vibration.acceleration_rms
acc_rms1, vel_rms1, dis_pkpk1 = Vibration.acceleration_rms(signal, SR)

# vibration.acceleration_rms2
acc_rms2, vel_rms2, dis_pkpk2 = Vibration.acceleration_rms2(signal, SR)

# vibration.get_fft_abs_data
fft_abs, freqs, angles = Vibration.get_fft_abs_data(signal, SR)

# feature.time_domain_fea
td_features = time_domain_fea(signal)

# feature.freq_domain_fea (using FFT output)
fd_features = freq_domain_fea(PL=fft_abs, f=freqs)

golden = {
    'signal': signal.tolist(),
    'sampling_rate': SR,
    'n': N,
    'acceleration_rms': {
        'acc_rms': float(acc_rms1),
        'vel_rms': float(vel_rms1),
        'dis_pkpk': float(dis_pkpk1),
    },
    'acceleration_rms2': {
        'acc_rms': float(acc_rms2),
        'vel_rms': float(vel_rms2),
        'dis_pkpk': float(dis_pkpk2),
    },
    'get_fft_abs_data': {
        'fft_abs': fft_abs.tolist(),
        'freqs': freqs.tolist(),
        'angles': angles.tolist(),
    },
    'time_domain_features': td_features.tolist(),
    'freq_domain_features': fd_features.tolist(),
}

output_path = os.path.join(os.path.dirname(__file__), 'golden_data.json')
with open(output_path, 'w') as f:
    json.dump(golden, f, indent=2)

print(f"Golden data written to {output_path}")
print(f"  Signal length: {N}")
print(f"  acc_rms1:  {acc_rms1:.15e}")
print(f"  vel_rms1:  {vel_rms1:.15e}")
print(f"  dis_pkpk1: {dis_pkpk1:.15e}")
print(f"  acc_rms2:  {acc_rms2:.15e}")
print(f"  vel_rms2:  {vel_rms2:.15e}")
print(f"  dis_pkpk2: {dis_pkpk2:.15e}")
print(f"  td_features: {td_features}")
print(f"  fd_features: {fd_features}")
