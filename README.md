# sox — Rust 原生音訊處理工具

> **SoX 的精神 · Rust 的安全 · 現代化的設計**

`sox`（rust-sox）是經典命令列音訊工具 [SoX](http://sox.sourceforge.net/)（Sound eXchange）
的 Rust 原生重製版。以安全性、可攜性與乾淨程式碼為優先，
從 WAV 核心出發，逐步擴充至 Symphonia 編解碼器生態。

---

## 快速開始

```bash
# 安裝（方法一：從原始碼建置）
cargo install rust-sox

# 安裝（方法二：使用安裝包）
# 從 Releases 下載對應平台的安裝包

# 安裝（方法三：從原始碼）
git clone https://github.com/stevenke1981/sox-rs.git
cd sox-rs
cargo build --release
./target/release/sox --help
```

---

## 使用範例

```bash
# 檢視音訊資訊
sox info input.wav
sox info --json input.wav

# 轉換 + 效果處理
sox convert input.wav output.wav --gain-db -3 --trim 0 10 --normalize

# 合成音訊
sox synth tone.wav --duration 2 --freq 440 --waveform sine

# 串接與混音
sox concat -o album.wav intro.wav body.wav outro.wav
sox mix -o bed.wav voice.wav music.wav --normalize

# 批次處理
sox batch "samples/*.wav" --out-dir out --normalize --rate 48000

# JSON 處理計畫
sox run-plan plan.json

# SoX 風格命令列（相容模式）
sox input.wav output.wav gain -6 trim 0 0.01 pad 0.01 0.01 speed 1.5 rate 22050

# 串流處理（適合大檔案）
sox stream input.wav output.wav --gain-db=-3 --fade-in 0.01 --limiter 0.7
```

### 支援的效果

| 效果 | 說明 |
|------|------|
| `gain` / `vol` | 音量增益（dB） |
| `normalize` | 正規化至目標音量 |
| `trim` | 裁剪音訊區段 |
| `fade` | 淡入淡出 |
| `reverse` | 反轉音訊 |
| `speed` | 變速（線性插值） |
| `pad` | 填補靜音 |
| `silence` | 移除前後靜音 |
| `lowpass` | 低通濾波器 |
| `highpass` | 高通濾波器 |
| `limiter` | 限制器（hard-clamp） |
| `rate` | 重取樣 |
| `channels` | 聲道轉換 |

### 支援的格式

| 方向 | 格式 |
|------|------|
| 讀取 | WAV, FLAC, MP3, Ogg/Vorbis, Opus, AAC, ALAC, CAF, MKV/WebM, MP4/M4A |
| 寫入 | WAV（16-bit PCM） |
| 串流 | WAV → WAV（gain, fade, limiter） |

---

## 架構

```
sox/               # ── crate root
├── src/
│   ├── main.rs    # 入口、命令分發
│   ├── cli.rs     # CLI 參數解析
│   ├── audio.rs   # AudioBuffer、WAV I/O、Symphonia 解碼
│   ├── effects.rs # Effect 枚舉、EffectChain、DSP 實作
│   ├── parse.rs   # 效果 token 解析器
│   ├── io.rs      # 檔案 I/O 公用函式
│   ├── mix.rs     # 串接與混音
│   ├── streaming.rs # 增量 WAV 管線
│   ├── synth.rs   # 波形合成
│   ├── stats.rs   # 音訊統計報告
│   └── util.rs    # 共用工具函式
├── tests/
│   ├── cli.rs     # CLI 整合測試（8 項）
│   ├── effects.rs # 效果單元測試（65 項）
│   └── common/    # 測試輔助工具
└── docs/
    ├── ARCHITECTURE.md
    ├── KNOWLEDGE_MAP.md
    └── ROADMAP.md
```

三條處理路徑：
1. **AudioBuffer 路徑**：將輸入解碼為 `f32` 交錯樣本，套用效果鏈，輸出 WAV
2. **Streaming 路徑**：增量處理 WAV 樣本，適合大檔案
3. **Synth 路徑**：生成內建波形，重複使用效果鏈與 WAV 寫入器

---

## 安裝包

每個 Release 包含以下平台的靜態連結二進位檔：

| 平台 | 格式 |
|------|------|
| Windows x86_64 | `sox-x.y.z-win64.zip`（`sox.exe`） |
| Linux x86_64 | `sox-x.y.z-linux.tar.gz`（`sox`） |
| macOS x86_64 | `sox-x.y.z-macos.tar.gz`（`sox`） |

### 從原始碼建置

```bash
cargo build --release
# 執行檔位於 target/release/sox (或 sox.exe)
```

---

## 授權

**LGPL-2.1-or-later** 或 **MIT**（任選其一）。

---

## 相關連結

- [English Documentation](pages/en/README.md)
- [繁體中文說明](pages/zh-TW/README.md)
- [架構文件](docs/ARCHITECTURE.md)
- [知識圖譜](docs/KNOWLEDGE_MAP.md)
- [開發藍圖](docs/ROADMAP.md)
