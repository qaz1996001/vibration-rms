# vibration-rms

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![Python](https://img.shields.io/badge/Python-3.8+-blue)
![Node.js](https://img.shields.io/badge/Node.js-14+-green)

A high-performance rotating-machinery vibration analysis and feature-extraction library. Implemented in Rust with Python (PyO3) and Node.js (NAPI-RS) bindings, with cross-validated golden-data architecture.

## Overview

**English:**  
Vibration-RMS provides efficient computation of rotating-machinery vibration metrics and features. The library implements acceleration/velocity/displacement RMS via two methods (inverse-FFT and PSD), FFT-based magnitude/frequency/phase extraction, and Lei et al. (2007)'s 13 time-domain and 12 frequency-domain vibration features. Originally a NumPy/SciPy reference implementation, the project has been ported to Rust for production-grade performance while maintaining bit-accurate validation against the Python original via a golden-data cross-validation discipline.

**中文:**  
振動-RMS 是一個高性能旋轉機械振動分析與特徵提取函式庫。提供加速度/速度/位移 RMS 計算（逆 FFT 與 PSD 雙方法）、FFT 頻率/振幅/相位提取、以及 Lei 等人 (2007) 提出的 13 個時域與 12 個頻域振動特徵。原始實作為 NumPy/SciPy 版本，現已改寫為 Rust，通過黃金數據驗證機制保證與 Python 版本的比特精度一致性。

## Features

- **Multi-method RMS calculation**: inverse-FFT and PSD methods for acceleration/velocity/displacement
- **FFT analysis**: magnitude, frequency, and phase extraction aligned with NumPy conventions
- **Time-domain features** (13): RMS, crest factor, kurtosis, skewness, peak, peak-peak, variance, mean absolute value, and more
- **Frequency-domain features** (12): centroid, mean, standard deviation, kurtosis, skewness, variance, and more
- **Dual naming**: reference both English names (`"rms"`, `"kurtosis"`) and Lei et al. paper IDs (`"t4"`, `"t7"`, `"f16"`, etc.)
- **Selective computation**: request only the features you need for performance
- **Cross-platform**: Windows and Linux, Rust/Python/Node.js
- **Golden-data validated**: all outputs verified against NumPy/SciPy reference at `rtol=1e-6`

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Reference Implementation (root)                    │
│   vibration.py, feature.py (NumPy/SciPy)            │
│   scripts/generate_golden_data.py → golden_data.json│
└────────────────┬────────────────────────────────────┘
                 │ cross-validate (rtol=1e-6)
                 ▼
┌─────────────────────────────────────────────────────┐
│  Rust Core (core_view_rs/core/)                     │
│   ├─ vibration.rs: acceleration_rms, acceleration_rms2, get_fft_abs_data
│   ├─ feature.rs: time/freq domain feature extraction
│   └─ fft_utils.rs: FFT helpers (rfftfreq, bandpass, etc.)
└────────┬───────────────────────────────┬────────────┘
         │                               │
         ▼                               ▼
    ┌─────────────┐          ┌──────────────────┐
    │ Python PyO3 │          │  Node.js NAPI-RS │
    │ (maturin)   │          │                  │
    └─────────────┘          └──────────────────┘
```

## Installation

### Rust
```bash
# Add to your Cargo.toml
cargo add vibration-rms-core

# Or git dependency
cargo add --git https://github.com/qaz1006001/vibration-rms vibration-rms-core
```

### Python
```bash
# Install from PyPI (once published)
pip install vibration-rms

# Or build locally
cd core_view_rs/bindings-python
uv run maturin develop --release
```

### Node.js
```bash
# Install from npm (once published)
npm install @vibration-rms/native

# Or build locally
cd core_view_rs/bindings-node
npm install
npm run build
```

## Quick Start

### Rust
```rust
use vibration_rms_core::*;

let signal = vec![/* your acceleration data */];
let sr = 4096.0; // sampling rate in Hz

// Acceleration RMS (inverse-FFT method)
let (acc_rms, vel_rms, dis_pkpk) = acceleration_rms(&signal, sr, 10.0, 1000.0);

// FFT analysis
let (fft_abs, freqs, angles) = get_fft_abs_data(&signal, sr);

// Time-domain features (all 13)
let td_feats = time_domain_features(&signal);

// Selective features by English name or paper ID
let selected = time_domain_features_select(&signal, &[TdFeature::Rms, TdFeature::Kurtosis]);
```

### Python
```python
import vibration_rms
import numpy as np

signal = np.array([/* your acceleration data */])
sr = 4096.0

# Acceleration RMS
acc_rms, vel_rms, dis_pkpk = vibration_rms.acceleration_rms(signal, sr)

# Time-domain features (selective by English name or paper ID)
td_feats = vibration_rms.time_domain_features(signal, features=['rms', 'kurtosis'])
# or: features=['t4', 't7']

# Frequency-domain features (requires FFT data)
fft_abs, freqs, angles = vibration_rms.get_fft_abs_data(signal, sr)
fd_feats = vibration_rms.freq_domain_features(fft_abs, freqs, features=['centroid', 'f16'])

# Both domains together
result = vibration_rms.both_features(
    signal, fft_abs, freqs,
    axis='X', create_time='2026-03-18T00:00:00',
    time_features=['rms'], freq_features=['centroid']
)
print(result['time_domain_feature'], result['freq_domain_feature'])
```

### Node.js
```javascript
const native = require('@vibration-rms/native');

const signal = new Float64Array([/* your acceleration data */]);
const sr = 4096.0;

// Acceleration RMS
const [accRms, velRms, disPkpk] = native.accelerationRms(signal, sr);

// Time-domain features (selective by English name or paper ID)
const tdFeats = native.timeDomainFeatures(signal, ['rms', 'kurtosis']);
// or: ['t4', 't7']

// Frequency-domain features
const [fftAbs, freqs, angles] = native.getFftAbsData(signal, sr);
const fdFeats = native.freqDomainFeatures(fftAbs, freqs, ['centroid', 'f16']);

// Both domains together
const result = native.bothFeatures(
  signal, fftAbs, freqs,
  'X', '2026-03-18T00:00:00',
  ['rms'], ['centroid']
);
console.log(result.timeDomainFeature, result.freqDomainFeature);
```

## Golden-Data Cross-Validation

The core engineering discipline of this project is **golden-data validation**: all Rust/Python/Node.js outputs are validated against the original NumPy/SciPy reference implementation at relative tolerance `rtol=1e-6`.

### How it works
1. **Golden data generation** (`scripts/generate_golden_data.py`): Uses the original Python implementations to compute reference outputs on a deterministic synthetic signal (seeded, 100Hz + 250Hz + noise, 1024 samples @ 4096 Hz).
2. **Committed golden data** (`core_view_rs/scripts/golden_data.json`, 60 KB): All reference results are checked into git, providing a stable target for all three layers.
3. **Cross-validation tests**:
   - **Rust** (`core/tests/golden_cross_validation.rs`): `cargo test` reads the JSON and verifies all 5 functions match within tolerance.
   - **Python** (`bindings-python/test_bindings.py`): Imports the compiled `vibration_rms` module and validates against the same JSON.
   - **Node.js** (`bindings-node/test_bindings.mjs`): Loads the native addon and cross-checks.

If any implementation diverges from golden data, it signals a port bug (not a test problem) — the Python reference is the ground truth. This architecture eliminates subtle FFT normalization, boundary condition, and floating-point precision issues that plague many numerical library ports.

## Development & Testing

### Rust
```bash
cd core_view_rs

# Format and lint
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests (includes golden cross-validation)
cargo test

# Just the core library
cargo test -p vibration-rms-core
```

### Python
```bash
cd core_view_rs/bindings-python

# Install in development mode
uv run maturin develop --release

# Run tests
uv run python test_bindings.py
```

### Node.js
```bash
cd core_view_rs/bindings-node

# Build
npm run build

# Run tests
node test_bindings.mjs
```

## Feature Reference

### Time-Domain Features (13)
| ID | English Name | Lei et al. | Description |
|---|---|---|---|
| t1 | `max` | T₁ | Peak absolute value |
| t2 | `peak_peak` | T₂ | Peak-to-peak value |
| t3 | `rms` | T₃ | Root-mean-square |
| t4 | `rms` | T₄ | RMS (alternative method) |
| t5 | `mean_abs` | T₅ | Mean absolute value |
| t6 | `variance` | T₆ | Variance |
| t7 | `kurtosis` | T₇ | Kurtosis (peakedness) |
| t8 | `skewness` | T₈ | Skewness (asymmetry) |
| t9 | `crest` | T₉ | Crest factor |
| t10 | `clearance` | T₁₀ | Clearance factor |
| t11 | `impulse` | T₁₁ | Impulse factor |

### Frequency-Domain Features (12)
| ID | English Name | Lei et al. | Description |
|---|---|---|---|
| f12 | `mean` | F₁ | Mean of power spectrum |
| f13 | `centroid` | F₂ | Centroid (center of mass) |
| f14 | `variance` | F₃ | Variance of spectrum |
| f15 | `rms` | F₄ | RMS of spectrum |
| f16 | `std` | F₅ | Standard deviation |
| f17 | `skewness` | F₆ | Skewness |
| f18 | `kurtosis` | F₇ | Kurtosis |
| f19 | `p` | F₈ | Peak power |
| f20 | `energy` | F₉ | Total energy |
| f21 | `entropy` | F₁₀ | Entropy |
| f22 | `max` | F₁₁ | Peak frequency |
| f23 | `pow_norm` | F₁₂ | Normalized power |

## CI/CD

GitHub Actions workflow (`.github/workflows/ci.yml`) runs on every push and pull request:
- `cargo fmt --check` — code formatting
- `cargo clippy` — linting with warnings-as-errors
- `cargo test` — unit tests + golden cross-validation (golden data already committed, no Python setup needed)

## References

- Lei, Y., He, Z., Zi, Y., & Hu, Q. (2007). [Fault diagnosis of rotating machinery based on multiple ANFIS combination with GAs.](https://doi.org/10.1016/j.ymssp.2006.10.003) *Mechanical Systems and Signal Processing*, 21(5), 2280–2294.  
  ↳ Feature set reference and naming convention (T₁–T₁₁, F₁–F₁₂)
- FFI binding design research: see [`claudedocs/research_rust_ffi_bindings_20260309.md`](claudedocs/research_rust_ffi_bindings_20260309.md) for PyO3 vs alternatives, NAPI-RS performance notes, and FFT normalization deep-dives.

## License

MIT — see [LICENSE](LICENSE) for details.

---

## 中文摘要

**vibration-rms** 是針對旋轉機械振動分析開發的高性能函式庫。原始實作使用 NumPy/SciPy，現已改寫為 Rust，並提供 Python 與 Node.js 綁定。

**核心特性:**
- 加速度/速度/位移 RMS 計算（逆 FFT 與 PSD 雙方法）
- FFT 頻率/振幅/相位提取（符合 NumPy 正規化慣例）
- Lei 等人 (2007) 的 13 個時域與 12 個頻域振動特徵
- 雙重命名：支援英文名稱（如 `"rms"`, `"kurtosis"`）與論文編號（如 `"t4"`, `"f16"`）
- 選擇性特徵計算：只提取所需特徵以獲得最佳效能
- 跨平台：Windows 與 Linux，Rust/Python/Node.js 三層綁定

**黃金數據驗證:**  
本專案採用「黃金數據驗證」核心工程紀律：所有 Rust、Python、Node.js 層的輸出都與原始 NumPy/SciPy 參考實作在 `rtol=1e-6` 內一致。這保證了數值計算的精度，消除了 FFT 正規化、邊界條件與浮點精度等隱含差異。

**安裝:**
- Rust: `cargo add vibration-rms-core` (或 git 依賴)
- Python: `pip install vibration-rms` 或本地 `maturin develop`
- Node.js: `npm install @vibration-rms/native` 或本地編譯

詳見上方英文說明與 [金鑰檔案](README.md) 中的使用範例、API 參考、與 CI 配置。

授權條款：MIT — 見 [LICENSE](LICENSE)
