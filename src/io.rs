use crate::audio::AudioBuffer;
use crate::effects::EffectChain;
use crate::stats;
use anyhow::{Context, Result};
use glob::glob;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Read an input file, apply the effect chain, and write the output.
pub fn convert_one(
    input: &Path,
    output: &Path,
    chain: &EffectChain,
    stat_json: bool,
) -> Result<()> {
    let mut audio =
        AudioBuffer::read(input).with_context(|| format!("failed to read {}", input.display()))?;
    chain.apply(&mut audio)?;
    write_output(&audio, output, stat_json || chain.wants_stats(), stat_json)
}

/// Write an `AudioBuffer` to a WAV file, optionally printing stats.
pub fn write_output(
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

/// Expand glob patterns into a sorted, deduplicated list of file paths.
pub fn expand_inputs(patterns: &[PathBuf]) -> Result<Vec<PathBuf>> {
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

/// Build the output path for a batch item: `out_dir/{stem}.{extension}`
pub fn output_for_batch(input: &Path, out_dir: &Path, extension: &str) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("output");
    out_dir.join(format!("{stem}.{extension}"))
}
