# Architecture

`rust-sox` is split around three processing paths:

- `AudioBuffer` path: decodes supported inputs into interleaved `f32` samples,
  applies `EffectChain`, and writes WAV.
- Streaming path: processes WAV samples incrementally for large-file gain,
  fade, and limiter workflows.
- Synth path: generates built-in waveforms into `AudioBuffer`, then reuses the
  same effect chain and WAV writer.

Codec support is intentionally read-heavy at this stage. WAV uses `hound` for
direct I/O; non-WAV input is decoded through `symphonia`. Encoding remains WAV
until each additional writer backend can be selected with clear licensing and
quality tradeoffs.
