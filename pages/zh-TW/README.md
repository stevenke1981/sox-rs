# sox — Rust 原生音訊處理工具

**純 Rust · 無 C 相依 · SoX 相容 CLI**

`sox`（rust-sox）是經典命令列音訊工具 [SoX](http://sox.sourceforge.net/)（Sound eXchange）
的 Rust 原生重製版。以安全性、可攜性與乾淨的程式碼優先，
從 WAV 處理核心出發，透過 [Symphonia](https://github.com/pdeljanov/Symphonia) 生態系
逐步擴充編解碼器支援。

> **狀態：** 第一階段里程碑已完成。  
> 已可應付日常音訊處理需求。

---

## 為什麼選擇 sox？

- **無 C 相依** — 純 Rust 音訊處理，建置、審計與貢獻門檻更低。
- **確定性管線** — 所有效果都作用於交錯 `f32` 取樣緩衝區，訊號路徑可預測且可測試。
- **SoX 相容 CLI** — 熟悉的 `input output effect...` 語法，也支援子命令模式。
- **串流處理** — 逐步處理大型檔案，無需全部載入記憶體。
- **JSON 計畫** — 可重複、可腳本化的處理計畫。
- **平行批次** — 一個指令在多核心上批次轉換。
- **內建合成** — 透過 `synth` 產生波形。

---

## 快速開始

```bash
# 透過 cargo 安裝
cargo install rust-sox

# 或從原始碼建置
git clone https://github.com/stevenke1981/sox-rs.git
cd sox-rs
cargo build --release
./target/release/sox --help
```

```bash
# 查看音檔資訊
sox info input.wav
sox info --json input.wav

# 基本轉換與效果
sox convert input.wav out.wav --gain-db -3 --trim 0 10 --normalize

# SoX 風格傳統語法
sox input.wav out.wav gain -3 trim 0 10 norm rate 48000 stat

# 串接多檔
sox concat -o album.wav intro.wav body.wav outro.wav

# 混音
sox mix -o bed.wav voice.wav music.wav --normalize

# 音訊合成
sox synth tone.wav --duration 2 --freq 440 --waveform sine --fade 0.05

# 串流處理（低記憶體，適合大型檔案）
sox stream huge.wav processed.wav --gain-db=-3 --fade-in 0.5 --fade-out 0.5

# 列出支援的編解碼器
sox formats

# 平行批次轉換
sox batch "samples/*.wav" --out-dir out --normalize --rate 48000

# 執行 JSON 計畫
sox run-plan plan.json
```

---

## 安裝方式

### 預編譯二進位檔

從 [GitHub Releases](https://github.com/stevenke1981/sox-rs/releases) 下載：

| 平台 | 套件 | 執行檔 |
|------|------|--------|
| Windows x86_64 | `sox-<version>-win64.zip` | `sox.exe` |
| Linux x86_64 | `sox-<version>-linux.tar.gz` | `sox` |
| macOS x86_64 | `sox-<version>-macos.tar.gz` | `sox` |

### 從原始碼建置

```bash
cargo install rust-sox
# 或
cargo build --release
# 執行檔位於 target/release/sox (或 sox.exe)
```

---

## 支援的效果

| 效果 | 語法 | 說明 |
|------|------|------|
| `gain` | `gain <db>` | 增益/衰減（分貝） |
| `norm` | `norm [target-db]` | 峰值正規化（預設 −1 dBFS） |
| `trim` | `trim <start-sec> [duration-sec]` | 從指定時間點裁剪 |
| `fade` | `fade <in-sec> [out-sec]` | 線性淡入/淡出 |
| `reverse` | `reverse` | 反轉取樣 |
| `speed` | `speed <factor>` | 改變播放速度 |
| `pad` | `pad <start-sec> [end-sec]` | 開頭/結尾補靜音 |
| `silence` | `silence <threshold-db> [min-sec]` | 移除前後靜音 |
| `lowpass` | `lowpass <hz>` | 低通濾波（一階） |
| `highpass` | `highpass <hz>` | 高通濾波（一階） |
| `limiter` | `limiter [threshold]` | 硬限制器（預設 0.95） |
| `rate` | `rate <hz>` | 重新取樣 |
| `channels` | `channels <count>` | 聲道數量轉換 |
| `stat` | `stat` | 列印音訊統計 |

## 合成波形

- 波形：`sine`（正弦波）、`square`（方波）、`triangle`（三角波）、`saw`（鋸齒波）、`noise`（雜訊）、`silence`（靜音）
- 後處理：`--gain-db`、`--normalize`、`--fade`

## 編解碼支援

| 方向 | 支援格式 |
|------|---------|
| **讀取** | WAV、FLAC、MP3、Ogg/Vorbis、Opus、AAC、ALAC、CAF、MKV/WebM、M4A |
| **寫入** | WAV（16-bit PCM） |
| **串流** | WAV → WAV（gain、fade、limiter） |

---

## 專案結構

```
sox/                       # crate 根目錄
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs            # 入口、命令分發
│   ├── cli.rs             # Clap 參數定義
│   ├── audio.rs           # AudioBuffer、WAV/Symphonia I/O
│   ├── effects.rs         # Effect 列舉、EffectChain、DSP
│   ├── parse.rs           # 效果 token 解析器
│   ├── io.rs              # 檔案 I/O 工具
│   ├── mix.rs             # 串接與混音
│   ├── streaming.rs       # 增量 WAV 管線
│   ├── synth.rs           # 波形合成
│   ├── stats.rs           # 音訊統計
│   └── util.rs            # 共用工具函式
├── tests/
│   ├── cli.rs             # 整合測試（8 項）
│   ├── effects.rs         # 效果單元測試（65 項）
│   └── common/mod.rs      # 測試輔助工具
├── docs/
│   ├── ARCHITECTURE.md
│   ├── KNOWLEDGE_MAP.md
│   └── ROADMAP.md
└── pages/
    ├── en/README.md
    └── zh-TW/README.md
```

---

## 建置與安裝

```powershell
# Debug 建置
cargo build

# Release 建置
cargo build --release

# 執行檔位於 target/release/sox (或 sox.exe)
./target/release/sox --help
```

**前置需求：** [Rust](https://www.rust-lang.org/tools/install) 1.85+

---

## 授權條款

雙重授權：**LGPL-2.1-or-later** 或 **MIT**（任選其一）。
