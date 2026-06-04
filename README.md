# rust-sox

**A Rust-first, SoX-inspired audio processor** ·
**以 Rust 為核心、靈感源自 SoX 的音訊處理工具**

---

## Choose your language / 選擇語言

| Language | |
|----------|-|
| 🇬🇧 [English](pages/en/README.md) | Full documentation in English |
| 🇹🇼 [繁體中文](pages/zh-TW/README.md) | 完整繁體中文說明 |

---

**rust-sox** is a from-scratch Rust rewrite of the classic SoX command-line
audio tool. It prioritises safety, portability, and clean code — starting with
a WAV core and expanding through the Symphonia codec ecosystem.

**rust-sox** 是經典命令列音訊工具 SoX 的 Rust 原生重製版。
以安全性、可攜性與乾淨的程式碼為優先，從 WAV 核心出發，逐步擴充編解碼器支援。

### Quick peek / 快速一瞥

```powershell
cargo run -- info input.wav
cargo run -- convert input.wav out.wav --gain-db -3 --trim 0 10 --normalize
cargo run -- synth tone.wav --duration 2 --freq 440 --waveform sine
cargo run -- concat -o album.wav intro.wav body.wav outro.wav
cargo run -- mix -o bed.wav voice.wav music.wav --normalize
cargo run -- batch "samples\*.wav" --out-dir out --normalize --rate 48000
cargo run -- run-plan plan.json
```

---

[English »](pages/en/README.md) · [繁體中文 »](pages/zh-TW/README.md)
