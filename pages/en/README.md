# sox — A Rust-first, SoX-inspired Audio Processor

**Pure Rust · No C dependencies · SoX-compatible CLI**

`sox` (rust-sox) is a from-scratch Rust rewrite of the classic [SoX](http://sox.sourceforge.net/)
(Sound eXchange) command-line audio tool. It prioritises safety, portability, and
a clean codebase over absolute feature parity — starting with a WAV processing
core and expanding outward through the [Symphonia](https://github.com/pdeljanov/Symphonia)
ecosystem for broader codec support.

> **Status:** Active development — Milestone 1 complete.  
> Ready for everyday audio processing tasks.

---

## Why sox?

- **No C dependencies** — pure Rust audio processing. Easier to build, audit, and contribute to.
- **Deterministic pipeline** — all effects operate on an interleaved `f32` sample buffer, making the signal path predictable and testable.
- **SoX-compatible CLI** — familiar `input output effect...` syntax for common workflows. Also supports subcommands.
- **Streaming** — process large files incrementally without loading everything into memory.
- **JSON plans** — repeatable, scriptable processing plans.
- **Parallel batch** — multi-core batch conversion with a single command.
- **Built-in synthesis** — generate tones and waveforms via `synth`.

---

## Quick Start

```bash
# Install via cargo
cargo install rust-sox

# Or build from source
git clone https://github.com/stevenke1981/sox-rs.git
cd sox-rs
cargo build --release
./target/release/sox --help
```

```bash
# Info
sox info input.wav
sox info --json input.wav

# Basic conversion with effects
sox convert input.wav out.wav --gain-db -3 --trim 0 10 --normalize

# SoX-style legacy syntax
sox input.wav out.wav gain -3 trim 0 10 norm rate 48000 stat

# Concatenate
sox concat -o album.wav intro.wav body.wav outro.wav

# Mix
sox mix -o bed.wav voice.wav music.wav --normalize

# Synthesis
sox synth tone.wav --duration 2 --freq 440 --waveform sine --fade 0.05

# Streaming (low-memory large file processing)
sox stream huge.wav processed.wav --gain-db=-3 --fade-in 0.5 --fade-out 0.5

# List supported codecs
sox formats

# Parallel batch
sox batch "samples/*.wav" --out-dir out --normalize --rate 48000

# Run from a JSON plan
sox run-plan plan.json
```

---

## Installation

### Pre-built binaries

Download from [GitHub Releases](https://github.com/stevenke1981/sox-rs/releases):

| Platform | Package | Binary |
|----------|---------|--------|
| Windows x86_64 | `sox-<version>-win64.zip` | `sox.exe` |
| Linux x86_64 | `sox-<version>-linux.tar.gz` | `sox` |
| macOS x86_64 | `sox-<version>-macos.tar.gz` | `sox` |

### From source

```bash
cargo install rust-sox
# or
cargo build --release
# binary at target/release/sox (or sox.exe on Windows)
```

---

## Supported Effects

| Effect | Syntax | Description |
|--------|--------|-------------|
| `gain` | `gain <db>` | Apply gain in decibels |
| `norm` | `norm [target-db]` | Normalise peak amplitude (default −1 dBFS) |
| `trim` | `trim <start-sec> [duration-sec]` | Cut from start, optionally for a duration |
| `fade` | `fade <in-sec> [out-sec]` | Linear fade in/out |
| `reverse` | `reverse` | Reverse samples in place |
| `speed` | `speed <factor>` | Change playback speed (resamples) |
| `pad` | `pad <start-sec> [end-sec]` | Add silence at beginning and/or end |
| `silence` | `silence <threshold-db> [min-sec]` | Remove leading/trailing silence |
| `lowpass` | `lowpass <hz>` | Low-pass filter (first-order) |
| `highpass` | `highpass <hz>` | High-pass filter (first-order) |
| `limiter` | `limiter [threshold]` | Hard limiter (default 0.95) |
| `rate` | `rate <hz>` | Resample to a new sample rate |
| `channels` | `channels <count>` | Convert channel count |
| `stat` | `stat` | Print audio statistics |

## Synth Waveforms

- Waveforms: `sine`, `square`, `triangle`, `saw`, `noise`, `silence`
- Post-processing: `--gain-db`, `--normalize`, `--fade`

## Codec Support

| Direction | Formats |
|-----------|---------|
| **Read** | WAV, FLAC, MP3, Ogg/Vorbis, Opus, AAC, ALAC, CAF, MKV/WebM, M4A |
| **Write** | WAV (16-bit PCM) |
| **Stream** | WAV → WAV (gain, fade, limiter only) |

---

## Project Layout

```
sox/                       # crate root
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs            # Entry point, command dispatch
│   ├── cli.rs             # Clap argument definitions
│   ├── audio.rs           # AudioBuffer, WAV/Symphonia I/O
│   ├── effects.rs         # Effect enum, EffectChain, DSP
│   ├── parse.rs           # Effect token parser
│   ├── io.rs              # File I/O utilities
│   ├── mix.rs             # Concat and mix
│   ├── streaming.rs       # Incremental WAV pipeline
│   ├── synth.rs           # Waveform generation
│   ├── stats.rs           # Audio statistics
│   └── util.rs            # Shared utility functions
├── tests/
│   ├── cli.rs             # Integration tests (8)
│   ├── effects.rs         # Effect unit tests (65)
│   └── common/mod.rs      # Test helpers
├── docs/
│   ├── ARCHITECTURE.md
│   ├── KNOWLEDGE_MAP.md
│   └── ROADMAP.md
└── pages/
    ├── en/README.md
    └── zh-TW/README.md
```

---

## Build & Install

```powershell
# Debug build
cargo build

# Release build
cargo build --release

# binary at target/release/sox (or sox.exe)
./target/release/sox --help
```

**Prerequisites:** [Rust](https://www.rust-lang.org/tools/install) 1.85+

---

## License

Dual-licensed under **LGPL-2.1-or-later** or **MIT** at your option.
