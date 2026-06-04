# rust-sox

**以 Rust 為核心、靈感源自 SoX 的音訊處理工具。**

`rust-sox` 是經典命令列音訊工具 [SoX](http://sox.sourceforge.net/)（Sound eXchange）
的 Rust 原生重製版。我們以安全性、可攜性與乾淨的程式碼優先於絕對的功能對等性——
從 WAV 處理核心出發，透過 [Symphonia](https://github.com/pdeljanov/Symphonia) 生態系
逐步擴充編解碼器支援。

> **狀態：** 積極開發中 — 尚未完全取代 SoX。  
> 第一階段里程碑聚焦於讓重製版真正實用且可擴充的核心功能。

---

## 為什麼選擇 rust-sox？

- **無 C 相依** — 純 Rust 音訊處理，建置、審計與貢獻門檻更低。
- **確定性管線** — 所有效果都作用於交錯 `f32` 取樣緩衝區，訊號路徑可預測且可測試。
- **SoX 相容 CLI** — 熟悉的 `input output effect...` 語法，無痛轉換。
- **串流處理** — 逐步處理大型檔案，無需將全部內容載入記憶體。
- **JSON 計畫** — 可重複、可腳本化的處理計畫。
- **平行批次** — 一個指令即可在多核心上執行批次轉換。
- **內建合成** — 透過 `synth` 產生波形。

---

## 快速開始

```powershell
# 查看音檔資訊
cargo run -- info input.wav

# 基本轉換與效果
cargo run -- convert input.wav out.wav --gain-db -3 --trim 0 10 --normalize

# SoX 風格傳統語法
cargo run -- input.wav out.wav gain -3 trim 0 10 norm rate 48000 stat

# 串接多檔
cargo run -- concat -o album.wav intro.wav body.wav outro.wav

# 混音
cargo run -- mix -o bed.wav voice.wav music.wav --normalize

# 音訊合成
cargo run -- synth tone.wav --duration 2 --freq 440 --waveform sine --fade 0.05

# 串流處理（低記憶體，適合大型檔案）
cargo run -- stream huge.wav huge-processed.wav --gain-db=-3 --fade-in 0.5 --fade-out 0.5

# 列出支援的編解碼器
cargo run -- formats

# 平行批次轉換
cargo run -- batch "samples\*.wav" --out-dir out --normalize --rate 48000

# 執行 JSON 計畫
cargo run -- run-plan plan.json
```

Release 建置產出 `target\release\rust-sox.exe`，同時會複製一份到 `dist\rust-sox.exe`。

---

## 功能特色

### 支援的效果

| 效果           | 語法                                   | 說明                           |
|----------------|----------------------------------------|--------------------------------|
| `gain`         | `gain <db>`                            | 增益/衰減（分貝）              |
| `norm`         | `norm [target-db]`                     | 峰值正規化（預設 −1 dBFS）     |
| `trim`         | `trim <start-sec> [duration-sec]`      | 從指定時間點裁剪，可選長度     |
| `fade`         | `fade <in-sec> [out-sec]`             | 線性淡入/淡出                  |
| `reverse`      | `reverse`                              | 反轉取樣                       |
| `speed`        | `speed <factor>`                       | 改變播放速度（會重新取樣）     |
| `pad`          | `pad <start-sec> [end-sec]`           | 開頭/結尾補靜音                |
| `silence`      | `silence <threshold-db> [min-sec]`     | 移除開頭的靜音段落             |
| `lowpass`      | `lowpass <hz>`                         | 低通濾波（一階）               |
| `highpass`     | `highpass <hz>`                        | 高通濾波（一階）               |
| `limiter`      | `limiter [threshold]`                  | 軟膝限幅器（預設 0.95）        |
| `rate`         | `rate <hz>`                            | 重新取樣至新的取樣率           |
| `channels`     | `channels <count>`                     | 聲道數量轉換（混音／取第一軌） |
| `stat`         | `stat`                                 | 列印音訊統計數據               |

### 合成波形

`synth` 子指令可從頭產生音訊：

- 波形：`sine`（正弦波）、`square`（方波）、`triangle`（三角波）、
  `saw`（鋸齒波）、`noise`（雜訊）、`silence`（靜音）
- 後處理：`--gain-db`、`--normalize`、`--fade`

### 編解碼支援

| 方向     | 支援格式                                                        |
|----------|----------------------------------------------------------------|
| **讀取** | WAV、FLAC、MP3、Ogg/Vorbis、Opus、AAC、ALAC、CAF、MKV/WebM、M4A |
| **寫入** | WAV（16-bit PCM）                                               |
| **串流** | WAV → WAV（僅限 gain、fade、limiter）                           |

WAV 以外的解碼由 [Symphonia](https://github.com/pdeljanov/Symphonia) 處理。
未來將會逐步加入更多的寫入後端。

### 串流處理

`stream` 子指令專為無法完全載入記憶體的大型檔案設計。它會逐步處理 WAV 取樣，
目前支援增益、淡入、淡出與限幅，過程中不會建立完整的 `AudioBuffer`。

### JSON 處理計畫

```json
{
  "mode": "convert",
  "inputs": ["input.wav"],
  "output": "out.wav",
  "effects": ["trim", "0", "10", "gain", "-3", "norm"],
  "stat_json": true
}
```

`mode` 可設定為 `convert`、`concat` 或 `mix`。

### 平行批次轉換

```powershell
cargo run -- batch "samples\*.wav" --out-dir out --normalize --rate 48000
```

符合 glob 條件的檔案會透過 `rayon` 在多核心上平行處理。
處理完成後會一併回報所有失敗項目，不會在首次錯誤就停止。

---

## 架構概覽

`rust-sox` 包含三條處理路徑：

1. **AudioBuffer 路徑** — 將輸入解碼為交錯 `f32` 取樣，套用 `EffectChain`，
   寫出 WAV。這是大多數指令的主要路徑。
2. **串流路徑** — 逐步處理 WAV 取樣，適合記憶體受限環境中的增益、淡入淡出與限幅。
3. **合成路徑** — 直接將內建波形產生至 `AudioBuffer`，再沿用同一條效果鏈與 WAV 寫入器。

詳細說明請見 [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md)。

---

## 專案結構

```
rust-sox/
├── Cargo.toml          # 套件中繼資料與相依套件
├── README.md           # 語言選擇頁面
├── src/
│   ├── main.rs         # CLI 分派、SoX 傳統語法相容、JSON 計畫執行器
│   ├── cli.rs          # Clap 參數定義
│   ├── audio.rs        # AudioBuffer、讀寫、重新取樣、聲道轉換
│   ├── effects.rs      # Effect 列舉與 EffectChain
│   ├── streaming.rs    # 串流 WAV 處理器
│   ├── synth.rs        # 波形合成
│   └── stats.rs        # 音訊統計數據
├── tests/
│   └── cli.rs          # 整合測試
├── examples/
│   ├── plan.convert.json
│   ├── plan.mix.json
│   ├── stream-large.ps1
│   └── synth-tone.ps1
├── docs/
│   ├── ARCHITECTURE.md
│   ├── KNOWLEDGE_MAP.md
│   └── ROADMAP.md
└── pages/
    ├── en/README.md    # 英文版說明
    └── zh-TW/README.md # 繁體中文版說明（就是你正在看的）
```

---

## 建置與安裝

```powershell
# Debug 建置
cargo build

# Release 建置
cargo build --release

# 執行檔位於 target/release/rust-sox.exe
# 同時會複製一份到 dist/rust-sox.exe
```

**前置需求：** [Rust](https://www.rust-lang.org/tools/install)（edition 2024，
最低 Rust 版本由相依套件決定）。

---

## 重製備註

指令風格的對照基準為上游專案 [chirlu/sox](https://github.com/chirlu/sox)。
現階段刻意從 WAV 開始，讓 Rust DSP 與 CLI 架構穩定後，
再逐步引入更多編解碼後端（FLAC、MP3、Ogg/Vorbis、Opus）以及平台音訊裝置 I/O。

---

## 授權條款

`rust-sox` 採用雙重授權：**LGPL-2.1-or-later** 或 **MIT**（任選其一）。
詳見 `LICENSE-LGPL` 與 `LICENSE-MIT` 檔案（或 `Cargo.toml` 的 license 欄位）。
