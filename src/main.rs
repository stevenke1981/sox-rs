mod audio;
mod cli;
mod effects;
mod io;
mod mix;
mod parse;
mod stats;
mod streaming;
mod synth;
mod util;

use anyhow::{Context, Result, anyhow, bail};
use audio::AudioBuffer;
use clap::Parser;
use cli::{
    BatchArgs, Cli, Commands, ConcatArgs, ConvertArgs, InfoArgs, LegacyCommand, MixArgs, PlanMode,
    ProcessingPlan, RunPlanArgs, StreamArgs, SynthArgs,
};
use effects::{Effect, EffectChain};
use parse::parse_effects;
use rayon::prelude::*;

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
    io::convert_one(&args.input, &args.output, &chain, args.stat_json)
}

fn run_concat(args: ConcatArgs) -> Result<()> {
    let mut audio = mix::concat_inputs(&args.inputs)?;
    if args.normalize {
        EffectChain::new(vec![Effect::Normalize { target_db: -1.0 }]).apply(&mut audio)?;
    }
    io::write_output(&audio, &args.output, args.stat_json, args.stat_json)
}

fn run_mix(args: MixArgs) -> Result<()> {
    let mut audio = mix::mix_inputs(&args.inputs)?;
    if args.normalize {
        EffectChain::new(vec![Effect::Normalize { target_db: -1.0 }]).apply(&mut audio)?;
    }
    io::write_output(&audio, &args.output, args.stat_json, args.stat_json)
}

fn run_synth(args: SynthArgs) -> Result<()> {
    let mut audio = synth::render(&args)?;
    let chain = args.effect_chain()?;
    chain.apply(&mut audio)?;
    io::write_output(
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
    let inputs = io::expand_inputs(&args.inputs)?;
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
            let output = io::output_for_batch(input, &args.out_dir, &extension);
            io::convert_one(input, &output, &chain, args.stat_json)
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
    io::convert_one(&command.input, &command.output, &chain, command.stat_json)
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
            io::convert_one(input, &plan.output, &chain, stat_json)
        }
        PlanMode::Concat => {
            let mut audio = mix::concat_inputs(&plan.inputs)?;
            chain.apply(&mut audio)?;
            if plan.normalize {
                EffectChain::new(vec![Effect::Normalize { target_db: -1.0 }]).apply(&mut audio)?;
            }
            io::write_output(
                &audio,
                &plan.output,
                stat_json || chain.wants_stats(),
                stat_json,
            )
        }
        PlanMode::Mix => {
            let mut audio = mix::mix_inputs(&plan.inputs)?;
            chain.apply(&mut audio)?;
            if plan.normalize {
                EffectChain::new(vec![Effect::Normalize { target_db: -1.0 }]).apply(&mut audio)?;
            }
            io::write_output(
                &audio,
                &plan.output,
                stat_json || chain.wants_stats(),
                stat_json,
            )
        }
    }
}
