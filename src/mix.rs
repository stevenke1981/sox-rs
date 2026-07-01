use crate::audio::{AudioBuffer, AudioSpec};
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// Concatenate audio files in order.
/// All inputs must have the same sample rate and channel count.
pub fn concat_inputs(inputs: &[PathBuf]) -> Result<AudioBuffer> {
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

/// Mix audio files by summing samples (no automatic averaging).
/// Files with different sample rates will be rejected.
/// Files with different channel counts will up-mix to the maximum channel count.
pub fn mix_inputs(inputs: &[PathBuf]) -> Result<AudioBuffer> {
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

    Ok(AudioBuffer {
        spec: AudioSpec {
            sample_rate,
            channels,
        },
        samples,
    })
}

/// Get the sample value at a given frame and channel.
/// Returns 0.0 if the frame is beyond the buffer's length.
/// If the source has fewer channels, the nearest available channel is used.
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
