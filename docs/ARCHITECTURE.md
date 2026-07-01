# Architecture

`sox` is split around three processing paths:

- **AudioBuffer path**: decodes supported inputs into interleaved `f32` samples,
  applies `EffectChain`, and writes WAV.
- **Streaming path**: processes WAV samples incrementally for large-file gain,
  fade, and limiter workflows.
- **Synth path**: generates built-in waveforms into `AudioBuffer`, then reuses the
  same effect chain and WAV writer.

Codec support is intentionally read-heavy at this stage. WAV uses `hound` for
direct I/O; non-WAV input is decoded through `symphonia`. Encoding remains WAV
until each additional writer backend can be selected with clear licensing and
quality tradeoffs.

## Module Layout

```
src/
├── main.rs        Entry point + command dispatch
├── cli.rs         CLI args, subcommands, legacy parser
├── audio.rs       AudioBuffer, WAV/Symphonia I/O
├── effects.rs     Effect enum, EffectChain, DSP
├── parse.rs       Effect token parser
├── io.rs          File I/O utilities
├── mix.rs         Concat and mix
├── streaming.rs   Incremental WAV pipeline
├── synth.rs       Waveform generation
├── stats.rs       Audio statistics (peak, RMS, clipping)
└── util.rs        Shared utility functions
```
