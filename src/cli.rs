use crate::effects::EffectChain;
use crate::parse::parse_effects;
use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Print WAV metadata and level statistics.
    Info(InfoArgs),
    /// List built-in format support.
    Formats,
    /// Convert one input file into one output file.
    Convert(ConvertArgs),
    /// Concatenate WAV files in order.
    Concat(ConcatArgs),
    /// Mix WAV files into one output.
    Mix(MixArgs),
    /// Generate audio from built-in waveforms.
    Synth(SynthArgs),
    /// Process WAV in a streaming path for very large files.
    Stream(StreamArgs),
    /// Convert many files in parallel.
    Batch(BatchArgs),
    /// Run a repeatable JSON processing plan.
    RunPlan(RunPlanArgs),
}

#[derive(Debug, Args)]
pub struct InfoArgs {
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    pub input: PathBuf,
    pub output: PathBuf,
    #[arg(long)]
    pub gain_db: Option<f32>,
    #[arg(long)]
    pub normalize: bool,
    #[arg(long, default_value_t = -1.0)]
    pub normalize_db: f32,
    #[arg(long, num_args = 1..=2, value_names = ["START", "DURATION"])]
    pub trim: Vec<f32>,
    #[arg(long, num_args = 1..=2, value_names = ["IN", "OUT"])]
    pub fade: Vec<f32>,
    #[arg(long)]
    pub reverse: bool,
    #[arg(long)]
    pub speed: Option<f32>,
    #[arg(long)]
    pub lowpass: Option<f32>,
    #[arg(long)]
    pub highpass: Option<f32>,
    #[arg(long)]
    pub limiter: Option<f32>,
    #[arg(long, num_args = 1..=2, value_names = ["START", "END"])]
    pub pad: Vec<f32>,
    #[arg(long, num_args = 1..=2, value_names = ["THRESHOLD_DB", "MIN_SECONDS"])]
    pub silence: Vec<f32>,
    #[arg(long)]
    pub rate: Option<u32>,
    #[arg(long)]
    pub channels: Option<u16>,
    #[arg(long)]
    pub stat_json: bool,
}

#[derive(Debug, Args)]
pub struct BatchArgs {
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,
    #[arg(long)]
    pub out_dir: PathBuf,
    #[arg(long, default_value = "wav")]
    pub ext: String,
    #[arg(long)]
    pub gain_db: Option<f32>,
    #[arg(long)]
    pub normalize: bool,
    #[arg(long, default_value_t = -1.0)]
    pub normalize_db: f32,
    #[arg(long)]
    pub rate: Option<u32>,
    #[arg(long)]
    pub channels: Option<u16>,
    #[arg(long)]
    pub speed: Option<f32>,
    #[arg(long)]
    pub lowpass: Option<f32>,
    #[arg(long)]
    pub highpass: Option<f32>,
    #[arg(long)]
    pub limiter: Option<f32>,
    #[arg(long)]
    pub stat_json: bool,
}

#[derive(Debug)]
pub struct LegacyCommand {
    pub input: PathBuf,
    pub output: PathBuf,
    pub effects: Vec<crate::effects::Effect>,
    pub stat_json: bool,
}

#[derive(Debug, Args)]
pub struct ConcatArgs {
    #[arg(required = true, num_args = 2..)]
    pub inputs: Vec<PathBuf>,
    #[arg(short, long)]
    pub output: PathBuf,
    #[arg(long)]
    pub normalize: bool,
    #[arg(long)]
    pub stat_json: bool,
}

#[derive(Debug, Args)]
pub struct MixArgs {
    #[arg(required = true, num_args = 2..)]
    pub inputs: Vec<PathBuf>,
    #[arg(short, long)]
    pub output: PathBuf,
    #[arg(long)]
    pub normalize: bool,
    #[arg(long)]
    pub stat_json: bool,
}

#[derive(Debug, Args)]
pub struct RunPlanArgs {
    pub plan: PathBuf,
}

#[derive(Debug, Args)]
pub struct SynthArgs {
    pub output: PathBuf,
    #[arg(long, default_value_t = 1.0)]
    pub duration: f32,
    #[arg(long, default_value_t = 440.0)]
    pub freq: f32,
    #[arg(long, default_value_t = 44_100)]
    pub rate: u32,
    #[arg(long, default_value_t = 1)]
    pub channels: u16,
    #[arg(long, default_value_t = 0.5)]
    pub amplitude: f32,
    #[arg(long, value_enum, default_value_t = Waveform::Sine)]
    pub waveform: Waveform,
    #[arg(long)]
    pub gain_db: Option<f32>,
    #[arg(long)]
    pub normalize: bool,
    #[arg(long)]
    pub fade: Option<f32>,
    #[arg(long)]
    pub stat_json: bool,
}

#[derive(Debug, Args)]
pub struct StreamArgs {
    pub input: PathBuf,
    pub output: PathBuf,
    #[arg(long)]
    pub gain_db: Option<f32>,
    #[arg(long)]
    pub fade_in: Option<f32>,
    #[arg(long)]
    pub fade_out: Option<f32>,
    #[arg(long, default_value_t = 1.0)]
    pub limiter: f32,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Waveform {
    Sine,
    Square,
    Triangle,
    Saw,
    Noise,
    Silence,
}

#[derive(Debug, Deserialize)]
pub struct ProcessingPlan {
    #[serde(default)]
    pub mode: PlanMode,
    pub inputs: Vec<PathBuf>,
    pub output: PathBuf,
    #[serde(default)]
    pub effects: Vec<String>,
    #[serde(default)]
    pub normalize: bool,
    #[serde(default)]
    pub stat_json: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanMode {
    #[default]
    Convert,
    Concat,
    Mix,
}

impl LegacyCommand {
    pub fn from_env() -> Result<Option<Self>> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        if args.is_empty() || is_subcommand(&args[0]) || args[0].starts_with('-') {
            return Ok(None);
        }

        if args.len() < 2 {
            bail!("SoX-style usage requires <input> <output> [effect ...]");
        }

        let input = PathBuf::from(&args[0]);
        let output = PathBuf::from(&args[1]);
        let (effects, stat_json) = parse_effects(&args[2..])?;
        Ok(Some(Self {
            input,
            output,
            effects,
            stat_json,
        }))
    }
}

impl ConvertArgs {
    pub fn effect_chain(&self) -> Result<EffectChain> {
        let mut effects = Vec::new();
        if let Some(db) = self.gain_db {
            effects.push(crate::effects::Effect::GainDb(db));
        }
        if self.normalize {
            effects.push(crate::effects::Effect::Normalize {
                target_db: self.normalize_db,
            });
        }
        if !self.trim.is_empty() {
            effects.push(crate::effects::Effect::Trim {
                start_sec: self.trim[0],
                duration_sec: self.trim.get(1).copied(),
            });
        }
        if !self.fade.is_empty() {
            effects.push(crate::effects::Effect::Fade {
                in_sec: self.fade[0],
                out_sec: self.fade.get(1).copied().unwrap_or(0.0),
            });
        }
        if self.reverse {
            effects.push(crate::effects::Effect::Reverse);
        }
        if let Some(factor) = self.speed {
            effects.push(crate::effects::Effect::Speed { factor });
        }
        if let Some(hz) = self.lowpass {
            effects.push(crate::effects::Effect::LowPass { hz });
        }
        if let Some(hz) = self.highpass {
            effects.push(crate::effects::Effect::HighPass { hz });
        }
        if let Some(threshold) = self.limiter {
            effects.push(crate::effects::Effect::Limiter { threshold });
        }
        if !self.pad.is_empty() {
            effects.push(crate::effects::Effect::Pad {
                start_sec: self.pad[0],
                end_sec: self.pad.get(1).copied().unwrap_or(0.0),
            });
        }
        if !self.silence.is_empty() {
            effects.push(crate::effects::Effect::Silence {
                threshold_db: self.silence[0],
                min_duration_sec: self.silence.get(1).copied().unwrap_or(0.1),
            });
        }
        if let Some(sample_rate) = self.rate {
            effects.push(crate::effects::Effect::Rate { sample_rate });
        }
        if let Some(channels) = self.channels {
            effects.push(crate::effects::Effect::Channels { channels });
        }
        Ok(EffectChain::new(effects))
    }
}

impl BatchArgs {
    pub fn effect_chain(&self) -> Result<EffectChain> {
        let mut effects = Vec::new();
        if let Some(db) = self.gain_db {
            effects.push(crate::effects::Effect::GainDb(db));
        }
        if self.normalize {
            effects.push(crate::effects::Effect::Normalize {
                target_db: self.normalize_db,
            });
        }
        if let Some(sample_rate) = self.rate {
            effects.push(crate::effects::Effect::Rate { sample_rate });
        }
        if let Some(channels) = self.channels {
            effects.push(crate::effects::Effect::Channels { channels });
        }
        if let Some(factor) = self.speed {
            effects.push(crate::effects::Effect::Speed { factor });
        }
        if let Some(hz) = self.lowpass {
            effects.push(crate::effects::Effect::LowPass { hz });
        }
        if let Some(hz) = self.highpass {
            effects.push(crate::effects::Effect::HighPass { hz });
        }
        if let Some(threshold) = self.limiter {
            effects.push(crate::effects::Effect::Limiter { threshold });
        }
        Ok(EffectChain::new(effects))
    }
}

impl SynthArgs {
    pub fn effect_chain(&self) -> Result<EffectChain> {
        let mut effects = Vec::new();
        if let Some(db) = self.gain_db {
            effects.push(crate::effects::Effect::GainDb(db));
        }
        if self.normalize {
            effects.push(crate::effects::Effect::Normalize { target_db: -1.0 });
        }
        if let Some(duration) = self.fade {
            effects.push(crate::effects::Effect::Fade {
                in_sec: duration,
                out_sec: duration,
            });
        }
        Ok(EffectChain::new(effects))
    }
}

fn is_subcommand(value: &str) -> bool {
    matches!(
        value,
        "info"
            | "formats"
            | "convert"
            | "concat"
            | "mix"
            | "synth"
            | "stream"
            | "batch"
            | "run-plan"
            | "help"
    )
}
