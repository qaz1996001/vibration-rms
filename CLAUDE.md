# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**專案名稱**: a003_core_view (`core-view`)
**用途**: 旋轉機械振動分析與特徵擷取函式庫 — 將原本的 Python (numpy/scipy) 實作改寫為 Rust，並同時提供 Python 與 Node.js 綁定
**技術棧**: Rust (workspace) + PyO3/maturin (Python 綁定) + NAPI-RS (Node.js 綁定)；保留 numpy/scipy 參考實作作為 golden data 來源

**Key Characteristics**:
- 三層 workspace 架構：`core` (純 Rust 邏輯) → `bindings-python` / `bindings-node` (FFI 薄殼)
- **Golden data 驗證為核心開發模式**：所有 Rust/binding 結果都必須與原始 Python 輸出在 `rtol=1e-6` 內一致
- 純函式庫設計，無 CLI、無服務、無 UI；只暴露純函式 API
- 對外 API 與原始 `vibration.py` / `feature.py` 一一對應，保留參考文獻 (Lei et al., 2007) 的命名 (`t1`–`t11`, `f12`–`f23`)
- 平台目標：Windows 與 Linux

---

## Development Commands

### 環境設定

**Rust core / bindings**:
```bash
# 從 workspace 根目錄
cd core_view_rs

# 編譯整個 workspace（含 core 與兩個 bindings）
cargo build

# 只編譯核心 crate
cargo build -p core-view-core
```

**Python bindings (PyO3 + maturin)**:
```bash
cd core_view_rs/bindings-python

# 使用 uv 管理虛擬環境（已有 .venv 與 uv.lock）
uv sync

# 開發模式安裝（編譯 Rust + 安裝到 .venv 的 site-packages）
uv run maturin develop

# Release 模式
uv run maturin develop --release
```

**Node.js bindings (NAPI-RS)**:
```bash
cd core_view_rs/bindings-node

npm install
npm run build         # release
npm run build-debug   # debug
```

### Git 工作流
```bash
# 提交格式（建議）
git commit -m "type(scope): description"
# scope 範例: core, py-bindings, node-bindings, scripts, docs
```

---

## Verification Commands

**🎯 核心原則**: 任何修改 `core/` 或 bindings 的程式碼後，**必須**跑 golden cross-validation。Rust 與 Python 結果若有差異一定是 port bug。

### 必執行驗證

#### Rust (`core_view_rs/`)
```bash
cd core_view_rs

# 1. 型別/借用檢查（最快）
cargo check

# 2. 格式化（rustup 內建，無需安裝）
cargo fmt --all                  # 自動修正
cargo fmt --all -- --check       # 只檢查不寫入（CI 用）

# 3. Lint（rustup 內建）
cargo clippy --all-targets --all-features -- -D warnings

# 4. 單元測試 + golden cross-validation
cargo test                       # 全 workspace
cargo test -p core-view-core     # 只跑 core
# 重要：core/tests/golden_cross_validation.rs 會讀取 scripts/golden_data.json
```

#### Python — 參考實作 + bindings
```bash
# 5. 重新產生 golden data（只在修改原始 Python 參考實作時）
python core_view_rs/scripts/generate_golden_data.py

# 6. Python format + lint（用 ruff，整個專案根目錄）
#    覆蓋範圍：vibration.py, feature.py, scripts/, bindings-python/
ruff format .                    # 自動修正
ruff format --check .            # 只檢查不寫入（CI 用）
ruff check .                     # lint
ruff check --fix .               # lint + 自動修正

# 7. Python bindings 編譯 + 交叉驗證
cd core_view_rs/bindings-python
uv run maturin develop
uv run python test_bindings.py   # 或 uv run pytest test_bindings.py
```

> ⚠️ **首次使用 ruff**：專案目前未安裝 ruff。
> 安裝方式（擇一）：
> - 全域：`uv tool install ruff`
> - 只在 bindings-python 的 venv：`cd core_view_rs/bindings-python && uv add --dev ruff`

#### Node.js (`core_view_rs/bindings-node/`)
```bash
cd core_view_rs/bindings-node

# 8. Format + lint（用 biome）
#    注意：必須排除 napi-rs 自動產生的 index.js / index.d.ts
npx @biomejs/biome format --write .         # 自動修正
npx @biomejs/biome format .                 # 只檢查不寫入
npx @biomejs/biome check .                  # format + lint 一次跑
npx @biomejs/biome check --write .          # 全部自動修正

# 9. Bindings 編譯 + 交叉驗證
npm run build
node test_bindings.mjs
```

> ⚠️ **首次使用 biome**：專案目前未安裝 biome。
> 安裝方式：`cd core_view_rs/bindings-node && npm install --save-dev @biomejs/biome && npx @biomejs/biome init`
> 然後在 `biome.json` 的 `files.ignore` 加入 `["index.js", "index.d.ts", "target/**", "node_modules/**"]`

### 驗證清單
修改程式碼後，依序執行：

**Rust 端**：
- [ ] `cargo fmt --all -- --check` 通過
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` 無警告
- [ ] `cargo check` 通過
- [ ] `cargo test` 全部通過（含 `golden_cross_validation`）

**Python 端**（若改了 `*.py`）：
- [ ] `ruff format --check .` 通過
- [ ] `ruff check .` 無警告
- [ ] 若改了 `vibration.py` / `feature.py` → 重新跑 `scripts/generate_golden_data.py`
- [ ] 若改了 `bindings-python` → `maturin develop` + `python test_bindings.py` 全 PASS

**Node.js 端**（若改了 `bindings-node/`）：
- [ ] `npx @biomejs/biome check .` 通過
- [ ] `npm run build` + `node test_bindings.mjs` 全 PASS

### 驗證失敗處理
```
golden test 失敗 → 對照 Python 參考實作 → 找出公式/正規化/邊界差異
                → 修 Rust 端（不要修 golden data）→ 重新驗證
```

---

## Anti-Patterns & Learnings

**📝 活文件規則**: Claude 犯錯時立即新增記錄

### 錯誤行為紀錄（持續更新）

| 日期 | ❌ 錯誤行為 | ✅ 正確做法 |
|------|------------|------------|
| 2026-03-12 | 在 Node.js v22 上安裝 `windows-build-tools` 來編譯原生模組 | 改用 napi-rs (Rust 編譯)，根本不需要 C++ Build Tools |
| 2026-03-12 | 用 `npm config set msvs_version` 指定 VS 版本 | 新版 npm 已移除此選項；napi-rs 專案不需要設定，必要時改用環境變數 `GYP_MSVS_VERSION=2022` |

### 禁止事項

- ❌ 修 golden data 來「讓測試通過」 — golden data 是 ground truth，差異一律修 Rust 端
- ❌ 為了填滿模板而虛構不存在的架構（例如假的例外階層、不存在的 UI 系統）
- ❌ 在不對齊原始 Python `numpy.fft` 正規化慣例的情況下使用 `realfft` 的原始輸出（`realfft` 的 inverse 未正規化，必須除以 `n`）
- ❌ 在 Rust 用 `ndarray` 的 boolean mask 風格寫 bandpass — `ndarray` 不支援，請用 `fft_utils::bandpass_range` / `highpass_start` 提供的索引範圍
- ❌ 跳過品質檢查
- ❌ 未經用戶確認就實作理解模糊的需求

### Numpy ↔ Rust FFT 正規化對照

```rust
// ❌ 錯誤：直接用 realfft inverse 的輸出
let mut output = ifft.make_output_vec();
ifft.process(&mut spec, &mut output).unwrap();
return output;   // numpy irfft 期望已除以 n，但這裡沒除

// ✅ 正確：除以 n 對齊 numpy 慣例
for v in output.iter_mut() { *v *= 1.0 / n as f64; }
```

```rust
// ❌ 錯誤：Vibration 計算中遺漏 freq_ticks[0] = 1.0 的避除零
// 這會導致 vel_rms / dis_pkpk 出現 NaN

// ✅ 正確：對照 vibration.py 的 `freq_ticks[0] = 1.` 並補回 DC 為 0
```

---

## Architecture

### 架構概覽

```
┌─────────────────────────────────────────────────────────┐
│  Reference Implementation (root)                        │
│   vibration.py, feature.py                              │  ← Python/numpy/scipy 原始實作
│   scripts/generate_golden_data.py                       │  ← 產出 golden_data.json
└────────────────────────┬────────────────────────────────┘
                         │ golden data (rtol=1e-6)
                         ▼
┌─────────────────────────────────────────────────────────┐
│  Rust workspace (core_view_rs/)                         │
│                                                         │
│  ┌──────────────────────────────────────────────┐       │
│  │  core/ — core-view-core                      │       │  ← 純 Rust 邏輯
│  │   src/vibration.rs    acceleration_rms[2],   │       │
│  │                       get_fft_abs_data        │       │
│  │   src/feature.rs      time/freq features      │       │
│  │   src/fft_utils.rs    rfftfreq, bandpass     │       │
│  │   tests/golden_cross_validation.rs            │       │
│  └──────────────────────────────────────────────┘       │
│         ▲                              ▲                │
│         │                              │                │
│  ┌──────┴───────────┐         ┌────────┴──────────┐     │
│  │ bindings-python/ │         │ bindings-node/    │     │
│  │  PyO3 + maturin  │         │  NAPI-RS          │     │
│  │  → core_view.pyd │         │  → index.node     │     │
│  └──────────────────┘         └───────────────────┘     │
└─────────────────────────────────────────────────────────┘
```

### 目錄結構
```
a003_core_view/
├── vibration.py                # Python 原始實作（保留為 reference）
├── feature.py                  # Python 原始實作（保留為 reference）
├── claudedocs/                 # 設計研究文件
│   └── research_rust_ffi_bindings_20260309.md
└── core_view_rs/               # Rust workspace
    ├── Cargo.toml              # workspace = ["core", "bindings-python", "bindings-node"]
    ├── core/                   # core-view-core (pure Rust lib)
    ├── bindings-python/        # core-view-python (PyO3 cdylib)
    ├── bindings-node/          # @core-view/native (NAPI-RS cdylib)
    └── scripts/                # golden data 產生器
```

### 核心模組

| 模組 | 用途 |
|------|------|
| `core/src/vibration.rs` | `acceleration_rms`, `acceleration_rms2`, `get_fft_abs_data` — 振動量轉換 (acc/vel/dis) |
| `core/src/feature.rs` | `time_domain_features`, `freq_domain_features` 與 `*_select` 變體；`TdFeature` / `FdFeature` enum |
| `core/src/fft_utils.rs` | `rfftfreq`, `bandpass_range`, `highpass_start`, `f64_max`, `f64_min` — 對齊 numpy 行為 |
| `bindings-python/src/lib.rs` | PyO3 包裝，模組名 `core_view`；用 `numpy::PyArray1` 達到 zero-copy |
| `bindings-node/src/lib.rs` | NAPI-RS 包裝，函式以 camelCase 暴露 (`accelerationRms`, …) |
| `scripts/generate_golden_data.py` | 用原始 Python 實作產出所有 reference 結果 → `golden_data.json` |

---

## Error Handling

> 本專案是純計算函式庫，無自訂例外階層。

- **Rust core**：函式直接回傳數值/陣列。內部 `unwrap()` 僅用於 FFT planner 不可能失敗的路徑。
- **Python bindings**：選用未知的 feature 名稱時拋 `ValueError`（見 `parse_td_features` / `parse_fd_features`）。
- **Node.js bindings**：napi-rs 自動把 panic 轉成 JS `Error`。

### 常見錯誤與解決方案

| 錯誤 | 原因 | 解決 |
|------|------|------|
| `golden_data.json not found` | 跑 cargo test 前沒先產生 golden | `python scripts/generate_golden_data.py` |
| `ValueError: Unknown time-domain feature: 'xxx'` | 傳給 `time_domain_features` 的字串名稱不在 enum 中 | 用 `t1`–`t11` 或 `rms` / `kurtosis` 等英文名 |
| `Unknown frequency-domain feature: 'xxx'` | 同上，但 frequency 端 | 用 `f12`–`f23` 或 `centroid` / `mean` 等英文名 |
| `dis_pkpk` 出現 NaN | 沒處理 `freq_ticks[0] = 0`（DC 分量除零） | 對照 `vibration.py:18` 補上 `freq_ticks[0] = 1.0` 與 DC 歸零 |

---

## Debug Commands

```bash
# Rust：詳細編譯錯誤 + backtrace
cd core_view_rs
RUST_BACKTRACE=full cargo test -p core-view-core -- --nocapture

# 跑單一 golden 測試
cargo test -p core-view-core --test golden_cross_validation acceleration_rms -- --nocapture

# Python bindings：maturin 詳細輸出
cd core_view_rs/bindings-python
uv run maturin develop --verbose

# 確認 .venv 中已安裝 core_view
uv run python -c "import core_view; print(core_view.__file__)"

# Node.js bindings：手動載入確認 .node 檔
cd core_view_rs/bindings-node
node -e "console.log(Object.keys(require('./index.js')))"
```

---

## Common Patterns

### 模式 1：Golden Data Cross-Validation

**這是本專案最核心的開發模式**。每個新函式都必須跟原始 Python 輸出對照。

**連動檔案**：

| 檔案 | 說明 |
|------|------|
| `vibration.py` / `feature.py` | Python ground truth — **修改時必須重新生成 golden** |
| `scripts/generate_golden_data.py` | 呼叫 Python 實作、把所有結果序列化成 JSON |
| `core_view_rs/scripts/golden_data.json` | 序列化後的 reference（含 `signal`, `sampling_rate`, 各函式輸出） |
| `core_view_rs/core/tests/golden_cross_validation.rs` | Rust 端讀 JSON、對照 `core` 輸出 |
| `core_view_rs/bindings-python/test_bindings.py` | Python 端載入 `core_view` 模組、對照同一份 JSON |
| `core_view_rs/bindings-node/test_bindings.mjs` | Node 端載入 `index.js`、對照同一份 JSON |

**標準工作流程**：
```
1. 在 vibration.py / feature.py 加新函式（或修現有的）
2. python scripts/generate_golden_data.py   # 更新 golden
3. 在 core/src/*.rs 實作 Rust 版本
4. cargo test                                # golden_cross_validation
5. 在 bindings-python/src/lib.rs + bindings-node/src/lib.rs 加 wrapper
6. 跑兩個 binding 的 test_bindings.{py,mjs}
```

### 模式 2：Feature 命名雙軌制

`TdFeature` / `FdFeature` 都同時接受**英文名稱**與**論文編號**：

```python
# 兩種寫法等價（Lei et al. 2007 編號）
core_view.time_domain_features(signal, ['rms', 'kurtosis'])
core_view.time_domain_features(signal, ['t4',  't7'])

core_view.freq_domain_features(pl, f, ['centroid'])
core_view.freq_domain_features(pl, f, ['f16'])
```

對應表見 `core/src/feature.rs` 的 `from_name` 實作。

### 模式 3：Numpy zero-copy via PyO3

```rust
// ✅ 正確：用 PyReadonlyArray1 接收，不複製
fn time_domain_features<'py>(
    py: Python<'py>,
    signal: PyReadonlyArray1<'py, f64>,
    ...
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let slice = signal.as_slice().unwrap();   // zero-copy view
    ...
}

// ❌ 避免：用 Vec<f64> 會強制 copy 整個陣列
fn time_domain_features(signal: Vec<f64>) { ... }
```

注意：Node.js 端 (NAPI-RS) 目前用 `Vec<f64>` 而非 `Float64Array`，是已知的可優化點。

---

## Key Files Reference

| 類別 | 檔案 |
|------|------|
| **Workspace 設定** | `core_view_rs/Cargo.toml` |
| **Core crate** | `core_view_rs/core/Cargo.toml`, `core/src/{lib,vibration,feature,fft_utils}.rs` |
| **Python bindings** | `core_view_rs/bindings-python/{Cargo.toml,pyproject.toml,core_view.pyi}`, `bindings-python/src/lib.rs` |
| **Node bindings** | `core_view_rs/bindings-node/{Cargo.toml,package.json}`, `bindings-node/src/lib.rs` |
| **Golden data 產生** | `core_view_rs/scripts/generate_golden_data.py` |
| **Golden data** | `core_view_rs/scripts/golden_data.json` |
| **Cross-validation 測試** | `core/tests/golden_cross_validation.rs`, `bindings-python/test_bindings.py`, `bindings-node/test_bindings.mjs` |
| **Python 參考實作** | `vibration.py`, `feature.py`（root）|

---

## References

| 文件 | 說明 |
|------|------|
| `claudedocs/research_rust_ffi_bindings_20260309.md` | Rust FFI 綁定技術選型研究（PyO3 vs alternatives, NAPI-RS vs alternatives, ndarray gotchas） |
| `core_view_rs/bindings-python/core_view.pyi` | Python bindings 的型別存根；對外 API 的權威定義 |
| Lei, Y., He, Z., Zi, Y., & Hu, Q. (2007) | *Fault diagnosis of rotating machinery based on multiple ANFIS combination with GAs.* Mechanical Systems and Signal Processing, 21(5), 2280–2294. — `feature.py` 的特徵編號來源 |

---

**最後更新**: 2026-04-07
