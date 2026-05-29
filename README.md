# rust-sox

`rust-sox` is a Rust-first rewrite path for the classic SoX command shape,
starting with a safe, portable WAV processing core.

This is not yet a full replacement for upstream SoX. The first milestone focuses
on the pieces that make the rewrite useful and extensible:

- WAV read/write without C bindings.
- Decode common input codecs through Symphonia, including FLAC, MP3,
  Ogg/Vorbis, Opus, AAC, ALAC, CAF, MKV/WebM, and MP4/M4A.
- A deterministic interleaved `f32` processing pipeline.
- SoX-style `input output effect...` compatibility for common effects.
- Multi-file `concat` and `mix` workflows.
- Repeatable JSON processing plans.
- Built-in audio generation through `synth`.
- Streaming WAV-to-WAV processing for large files that should not be fully
  loaded into memory.
- Structured `info` and `stat` output, including JSON.
- Parallel batch conversion.

## Examples

```powershell
cargo run -- info input.wav
cargo run -- convert input.wav out.wav --gain-db -3 --trim 0 10 --normalize
cargo run -- input.wav out.wav gain -3 trim 0 10 norm rate 48000 stat
cargo run -- concat -o album.wav intro.wav body.wav outro.wav
cargo run -- mix -o bed.wav voice.wav music.wav --normalize
cargo run -- synth tone.wav --duration 2 --freq 440 --waveform sine --fade 0.05
cargo run -- stream huge.wav huge-processed.wav --gain-db=-3 --fade-in 0.5 --fade-out 0.5
cargo run -- formats
cargo run -- batch "samples\*.wav" --out-dir out --normalize --rate 48000
cargo run -- run-plan plan.json
```

Release builds place the executable at `target\release\rust-sox.exe`; the
delivery copy is `dist\rust-sox.exe`.

## Supported Effects

- `gain <db>`
- `norm [target-db]`
- `trim <start-sec> [duration-sec]`
- `fade <in-sec> [out-sec]`
- `reverse`
- `speed <factor>`
- `pad <start-sec> [end-sec]`
- `silence <threshold-db> [min-sec]`
- `lowpass <hz>`
- `highpass <hz>`
- `limiter [threshold]`
- `rate <sample-rate>`
- `channels <count>`
- `stat`

## Synth Waveforms

`synth` can generate `sine`, `square`, `triangle`, `saw`, `noise`, and
`silence`. Generated audio can be post-processed with `--gain-db`,
`--normalize`, and `--fade`.

## Codecs

`rust-sox` writes WAV today. Input decoding is wider: WAV uses the native hound
reader, while non-WAV inputs go through Symphonia. Use `rust-sox formats` to see
the supported read/write summary.

## Streaming

`stream` is the large-file path. It currently supports WAV input/output with
gain, fade-in, fade-out, and limiting while processing samples incrementally
instead of building an `AudioBuffer`.

## JSON Plans

```json
{
  "mode": "convert",
  "inputs": ["input.wav"],
  "output": "out.wav",
  "effects": ["trim", "0", "10", "gain", "-3", "norm"],
  "stat_json": true
}
```

`mode` can be `convert`, `concat`, or `mix`.

## Rewrite Notes

The upstream reference used for command-shape comparison is
`https://github.com/chirlu/sox.git`.

The current implementation intentionally starts with WAV because it allows the
Rust DSP and CLI architecture to stabilize before introducing codec backends
such as FLAC, MP3, Ogg/Vorbis, Opus, and platform audio devices.
