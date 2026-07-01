# sox (rust-sox) Knowledge Maps

## 1. Module Dependency Map

```mermaid
graph TD
    main["main.rs<br/><i>entry point + orchestration</i>"]
    cli["cli.rs<br/><i>CLI args, subcommands, legacy parser</i>"]
    audio["audio.rs<br/><i>AudioBuffer, WAV/Symphonia I/O</i>"]
    effects["effects.rs<br/><i>Effect enum, EffectChain, DSP</i>"]
    stats["stats.rs<br/><i>Report, peak/RMS/clipping</i>"]
    streaming["streaming.rs<br/><i>incremental WAV pipeline</i>"]
    synth["synth.rs<br/><i>waveform generation</i>"]

    main --> cli
    main --> audio
    main --> effects
    main --> stats
    main --> streaming
    main --> synth
    cli --> effects
    synth --> audio
    synth --> cli
    streaming --> cli
    stats --> audio
```

## 2. Type Relationship Map

```mermaid
classDiagram
    class AudioSpec {
        +u32 sample_rate
        +u16 channels
    }

    class AudioBuffer {
        +AudioSpec spec
        +Vec~f32~ samples
        +read(path) Result~Self~
        +read_wav(path) Result~Self~
        +write_wav(path) Result~()~
        +frames() usize
        +duration_seconds() f64
        +peak() f32
    }

    class Effect {
        <<enum>>
        GainDb(f32)
        Normalize{target_db}
        Trim{start_sec, duration_sec}
        Fade{in_sec, out_sec}
        Reverse
        Speed{factor}
        Pad{start_sec, end_sec}
        Silence{threshold_db, min_duration_sec}
        LowPass{hz}
        HighPass{hz}
        Limiter{threshold}
        Rate{sample_rate}
        Channels{channels}
        Stats
    }

    class EffectChain {
        -Vec~Effect~ effects
        +new(effects) Self
        +apply(audio) Result~()~
        +wants_stats() bool
    }

    class Report {
        +PathBuf path
        +AudioSpec spec
        +usize frames
        +f64 duration_seconds
        +f32 peak
        +f32 rms
        +usize clipped_samples
        +from_audio(path, audio) Self
    }

    class Commands {
        <<enum>>
        Info
        Formats
        Convert
        Concat
        Mix
        Synth
        Stream
        Batch
        RunPlan
    }

    class ProcessingPlan {
        +PlanMode mode
        +Vec~PathBuf~ inputs
        +PathBuf output
        +Vec~String~ effects
        +bool normalize
        +bool stat_json
    }

    AudioBuffer *-- AudioSpec
    EffectChain o-- Effect
    Report *-- AudioSpec
    Report ..> AudioBuffer : from_audio
    EffectChain ..> AudioBuffer : apply
```

## 3. CLI Command Routing

```mermaid
flowchart LR
    ARGV["argv"] --> LEGACY{LegacyCommand<br/>from_env}
    LEGACY -->|positional args| LC["run_legacy"]
    LEGACY -->|subcommand| CLAP["clap parse"]

    CLAP --> INFO["run_info"]
    CLAP --> FMT["run_formats"]
    CLAP --> CONV["run_convert"]
    CLAP --> CAT["run_concat"]
    CLAP --> MIX["run_mix"]
    CLAP --> SYN["run_synth"]
    CLAP --> STR["run_stream"]
    CLAP --> BAT["run_batch"]
    CLAP --> PLAN["run_plan"]

    LC --> CONVERT_ONE["convert_one"]
    CONV --> CONVERT_ONE
    CAT --> CONCAT["concat_inputs"]
    MIX --> MIXFN["mix_inputs"]
    SYN --> RENDER["synth::render"]
    STR --> STREAM["streaming::stream_wav"]
    BAT --> CONVERT_ONE
    PLAN --> CONVERT_ONE
    PLAN --> CONCAT
    PLAN --> MIXFN

    CONVERT_ONE --> READ["AudioBuffer::read"]
    CONVERT_ONE --> APPLY["EffectChain::apply"]
    CONVERT_ONE --> WRITE["write_output"]

    CONCAT --> READ
    MIXFN --> READ
    RENDER --> APPLY

    WRITE --> WAV["AudioBuffer::write_wav"]
    WRITE --> REPORT["stats::Report"]
```

## 4. Audio Data Flow

```mermaid
flowchart TD
    subgraph Input
        WAV_IN["WAV file<br/>(hound)"]
        SYM_IN["FLAC/MP3/OGG/OPUS/AAC/...<br/>(symphonia)"]
    end

    subgraph Decode
        READ_WAV["AudioBuffer::read_wav<br/>8/16/24/32-bit int + float"]
        READ_SYM["AudioBuffer::read_with_symphonia<br/>packet decode loop"]
    end

    subgraph Process
        CHAIN["EffectChain::apply"]
        GAIN["gain_db"]
        NORM["normalize"]
        TRIM["trim"]
        FADE["fade"]
        REV["reverse"]
        SPD["speed (linear interp)"]
        PAD["pad"]
        SIL["trim_silence"]
        LP["lowpass (1-pole IIR)"]
        HP["highpass (1-pole IIR)"]
        LIM["limiter (hard clamp)"]
        RATE["rate (linear resample)"]
        CH["channels (up/downmix)"]
    end

    subgraph Output
        WRITE["AudioBuffer::write_wav<br/>16-bit PCM"]
        STREAM_OUT["streaming::stream_wav<br/>sample-by-sample"]
        STAT_OUT["stats::Report<br/>text or JSON"]
    end

    WAV_IN --> READ_WAV
    SYM_IN --> READ_SYM
    READ_WAV --> CHAIN
    READ_SYM --> CHAIN
    CHAIN --> GAIN --> NORM --> TRIM --> FADE --> REV --> SPD --> PAD --> SIL --> LP --> HP --> LIM --> RATE --> CH
    CH --> WRITE
    CH --> STAT_OUT
    WAV_IN --> STREAM_OUT
```

## 5. Call Graph (Key Functions)

```mermaid
graph TD
    main_fn["main()"] --> run["run()"]
    run --> legacy["LegacyCommand::from_env()"]
    run --> run_info["run_info()"]
    run --> run_formats["run_formats()"]
    run --> run_convert["run_convert()"]
    run --> run_concat["run_concat()"]
    run --> run_mix["run_mix()"]
    run --> run_synth["run_synth()"]
    run --> run_stream["run_stream()"]
    run --> run_batch["run_batch()"]
    run --> run_plan["run_plan()"]

    run_convert --> convert_one["convert_one()"]
    run_batch --> convert_one
    run_plan --> convert_one
    convert_one --> ab_read["AudioBuffer::read()"]
    convert_one --> ec_apply["EffectChain::apply()"]
    convert_one --> write_output["write_output()"]

    run_concat --> concat_inputs["concat_inputs()"]
    concat_inputs --> ab_read
    run_mix --> mix_inputs["mix_inputs()"]
    mix_inputs --> ab_read
    mix_inputs --> sample_at["sample_at()"]

    run_synth --> synth_render["synth::render()"]
    synth_render --> waveform_sample["waveform_sample()"]
    synth_render --> noise_next["Noise::next()"]

    run_stream --> stream_wav["streaming::stream_wav()"]
    stream_wav --> stream_samples["stream_samples()"]
    stream_samples --> fade_factor["fade_factor()"]

    write_output --> ab_write["AudioBuffer::write_wav()"]
    write_output --> report_from["Report::from_audio()"]

    ec_apply --> gain_db["gain_db()"]
    ec_apply --> normalize["normalize()"]
    ec_apply --> trim["trim()"]
    ec_apply --> fade["fade()"]
    ec_apply --> reverse["reverse()"]
    ec_apply --> speed["speed()"]
    ec_apply --> pad["pad()"]
    ec_apply --> trim_silence["trim_silence()"]
    ec_apply --> lowpass["lowpass()"]
    ec_apply --> highpass["highpass()"]
    ec_apply --> limiter["limiter()"]
    ec_apply --> rate_fn["rate()"]
    ec_apply --> convert_channels["convert_channels()"]

    ab_read --> read_wav["AudioBuffer::read_wav()"]
    ab_read --> read_symphonia["AudioBuffer::read_with_symphonia()"]
    read_wav --> read_int_samples["read_int_samples()"]

    run_plan --> parse_effects["parse_effects()"]
    legacy --> parse_effects
```

## 6. Effect Parser Token Flow

```mermaid
flowchart LR
    TOKENS["token stream<br/>['gain', '-3', 'trim', '0', '10', 'norm']"] --> PARSE["parse_effects()"]
    PARSE --> LOOP["match loop"]

    LOOP -->|"gain / vol"| GAIN_E["GainDb(f32)"]
    LOOP -->|"norm / normalize"| NORM_E["Normalize{target_db}"]
    LOOP -->|"trim"| TRIM_E["Trim{start_sec, duration_sec}"]
    LOOP -->|"fade"| FADE_E["Fade{in_sec, out_sec}"]
    LOOP -->|"reverse"| REV_E["Reverse"]
    LOOP -->|"speed"| SPD_E["Speed{factor}"]
    LOOP -->|"pad"| PAD_E["Pad{start_sec, end_sec}"]
    LOOP -->|"silence"| SIL_E["Silence{threshold_db, min_duration_sec}"]
    LOOP -->|"lowpass"| LP_E["LowPass{hz}"]
    LOOP -->|"highpass"| HP_E["HighPass{hz}"]
    LOOP -->|"limiter"| LIM_E["Limiter{threshold}"]
    LOOP -->|"rate / resample"| RATE_E["Rate{sample_rate}"]
    LOOP -->|"channels / ch"| CH_E["Channels{channels}"]
    LOOP -->|"stat / stats"| STAT_E["Stats"]
    LOOP -->|"stat-json"| STAT_JSON["stat_json = true<br/>+ Stats"]
```

## 7. External Crate Dependencies

```mermaid
graph LR
    subgraph rust-sox
        A["audio.rs"]
        C["cli.rs"]
        E["effects.rs"]
        S["stats.rs"]
        ST["streaming.rs"]
        SY["synth.rs"]
        M["main.rs"]
    end

    subgraph Crates
        HOUND["hound 3.5<br/><i>WAV read/write</i>"]
        SYMPH["symphonia 0.5<br/><i>multi-codec decode</i>"]
        CLAP["clap 4.5<br/><i>CLI parsing</i>"]
        SERDE["serde + serde_json<br/><i>JSON plans, info output</i>"]
        ANYHOW["anyhow 1.0<br/><i>error handling</i>"]
        RAYON["rayon 1.10<br/><i>parallel batch</i>"]
        GLOB["glob 0.3<br/><i>file expansion</i>"]
    end

    A --> HOUND
    A --> SYMPH
    ST --> HOUND
    C --> CLAP
    C --> SERDE
    M --> SERDE
    M --> ANYHOW
    M --> RAYON
    M --> GLOB
    S --> SERDE
```

## 8. Test Coverage Map

```mermaid
graph LR
    subgraph "tests/cli.rs"
        T1["converts_with_sox_style_effects"]
        T2["lists_format_support"]
        T3["prints_json_info"]
        T4["concatenates_and_mixes_inputs"]
        T5["streams_wav_without_full_buffer_pipeline"]
        T6["decodes_flac_when_ffmpeg_is_available"]
        T7["runs_json_plan"]
        T8["synthesizes_audio_with_effects"]
    end

    subgraph "Exercised Paths"
        P1["legacy parse → gain/trim/pad/speed/rate/lowpass/highpass/limiter"]
        P2["formats subcommand"]
        P3["info --json → Report serialization"]
        P4["concat + mix subcommands"]
        P5["stream → gain/fade/limiter pipeline"]
        P6["symphonia FLAC decode path"]
        P7["run-plan → JSON parse → effects → channels"]
        P8["synth → triangle waveform → fade → stat-json"]
    end

    T1 --> P1
    T2 --> P2
    T3 --> P3
    T4 --> P4
    T5 --> P5
    T6 --> P6
    T7 --> P7
    T8 --> P8
```

## 9. File Inventory

| File | Lines | Role |
|------|-------|------|
| `src/main.rs` | 496 | Entry point, command dispatch, batch/concat/mix/plan orchestration, effect token parser |
| `src/cli.rs` | 368 | Clap derive structs, legacy SoX arg parser, JSON plan deserialization |
| `src/audio.rs` | 242 | `AudioBuffer` + `AudioSpec`, WAV read/write via hound, Symphonia multi-codec decode |
| `src/effects.rs` | 403 | `Effect` enum (14 variants), `EffectChain`, all DSP implementations |
| `src/stats.rs` | 58 | `Report` struct with peak/RMS/clipping, text + JSON display |
| `src/streaming.rs` | 173 | Incremental WAV-to-WAV pipeline (gain, fade, limiter) |
| `src/synth.rs` | 79 | Waveform generation (sine, square, triangle, saw, noise, silence) |
| `tests/cli.rs` | 255 | 8 integration tests covering all major command paths |
| `Cargo.toml` | 16 | 7 dependencies, Rust 2024 edition |
| **Total src** | **1,819** | |
