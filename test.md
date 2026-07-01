# rust-sox 測試改善計畫

> 基於 CBM 分析，現有測試僅 1 檔案 8 個整合測試，覆蓋率嚴重不足。

---

## 現狀分析

| 模組 | 行數 | 測試數 | 測試類型 | 風險 |
|------|------|--------|----------|------|
| `src/effects.rs` | 403 | 0 | — | 🔴 14 種效果無單元測試 |
| `src/audio.rs` | 242 | 0 | — | 🟡 WAV I/O、symphonia 解碼無獨立測試 |
| `src/main.rs` | 496 | 0 | — | 🟡 公用函式無測試 |
| `src/cli.rs` | 368 | 0 | — | 🟢 部分經整合測試覆蓋 |
| `src/streaming.rs` | 173 | 0 | — | 🟡 串流管線無獨立測試 |
| `src/synth.rs` | 79 | 0 | — | 🟢 部分經整合測試覆蓋 |
| `src/stats.rs` | 58 | 0 | — | 🟢 部分經整合測試覆蓋 |
| `tests/cli.rs` | 255 | 8 | 整合測試 | 🟡 hermetic 問題 |

---

## 測試計畫

### Phase 1 — 效果單元測試（最高優先）

**檔案**: `tests/effects.rs`（新增）

目標：為 `effects.rs` 中每種 Effect 變體建立最少 2 個測試案例。

```
tests/effects.rs
├── gain_db
│   ├── zero_db_identity          // gain 0dB 不改變樣本
│   ├── positive_db_amplifies     // +6dB → 振幅約 2x
│   └── negative_db_attenuates    // -6dB → 振幅約 0.5x
├── normalize
│   ├── normalize_to_target       // 驗證 peak 達到 target_db
│   └── silent_audio_noop         // 全靜音不 crash
├── trim
│   ├── trim_start                // 裁剪開頭
│   ├── trim_with_duration        // 裁剪指定區段
│   └── trim_beyond_end           // 超出範圍回傳空 buffer
├── fade
│   ├── fade_in_linear            // 驗證淡入曲線線性
│   ├── fade_out_linear           // 驗證淡出曲線線性
│   └── fade_both                 // 同時淡入淡出
├── reverse
│   ├── reverse_samples           // 反轉後與原始對稱
│   └── reverse_twice_identity    // reverse(reverse(x)) == x
├── speed
│   ├── speed_half                // 0.5x → 長度加倍
│   ├── speed_double              // 2.0x → 長度減半
│   └── speed_identity            // 1.0x → 不變
├── pad
│   ├── pad_start                 // 開頭填入靜音
│   ├── pad_end                   // 結尾填入靜音
│   └── pad_both                  // 兩端填入
├── trim_silence
│   ├── leading_silence           // 移除開頭靜音
│   ├── trailing_silence          // 移除結尾靜音
│   └── no_silence_noop           // 無靜音不移除
├── lowpass
│   ├── dc_preserved              // DC (0 Hz) 通過
│   ├── high_freq_attenuated      // 高頻被衰減
│   └── impulse_response          // 脈衝響應穩定
├── highpass
│   ├── dc_removed                // DC 被移除
│   ├── high_freq_preserved       // 高頻通過
│   └── impulse_response          // 脈衝響應穩定
├── limiter
│   ├── clip_at_threshold         // 超過 threshold 被裁剪
│   ├── below_threshold_unchanged // 未超過不變
│   └── threshold_zero            // threshold=0 → 全靜音
├── rate (resample)
│   ├── half_rate                 // 48k→24k，frame 減半
│   ├── double_rate               // 24k→48k，frame 加倍
│   └── same_rate_noop            // 相同 rate 不做改變
├── convert_channels
│   ├── stereo_to_mono            // 2ch→1ch 平均
│   ├── mono_to_stereo            // 1ch→2ch 重複
│   ├── stereo_to_51              // 2ch→6ch 映射
│   └── same_channels_noop        // 相同 channel 不變
└── effect_chain
    ├── multiple_effects_order    // 驗證 chain 順序正確
    └── empty_chain_noop          // 空 chain 不改變音訊
```

### Phase 2 — 音訊 I/O 單元測試

**檔案**: `tests/audio.rs`（新增）

```
tests/audio.rs
├── read_wav
│   ├── reads_float_wav           // 讀取 float WAV
│   ├── reads_int16_wav           // 讀取 16-bit WAV
│   ├── reads_int24_wav           // 讀取 24-bit WAV
│   ├── reads_int32_wav           // 讀取 32-bit WAV
│   ├── reads_int8_wav            // 讀取 8-bit WAV
│   └── rejects_invalid_wav       // 無效檔案回傳錯誤
├── write_wav
│   ├── roundtrip_16bit           // 寫入後讀回確認一致
│   └── creates_parent_dir        // 自動建立父目錄
├── frames_and_duration
│   ├── frame_count               // frame 計算正確
│   └── duration                  // 時長計算正確
└── peak
    ├── peak_value                // peak 計算正確
    └── silent_peak_zero          // 靜音 peak = 0
```

### Phase 3 — 串流模組測試

**檔案**: `tests/streaming.rs`（新增）

```
tests/streaming.rs
├── stream_gain
│   └── gain_applied              // 驗證增益正確
├── stream_fade
│   ├── fade_in_applied           // 驗證淡入曲線
│   └── fade_out_applied          // 驗證淡出曲線
├── stream_limiter
│   ├── limiter_clips             // 驗證 limiter 裁剪
│   └── limiter_noop_below        // 低於 threshold 不變
└── stream_pipeline
    ├── combined_effects          // 組合 gain+fade+limiter
    └── large_file                // 驗證大檔案串流不 OOM
```

### Phase 4 — 整合測試強化

**檔案**: `tests/cli.rs`（擴充）

```
現有 8 個測試 → 擴充至 15+ 個

新增：
├── edge_cases
│   ├── empty_file_error           // 空檔案 → 合理錯誤
│   ├── nonexistent_file_error     // 不存在檔案 → 合理錯誤
│   ├── unsupported_format_error   // 不支援格式 → 合理錯誤
│   └── zero_length_synth          // synth 0 秒 → 空檔案
├── cli_parsing
│   ├── legacy_style               // SoX 風格命令列
│   └── missing_required_arg       // 缺少必要參數 → 錯誤
├── batch
│   ├── batch_multiple_files       // 批次處理多檔案
│   └── batch_glob_pattern         // glob 展開
├── run_plan
│   ├── plan_concat                // JSON plan concat
│   └── plan_mix                   // JSON plan mix
└── stream
    └── stream_ffmpeg_input        // stream 非 WAV 格式（若支援）
```

### Phase 5 — 屬性測試（Property-based Testing）

**檔案**: `tests/property.rs`（新增）

使用 `proptest` 進行隨機化測試：

```
tests/property.rs
├── gain_identity                  // gain(0dB, x) == x
├── reverse_involution            // reverse(reverse(x)) == x
├── channels_roundtrip            // 1ch→2ch→1ch == x
├── rate_same_noop                // rate(same_rate, x) == x
├── chain_does_not_panic           // 任意效果組合不 panic
├── constant_amplitude_preserved   // 恆定振幅訊號通過線性效果不變
└── silence_remains_silent         // 全靜音通過任意 chain 保持靜音
```

### Phase 6 — 回歸測試

**檔案**: `tests/regression.rs`（新增）

```
tests/regression.rs
├── issue_xxx                      // 追蹤已修復 bug 的回歸測試
└── clip_detection                 // clipping 偵測正確性
```

---

## 測試基礎設施

### Test Helper 模組

建立 `tests/common/mod.rs` 共享測試工具：

```rust
// 目標 API
pub fn create_test_buffer(frames: usize, channels: u16, sample_rate: u32) -> AudioBuffer
pub fn create_test_wav(path: &Path, frames: usize, channels: u16, amplitude: f32)
pub fn create_silent_buffer(frames: usize, channels: u16, sample_rate: u32) -> AudioBuffer
pub fn create_sine_buffer(frames: usize, channels: u16, freq: f32, sample_rate: u32) -> AudioBuffer
pub fn create_test_dir() -> TempDir
pub fn assert_buffer_eq(actual: &AudioBuffer, expected: &AudioBuffer, tolerance: f32)
pub fn assert_peak(actual: &AudioBuffer, expected: f32, tolerance: f32)
```

### 測試 WAV 檔案嵌入

使用 `include_bytes!` 嵌入小型測試 WAV 檔案：

```rust
const TEST_WAV_16BIT_MONO: &[u8] = include_bytes!("fixtures/sine-440-1ch-16bit.wav");
const TEST_WAV_16BIT_STEREO: &[u8] = include_bytes!("fixtures/sine-440-2ch-16bit.wav");
```

可透過 `synth` 命令產生這些檔案並 commit 到 `tests/fixtures/`。

---

## 測試執行計畫

| Phase | 檔案 | 測試數 (預估) | 相依性 | 優先度 |
|-------|------|---------------|--------|--------|
| 1 | `tests/effects.rs` | ~45 | 無 | P0 |
| 2 | `tests/audio.rs` | ~10 | 無 | P1 |
| 3 | `tests/streaming.rs` | ~6 | 無 | P1 |
| 4 | `tests/cli.rs` | +7 | cargo build | P1 |
| 5 | `tests/property.rs` | ~7 | proptest crate | P2 |
| 6 | `tests/regression.rs` | 持續增加 | — | P2 |

---

## 覆蓋率目標

| 階段 | 行覆蓋率目標 | 方法 |
|------|-------------|------|
| 當前 | ~20% | 僅 8 個整合測試 |
| Phase 1 完成後 | ~50% | 主要效果有單元測試 |
| Phase 2-3 完成後 | ~65% | I/O + 串流覆蓋 |
| Phase 4 完成後 | ~75% | CLI edge cases 涵蓋 |
| Phase 5 完成後 | ~80%+ | 屬性測試覆蓋邊界 |

---

## 優先執行建議

1. **立即**: Phase 1 — 效果單元測試（effects.rs 403 行零測試是最嚴重的漏洞）
2. **緊接**: Phase 4 強化 — 整合測試 edge cases
3. **次之**: Phase 2 + Phase 3 — I/O 和串流測試
4. **長期**: Phase 5 + Phase 6 — 屬性測試和回歸測試

```bash
# 執行所有測試
cargo test

# 執行特定測試
cargo test --test effects
cargo test --test cli -- converts_with_sox_style_effects

# 覆蓋率報告（需安裝 cargo-llvm-cov）
cargo llvm-cov --html
```
