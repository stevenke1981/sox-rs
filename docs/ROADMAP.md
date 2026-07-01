# sox (rust-sox) Roadmap

## Milestone 1: Rust Core

- Implement portable WAV I/O.
- Represent samples as normalized interleaved `f32`.
- Add common effects: gain, normalize, trim, fade, reverse, resample, channel
  conversion.
- Provide SoX-style positional command parsing.
- Add JSON metadata and statistics for automation.
- Add parallel batch processing.
- Add multi-file concat/mix workflows.
- Add JSON processing plans for repeatable jobs.
- Add built-in synth generation for tones, noise, and silence.

## Milestone 2: Codec Expansion

- Add feature-gated decoders and encoders:
  - FLAC, MP3, Ogg/Vorbis, Opus, AAC, ALAC, CAF, MKV/WebM, and MP4/M4A
    decoding is started through Symphonia.
  - WAV remains the first writer target.
  - Add additional encoders once backend and licensing choices are explicit.
- Preserve streaming APIs so large files do not require full memory loading for
  effects that can run incrementally.

## Milestone 3: DSP Compatibility

- Port SoX effects incrementally with golden tests against upstream output.
- Start with effects that have simple state and stable semantics:
  `vol`, `gain`, `trim`, `fade`, `reverse`, `speed`, `pad`, `rate`,
  `channels`, `silence`, `lowpass`, `highpass`, `limiter`.
- Add more complex filters after the test harness can compare spectra and
  sample tolerances.

## Milestone 4: New Rust-First Features

- Richer JSON plans with per-step names, comments, and batch targets.
- Parallel batch jobs with per-file reports.
- Safer clipping diagnostics and optional true-peak estimation.
- More streaming effects beyond gain/fade/limiter.
- WASM-compatible DSP core for future UI or browser use.
