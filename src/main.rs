mod audio;
mod cli;
mod effects;
mod stats;
mod streaming;
mod synth;

use anyhow::{Context, Result, anyhow, bail};
use audio::AudioBuffer;
use clap::Parser;
use cli::{
    BatchArgs, Cli, Commands, ConcatArgs, ConvertArgs, InfoArgs, LegacyCommand, MixArgs, PlanMode,
    ProcessingPlan, RunPlanArgs, StreamArgs, SynthArgs,
};
use effects::{Effect, EffectChain};
use glob::glob;
use rayon::prelude::*;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let legacy = LegacyCommand::from_env()?;
    if let Some(command) = legacy {
        return run_legacy(command);
    }

    let cli = Cli::parse();
    match cli.command {
        Commands::Info(args) => run_info(args),
        Commands::Formats => run_formats(),
        Commands::Convert(args) => run_convert(args),
        Commands::Concat(args) => run_concat(args),
        Commands::Mix(args) => run_mix(args),
        Commands::Synth(args) => run_synth(args),
        Commands::Stream(args) => run_stream(args),
        Commands::Batch(args) => run_batch(args),
        Commands::RunPlan(args) => run_plan(args),
    }
}

fn run_formats() -> Result<()> {
    println!("read: wav, flac, mp3, ogg/vorbis, opus, aac, alac, caf, mkv/webm, mp4/m4a");
    println!("write: wav (16-bit PCM)");
    println!("stream: wav -> wav for gain, fade, and limiter without loading the full file");
    Ok(())
}

fn run_info(args: InfoArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.inputs.len());
    for input in args.inputs {
        let audio = AudioBuffer::read(&input)
            .with_context(|| format!("failed to read {}", input.display()))?;
        reports.push(stats::Report::from_audio(&input, &audio));
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&reports)?);
    } else {
        for report in reports {
            println!("{report}");
        }
    }

    Ok(())
}

fn run_convert(args: ConvertArgs) -> Result<()> {
    let chain = args.effect_chain()?;
    convert_one(&args.input, &args.output, &chain, args.stat_json)
}

fn run_concat(args: ConcatArgs) -> Result<()> {
    let mut audio = concat_inputs(&args.inputs)?;
    if args.normalize {
        EffectChain::new(vec![Effect::Normalize { target_db: -1.0 }]).apply(&mut audio)?;
    }
    write_output(&audio, &args.output, args.stat_json, args.stat_json)
}

fn run_mix(args: MixArgs) -> Result<()> {
    let mut audio = mix_inputs(&args.inputs)?;
    if args.normalize {
        EffectChain::new(vec![Effect::Normalize { target_db: -1.0 }]).apply(&mut audio)?;
    }
    write_output(&audio, &args.output, args.stat_json, args.stat_json)
}

fn run_synth(args: SynthArgs) -> Result<()> {
    let mut audio = synth::render(&args)?;
    let chain = args.effect_chain()?;
    chain.apply(&mut audio)?;
    write_output(
        &audio,
        &args.output,
        args.stat_json || chain.wants_stats(),
        args.stat_json,
    )
}

fn run_stream(args: StreamArgs) -> Result<()> {
    streaming::stream_wav(&args)
}

fn run_batch(args: BatchArgs) -> Result<()> {
    let inputs = expand_inputs(&args.inputs)?;
    if inputs.is_empty() {
        bail!("no input files matched");
    }

    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("failed to create {}", args.out_dir.display()))?;

    let chain = args.effect_chain()?;
    let extension = args.ext.trim_start_matches('.').to_string();
    let failures: Vec<_> = inputs
        .par_iter()
        .filter_map(|input| {
            let output = output_for_batch(input, &args.out_dir, &extension);
            convert_one(input, &output, &chain, args.stat_json)
                .err()
                .map(|err| format!("{}: {err:#}", input.display()))
        })
        .collect();

    if !failures.is_empty() {
        for failure in &failures {
            eprintln!("{failure}");
        }
        bail!("{} batch item(s) failed", failures.len());
    }

    Ok(())
}

fn run_legacy(command: LegacyCommand) -> Result<()> {
    let chain = EffectChain::new(command.effects);
    convert_one(&command.input, &command.output, &chain, command.stat_json)
}

fn run_plan(args: RunPlanArgs) -> Result<()> {
    let text = std::fs::read_to_string(&args.plan)
        .with_context(|| format!("failed to read {}", args.plan.display()))?;
    let plan: ProcessingPlan = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse {}", args.plan.display()))?;
    let (effects, effect_stat_json) = parse_effects(&plan.effects)?;
    let chain = EffectChain::new(effects);
    let stat_json = plan.stat_json || effect_stat_json;

    match plan.mode {
        PlanMode::Convert => {
            let input = plan
                .inputs
                .first()
                .ok_or_else(|| anyhow!("convert plan requires at least one input"))?;
            convert_one(input, &plan.output, &chain, stat_json)
        }
        PlanMode::Concat => {
            let mut audio = concat_inputs(&plan.inputs)?;
            chain.apply(&mut audio)?;
            if plan.normalize {
                EffectChain::new(vec![Effect::Normalize { target_db: -1.0 }]).apply(&mut audio)?;
            }
            write_output(
                &audio,
                &plan.output,
                stat_json || chain.wants_stats(),
                stat_json,
            )
        }
        PlanMode::Mix => {
            let mut audio = mix_inputs(&plan.inputs)?;
            chain.apply(&mut audio)?;
            if plan.normalize {
                EffectChain::new(vec![Effect::Normalize { target_db: -1.0 }]).apply(&mut audio)?;
            }
            write_output(
                &audio,
                &plan.output,
                stat_json || chain.wants_stats(),
                stat_json,
            )
        }
    }
}

fn convert_one(input: &Path, output: &Path, chain: &EffectChain, stat_json: bool) -> Result<()> {
    let mut audio =
        AudioBuffer::read(input).with_context(|| format!("failed to read {}", input.display()))?;
    chain.apply(&mut audio)?;
    write_output(&audio, output, stat_json || chain.wants_stats(), stat_json)
}

fn write_output(
    audio: &AudioBuffer,
    output: &Path,
    print_stats: bool,
    stat_json: bool,
) -> Result<()> {
    audio
        .write_wav(output)
        .with_context(|| format!("failed to write {}", output.display()))?;

    if print_stats {
        let report = stats::Report::from_audio(output, audio);
        if stat_json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("{report}");
        }
    }

    Ok(())
}

fn concat_inputs(inputs: &[PathBuf]) -> Result<AudioBuffer> {
    if inputs.is_empty() {
        bail!("concat requires at least one input");
    }

    let mut iter = inputs.iter();
    let first_path = iter.next().expect("checked non-empty");
    let mut output = AudioBuffer::read(first_path)
        .with_context(|| format!("failed to read {}", first_path.display()))?;

    for input in iter {
        let audio = AudioBuffer::read(input)
            .with_context(|| format!("failed to read {}", input.display()))?;
        ensure_same_format(&output, &audio, input)?;
        output.samples.extend_from_slice(&audio.samples);
    }

    Ok(output)
}

fn mix_inputs(inputs: &[PathBuf]) -> Result<AudioBuffer> {
    if inputs.is_empty() {
        bail!("mix requires at least one input");
    }

    let mut buffers = Vec::with_capacity(inputs.len());
    for input in inputs {
        buffers.push(
            AudioBuffer::read(input)
                .with_context(|| format!("failed to read {}", input.display()))?,
        );
    }

    let sample_rate = buffers[0].spec.sample_rate;
    if let Some((index, bad)) = buffers
        .iter()
        .enumerate()
        .find(|(_, audio)| audio.spec.sample_rate != sample_rate)
    {
        bail!(
            "mix input {} has sample rate {}, expected {}",
            inputs[index].display(),
            bad.spec.sample_rate,
            sample_rate
        );
    }

    let channels = buffers
        .iter()
        .map(|audio| audio.spec.channels)
        .max()
        .unwrap_or(1);
    let frames = buffers.iter().map(AudioBuffer::frames).max().unwrap_or(0);
    let channel_count = usize::from(channels);
    let mut samples = vec![0.0; frames * channel_count];

    for audio in &buffers {
        for frame in 0..frames {
            for channel in 0..channel_count {
                samples[frame * channel_count + channel] += sample_at(audio, frame, channel);
            }
        }
    }

    let scale = buffers.len() as f32;
    for sample in &mut samples {
        *sample /= scale;
    }

    Ok(AudioBuffer {
        spec: audio::AudioSpec {
            sample_rate,
            channels,
        },
        samples,
    })
}

fn ensure_same_format(reference: &AudioBuffer, audio: &AudioBuffer, path: &Path) -> Result<()> {
    if reference.spec.sample_rate != audio.spec.sample_rate
        || reference.spec.channels != audio.spec.channels
    {
        bail!(
            "{} has {} Hz / {} channel(s), expected {} Hz / {} channel(s)",
            path.display(),
            audio.spec.sample_rate,
            audio.spec.channels,
            reference.spec.sample_rate,
            reference.spec.channels
        );
    }
    Ok(())
}

fn sample_at(audio: &AudioBuffer, frame: usize, channel: usize) -> f32 {
    if frame >= audio.frames() {
        return 0.0;
    }

    let source_channels = usize::from(audio.spec.channels);
    let source_channel = if source_channels == 1 {
        0
    } else {
        channel.min(source_channels - 1)
    };
    audio.samples[frame * source_channels + source_channel]
}

fn expand_inputs(patterns: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut inputs = Vec::new();
    for pattern in patterns {
        let text = pattern.to_string_lossy();
        if text.contains('*') || text.contains('?') || text.contains('[') {
            for entry in glob(&text).with_context(|| format!("invalid glob {text}"))? {
                inputs.push(entry.with_context(|| format!("failed to expand glob {text}"))?);
            }
        } else {
            inputs.push(pattern.clone());
        }
    }
    inputs.sort();
    inputs.dedup();
    Ok(inputs)
}

fn output_for_batch(input: &Path, out_dir: &Path, extension: &str) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("output");
    out_dir.join(format!("{stem}.{extension}"))
}

pub(crate) fn parse_effects(tokens: &[String]) -> Result<(Vec<Effect>, bool)> {
    let mut effects = Vec::new();
    let mut stat_json = false;
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].as_str() {
            "gain" | "vol" => {
                let db = parse_next_f32(tokens, &mut i, "gain requires <db>")?;
                effects.push(Effect::GainDb(db));
            }
            "norm" | "normalize" => {
                let target_db = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        value
                    }
                    None => -1.0,
                };
                effects.push(Effect::Normalize { target_db });
            }
            "trim" => {
                let start = parse_next_f32(tokens, &mut i, "trim requires <start-sec>")?;
                let duration = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        Some(value)
                    }
                    None => None,
                };
                effects.push(Effect::Trim {
                    start_sec: start,
                    duration_sec: duration,
                });
            }
            "fade" => {
                let in_sec = parse_next_f32(tokens, &mut i, "fade requires <in-sec>")?;
                let out_sec = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        value
                    }
                    None => 0.0,
                };
                effects.push(Effect::Fade { in_sec, out_sec });
            }
            "reverse" | "reverse-samples" => effects.push(Effect::Reverse),
            "speed" => {
                let factor = parse_next_f32(tokens, &mut i, "speed requires <factor>")?;
                effects.push(Effect::Speed { factor });
            }
            "pad" => {
                let start_sec = parse_next_f32(tokens, &mut i, "pad requires <start-sec>")?;
                let end_sec = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        value
                    }
                    None => 0.0,
                };
                effects.push(Effect::Pad { start_sec, end_sec });
            }
            "silence" => {
                let threshold_db =
                    parse_next_f32(tokens, &mut i, "silence requires <threshold-db>")?;
                let min_duration_sec = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        value
                    }
                    None => 0.1,
                };
                effects.push(Effect::Silence {
                    threshold_db,
                    min_duration_sec,
                });
            }
            "lowpass" => {
                let hz = parse_next_f32(tokens, &mut i, "lowpass requires <hz>")?;
                effects.push(Effect::LowPass { hz });
            }
            "highpass" => {
                let hz = parse_next_f32(tokens, &mut i, "highpass requires <hz>")?;
                effects.push(Effect::HighPass { hz });
            }
            "limiter" => {
                let threshold = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        value
                    }
                    None => 0.95,
                };
                effects.push(Effect::Limiter { threshold });
            }
            "rate" | "resample" => {
                let sample_rate = parse_next_u32(tokens, &mut i, "rate requires <hz>")?;
                effects.push(Effect::Rate { sample_rate });
            }
            "channels" | "ch" => {
                let channels = parse_next_u16(tokens, &mut i, "channels requires <count>")?;
                effects.push(Effect::Channels { channels });
            }
            "stat" | "stats" => effects.push(Effect::Stats),
            "stat-json" | "stats-json" => {
                stat_json = true;
                effects.push(Effect::Stats);
            }
            unknown => return Err(anyhow!("unsupported effect '{unknown}'")),
        }
        i += 1;
    }

    Ok((effects, stat_json))
}

fn parse_next_f32(tokens: &[String], i: &mut usize, message: &str) -> Result<f32> {
    *i += 1;
    tokens
        .get(*i)
        .ok_or_else(|| anyhow!(message.to_string()))?
        .parse::<f32>()
        .with_context(|| message.to_string())
}

fn parse_next_u32(tokens: &[String], i: &mut usize, message: &str) -> Result<u32> {
    *i += 1;
    tokens
        .get(*i)
        .ok_or_else(|| anyhow!(message.to_string()))?
        .parse::<u32>()
        .with_context(|| message.to_string())
}

fn parse_next_u16(tokens: &[String], i: &mut usize, message: &str) -> Result<u16> {
    *i += 1;
    tokens
        .get(*i)
        .ok_or_else(|| anyhow!(message.to_string()))?
        .parse::<u16>()
        .with_context(|| message.to_string())
}
