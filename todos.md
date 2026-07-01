# rust-sox 優化改善清單

> 基於 CBM 架構分析與原始碼審查結果，按優先級排列。

---

## P0 — 關鍵缺陷（必須修復）

### [P0] 重複的 `seconds_to_frames` 函式

- **位置**: `src/effects.rs:377` (回傳 `usize`)、`src/streaming.rs:171` (回傳 `u32`)
- **問題**: 相同邏輯複製兩次，回傳型別不同，若未來修改一處會忘記另一處。
- **建議**: 移至共享模組 `src/util.rs`，統一使用 `usize`。
- **驗證**: `cargo build` + `cargo test` 通過。

### [P0] `main.rs` 過大（496 行）

- **問題**: `main.rs` 包含 `parse_effects`、`expand_inputs`、`concat_inputs`、`mix_inputs`、`sample_at`、`ensure_same_format`、`write_output`、`convert_one` 等大量公用函式，職責不清。
- **建議**: 拆分至專用模組：
  - `src/parse.rs`: `parse_effects`, `parse_next_f32/u32/u16`
  - `src/io.rs`: `expand_inputs`, `output_for_batch`, `convert_one`, `write_output`
  - `src/mix.rs`: `concat_inputs`, `mix_inputs`, `sample_at`, `ensure_same_format`
- **驗證**: `cargo build` + `cargo test` 通過，功能不變。

### [P0]  WAV 位元深度處理重複

- **位置**: `src/audio.rs:190-241` (`read_int_samples`) 與 `src/streaming.rs:48-114` (match on `sample_format`)
- **問題**: 8/16/24/32-bit 整數轉 `f32` 的邏輯在不同模組重複實作。
- **建議**: 提取為共用函式，例如 `wav_to_f32(reader, bits) -> Vec<f32>` 放在 `util.rs` 或 `audio.rs`。
- **驗證**: `cargo test` 所有現有測試通過。

### [P0] 三倍重複的 `effect_chain()` 模式

- **位置**: `src/cli.rs` — `ConvertArgs::effect_chain()` (L243)、`BatchArgs::effect_chain()` (L303)、`SynthArgs::effect_chain()` (L336)
- **問題**: 三個 struct 各自實作幾乎相同的 effect chain 建構邏輯。
- **建議**: 引入 Builder 模式或巨集：
  ```rust
  // 範例
  EffectChainBuilder::default()
      .gain_db(self.gain_db)
      .normalize(self.normalize, self.normalize_db)
      .trim(&self.trim)
      .fade(&self.fade)
      .reverse(self.reverse)
      .speed(self.speed)
      .lowpass(self.lowpass)
      .highpass(self.highpass)
      .limiter(self.limiter)
      .pad(&self.pad)
      .silence(&self.silence)
      .rate(self.rate)
      .channels(self.channels)
      .build()
  ```
- **驗證**: `cargo test` 通過，行為一致。

---

## P1 — 重要改善

### [P1] 新增效果單元測試（effects.rs 403 行零測試）

- **位置**: `src/effects.rs`
- **問題**: 403 行 DSP 程式碼完全沒有單元測試。
- **建議**: 為每個 Effect 變體新增單元測試：
  - `gain_db`: 驗證 0dB 不變、+6dB 振幅加倍、-6dB 減半
  - `normalize`: 驗證 target peak 正確
  - `trim`: 驗證 start/duration 邊界
  - `fade`: 驗證線性淡入淡出曲線
  - `reverse`: 驗證樣本反轉
  - `speed`: 驗證速度倍率正確
  - `lowpass/highpass`: 驗證 DC 回應與高頻衰減
  - `limiter`: 驗證 threshold 裁剪
  - `convert_channels`: mono↔stereo 轉換
- **驗證**: `cargo test` 新增測試通過。

### [P1] Mix 行為修正（平均 → 疊加）

- **位置**: `src/main.rs:286-289`
- **問題**: Mix 將所有輸入相加後除以輸入數量，這不是標準混音行為。標準作法是疊加後可選 limiter/clipping guard。
- **建議**: 去除平均除法，改為疊加後提供 `--normalize` 或自動 limiter 防止 clipping。
- **驗證**: 建立已知 test vectors 驗證混音結果。

### [P1] 僅輸出 16-bit WAV — 支援更多格式

- **位置**: `src/audio.rs:68-90` (`write_wav`)
- **問題**: 只能輸出 16-bit 整數 PCM WAV。
- **建議**: 新增 `--bits` (8/16/24/32) 和 `--format` (wav/f32) 參數。提供 float WAV 輸出選項。
- **驗證**: `cargo test` + 手動驗證不同 bit depth 輸出。

### [P1] Stream 模式僅支援 WAV

- **位置**: `src/streaming.rs:5-118`
- **問題**: streaming 路徑只處理 WAV，無法串流 FLAC/MP3 等格式。
- **建議**: 使用 symphonia 實作通用串流解碼 → 處理 → 編碼管線。
- **驗證**: 串流處理 1GB+ FLAC 檔案不 OOM。

### [P1] 無最大檔案保護（OOM 風險）

- **問題**: `AudioBuffer::read` 無大小上限，超大檔案會 OOM。
- **建議**: 可選 `--max-size` 參數；預設對 >500MB 檔案提出警告。
- **驗證**: 嘗試讀取超大檔案時獲得合理錯誤訊息。

---

## P2 — 品質與 DX

### [P2] 新增 `rust-toolchain.toml`

- **問題**: Rust edition 2024 需要 Rust 1.85+，但無鎖定 toolchain 版本。
- **建議**: 新增 `rust-toolchain.toml` 鎖定 `1.94.1` (目前已安裝)。
- **驗證**: `cargo build` 在不同環境使用正確的 toolchain。

### [P2] 降低 symphonia 編譯負擔

- **位置**: `Cargo.toml:16` — `features = ["all"]`
- **問題**: 引入所有 codec decoder，增加編譯時間和 binary 體積。
- **建議**: 僅啟用需要的 features: `["flac", "mp3", "vorbis", "opus"]`，或設為 feature gate。
- **驗證**: `cargo build` 成功，支援的格式仍然可讀。

### [P2] 新增 CI 配置

- **問題**: 無 GitHub Actions 或其他 CI 設定。
- **建議**: 新增 `.github/workflows/ci.yml`：
  - `cargo build` (Windows + Ubuntu + macOS)
  - `cargo test`
  - `cargo clippy`
  - `cargo fmt --check`
- **驗證**: PR 自動觸發 CI。

### [P2] Hermetic FLAC 測試

- **位置**: `tests/cli.rs:125-160`
- **問題**: FLAC 解碼測試依賴外部 ffmpeg，非 hermetic。
- **建議**: 使用 `include_bytes!` 嵌入一個小型 FLAC 測試檔案。
- **驗證**: `cargo test` 不需要 ffmpeg 即可通過。

### [P2] Linear interpolation 提升品質

- **位置**: `src/effects.rs:170-202` (speed)、`src/effects.rs:302-338` (rate)
- **問題**: 線性插值在 >2x 變速時產生明顯混疊失真。
- **建議**: 實作 windowed Sinc 或至少 cubic Hermite 插值。
- **驗證**: 測試已知頻率的頻譜分析（FFT 比較）。

---

## P3 — 新增功能

### [P3] 遺漏的 SoX 效果

- **建議新增**:
  - `compand` — 動態範圍壓縮
  - `echo` / `delay` — 回音 / 延遲
  - `reverb` — 殘響
  - `phaser` / `flanger` — 相位 / 鑲邊效果
  - `tempo` — 不變調速
  - `pitch` — 音高偏移（不變速度）
- **驗證**: 與 SoX 輸出進行 golden 測試比對。

### [P3] `--guard` Clipping 保護

- **問題**: SoX 有 `--guard` 參數可在處理鏈末端插入 soft-knee limiter。
- **建議**: 新增全域 `--guard` 參數。
- **驗證**: 處理已知 clipping 案例時輸出不超過 0dBFS。

### [P3] 屬性測試（Property-based Testing）

- **建議**: 使用 `proptest` 或 `quickcheck` 對 DSP 進行屬性測試：
  - `gain_db(0) = identity`
  - `reverse(reverse(x)) = x`
  - `channels(1) -> channels(2)` 保持能量
  - 所有 chain 組合不 panic
- **驗證**: `cargo test` 包含 1000+ 隨機案例。

---

## 優先級摘要

| 優先級 | 數量 | 類型 |
|--------|------|------|
| P0 | 4 | 重複程式碼、職責拆分、bug |
| P1 | 5 | 測試覆蓋、行為修正、功能增強 |
| P2 | 5 | 開發體驗、CI、編譯優化 |
| P3 | 3 | 新功能、進階測試 |

---

## 風險說明

- **P0 項目** 為目前最緊迫的技術債，建議在下一輪開發優先處理。
- **main.rs 拆分** 需注意所有 `pub(crate)` 函式的引用路徑更新。
- **Mix 行為變更** 可能影響現有使用者的混音結果，需在 changelog 標註 breaking change。
- **Symphonia feature 調整** 需確認常用格式不受影響。
