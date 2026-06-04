# rust-sox

**A Rust-first, SoX-inspired audio processor.**

`rust-sox` is a from-scratch Rust rewrite of the classic [SoX](http://sox.sourceforge.net/)
(Sound eXchange) command-line audio tool. It prioritises safety, portability, and
a clean codebase over absolute feature parity — starting with a WAV processing
core and expanding outward through the [Symphonia](https://github.com/pdeljanov/Symphonia)
ecosystem for broader codec support.

> **Status:** Active development — not yet a full SoX replacement.  
> Milestone 1 focuses on the pieces that make a rewrite useful and extensible.

---

## Why rust-sox?

- **No C dependencies** — pure Rust audio processing. Easier to build, audit, and
  contribute to.
- **Deterministic pipeline** — all effects operate on an interleaved `f32` sample
  buffer, making the signal path predictable and testable.
- **SoX-compatible CLI** — familiar `input output effect...` syntax for common
  workflows.
- **Streaming** — process large files incrementally without loading everything
  into memory.
- **JSON plans** — repeatable, scriptable processing plans.
- **Parallel batch** — multi-core batch conversion with a single command.
- **Built-in synthesis** — generate tones and waveforms via `synth`.

---

## Quick Start

```powershell
# Info
cargo run -- info input.wav

# Basic conversion with effects
cargo run -- convert input.wav out.wav --gain-db -3 --trim 0 10 --normalize

# SoX-style legacy syntax
cargo run -- input.wav out.wav gain -3 trim 0 10 norm rate 48000 stat

# Concatenate
cargo run -- concat -o album.wav intro.wav body.wav outro.wav

# Mix
cargo run -- mix -o bed.wav voice.wav music.wav --normalize

# Synthesis
cargo run -- synth tone.wav --duration 2 --freq 440 --waveform sine --fade 0.05

# Streaming (low-memory large file processing)
cargo run -- stream huge.wav huge-processed.wav --gain-db=-3 --fade-in 0.5 --fade-out 0.5

# List supported codecs
cargo run -- formats

# Parallel batch
cargo run -- batch "samples\*.wav" --out-dir out --normalize --rate 48000

# Run from a JSON plan
cargo run -- run-plan plan.json
```

Release builds produce `target\release\rust-sox.exe`; a delivery copy is also
placed at `dist\rust-sox.exe`.

---

## Features

### Supported Effects

| Effect         | Syntax                              | Description                                    |
|----------------|-------------------------------------|------------------------------------------------|
| `gain`         | `gain <db>`                         | Apply gain in decibels                         |
| `norm`         | `norm [target-db]`                  | Normalise peak amplitude (default −1 dBFS)     |
| `trim`         | `trim <start-sec> [duration-sec]`   | Cut from start, optionally for a duration      |
| `fade`         | `fade <in-sec> [out-sec]`           | Linear fade in/out                             |
| `reverse`      | `reverse`                           | Reverse samples in place                       |
| `speed`        | `speed <factor>`                    | Change playback speed (resamples)              |
| `pad`          | `pad <start-sec> [end-sec]`         | Add silence at beginning and/or end            |
| `silence`      | `silence <threshold-db> [min-sec]`  | Remove leading silence                         |
| `lowpass`      | `lowpass <hz>`                      | Low-pass filter (first-order)                  |
| `highpass`     | `highpass <hz>`                     | High-pass filter (first-order)                 |
| `limiter`      | `limiter [threshold]`               | Soft-knee limiter (default 0.95)               |
| `rate`         | `rate <hz>`                         | Resample to a new sample rate                  |
| `channels`     | `channels <count>`                  | Convert channel count (mix up / take first)    |
| `stat`         | `stat`                              | Print audio statistics                         |

### Synth Waveforms

The `synth` subcommand generates audio from scratch:

- Waveforms: `sine`, `square`, `triangle`, `saw`, `noise`, `silence`
- Post-processing: `--gain-db`, `--normalize`, `--fade`

### Codec Support

| Direction | Formats                                                          |
|-----------|------------------------------------------------------------------|
| **Read**  | WAV, FLAC, MP3, Ogg/Vorbis, Opus, AAC, ALAC, CAF, MKV/WebM, M4A |
| **Write** | WAV (16-bit PCM)                                                 |
| **Stream**| WAV → WAV (gain, fade, limiter only)                             |

Input decoding beyond WAV is handled by [Symphonia](https://github.com/pdeljanov/Symphonia).
Additional writer backends will be added as the project matures.

### Streaming

The `stream` subcommand is designed for files too large to fit in memory. It
processes WAV samples incrementally — currently supporting gain, fade-in,
fade-out, and limiting — without constructing an `AudioBuffer`.

### JSON Processing Plans

```json
{
  "mode": "convert",
  "inputs": ["input.wav"],
  "output": "out.wav",
  "effects": ["trim", "0", "10", "gain", "-3", "norm"],
  "stat_json": true
}
```

`mode` accepts `convert`, `concat`, or `mix`.

### Parallel Batch Conversion

```powershell
cargo run -- batch "samples\*.wav" --out-dir out --normalize --rate 48000
```

Files matching the glob are processed in parallel across CPU cores via `rayon`.
Any failures are reported at the end — the batch does not stop on the first error.

---

## Architecture

`rust-sox` is organised around three processing paths:

1. **AudioBuffer path** — decodes input into interleaved `f32` samples, applies
   `EffectChain`, writes WAV. The main path for most commands.
2. **Streaming path** — processes WAV samples incrementally for memory-efficient
   gain, fade, and limiter workflows.
3. **Synth path** — generates built-in waveforms directly into an `AudioBuffer`,
   then reuses the same effect chain and WAV writer.

See [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) for more detail.

---

## Project Layout

```
rust-sox/
├── Cargo.toml          # Package metadata and dependencies
├── README.md           # This file
├── src/
│   ├── main.rs         # CLI dispatch, legacy SoX compat, plan runner
│   ├── cli.rs          # Clap argument definitions
│   ├── audio.rs        # AudioBuffer, read/write, resampling, channel conversion
│   ├── effects.rs      # Effect enum and EffectChain
│   ├── streaming.rs    # Streaming WAV processor
│   ├── synth.rs        # Waveform synthesis
│   └── stats.rs        # Audio statistics reporting
├── tests/
│   └── cli.rs          # Integration tests
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

# The binary is at target/release/rust-sox.exe
# A convenience copy is at dist/rust-sox.exe
```

**Prerequisites:** [Rust](https://www.rust-lang.org/tools/install) (edition 2024,
MSRV determined by dependencies).

---

## Rewrite Notes

The upstream reference for command-shape comparison is
[chirlu/sox](https://github.com/chirlu/sox). The current implementation
intentionally starts with WAV to let the Rust DSP and CLI architecture stabilise
before introducing additional codec backends (FLAC, MP3, Ogg/Vorbis, Opus) and
platform audio device I/O.

---

## License

`rust-sox` is dual-licensed under **LGPL-2.1-or-later** or **MIT** at your
option. See the [`LICENSE-LGPL`](LICENSE-LGPL) and [`LICENSE-MIT`](LICENSE-MIT)
files (or the top-level `Cargo.toml` license field) for details.
