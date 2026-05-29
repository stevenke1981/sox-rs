use crate::cli::StreamArgs;
use anyhow::{Context, Result, bail};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

pub fn stream_wav(args: &StreamArgs) -> Result<()> {
    if !(0.0..=1.0).contains(&args.limiter) {
        bail!("stream limiter must be between 0 and 1");
    }
    if args.fade_in.is_some_and(|value| value < 0.0)
        || args.fade_out.is_some_and(|value| value < 0.0)
    {
        bail!("stream fade durations must be >= 0");
    }

    let mut reader = WavReader::open(&args.input)
        .with_context(|| format!("failed to open {}", args.input.display()))?;
    let input_spec = reader.spec();
    if input_spec.channels == 0 {
        bail!("WAV file has zero channels");
    }

    let output_spec = WavSpec {
        channels: input_spec.channels,
        sample_rate: input_spec.sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    if let Some(parent) = args
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut writer = WavWriter::create(&args.output, output_spec)
        .with_context(|| format!("failed to create {}", args.output.display()))?;

    let channels = u32::from(input_spec.channels);
    let total_frames = reader.duration() / channels;
    let gain = args
        .gain_db
        .map(|db| 10.0_f32.powf(db / 20.0))
        .unwrap_or(1.0);
    let fade_in_frames = seconds_to_frames(args.fade_in.unwrap_or(0.0), input_spec.sample_rate);
    let fade_out_frames = seconds_to_frames(args.fade_out.unwrap_or(0.0), input_spec.sample_rate);

    match input_spec.sample_format {
        SampleFormat::Float => stream_samples::<f32, _>(
            reader.samples::<f32>(),
            &mut writer,
            StreamSettings {
                channels,
                total_frames,
                gain,
                fade_in_frames,
                fade_out_frames,
                limiter: args.limiter,
            },
            |value| value,
        )?,
        SampleFormat::Int => match input_spec.bits_per_sample {
            0 => bail!("WAV file has zero bits per sample"),
            1..=8 => {
                let max = ((1_i32 << (input_spec.bits_per_sample - 1)) - 1) as f32;
                stream_samples::<i8, _>(
                    reader.samples::<i8>(),
                    &mut writer,
                    StreamSettings {
                        channels,
                        total_frames,
                        gain,
                        fade_in_frames,
                        fade_out_frames,
                        limiter: args.limiter,
                    },
                    |value| value as f32 / max,
                )?
            }
            9..=16 => {
                let max = ((1_i32 << (input_spec.bits_per_sample - 1)) - 1) as f32;
                stream_samples::<i16, _>(
                    reader.samples::<i16>(),
                    &mut writer,
                    StreamSettings {
                        channels,
                        total_frames,
                        gain,
                        fade_in_frames,
                        fade_out_frames,
                        limiter: args.limiter,
                    },
                    |value| value as f32 / max,
                )?
            }
            17..=32 => {
                let max = ((1_i64 << (input_spec.bits_per_sample - 1)) - 1) as f32;
                stream_samples::<i32, _>(
                    reader.samples::<i32>(),
                    &mut writer,
                    StreamSettings {
                        channels,
                        total_frames,
                        gain,
                        fade_in_frames,
                        fade_out_frames,
                        limiter: args.limiter,
                    },
                    |value| value as f32 / max,
                )?
            }
            bits => bail!("unsupported integer WAV depth: {bits}"),
        },
    }

    writer.finalize()?;
    Ok(())
}

#[derive(Clone, Copy)]
struct StreamSettings {
    channels: u32,
    total_frames: u32,
    gain: f32,
    fade_in_frames: u32,
    fade_out_frames: u32,
    limiter: f32,
}

fn stream_samples<T, F>(
    samples: hound::WavSamples<'_, std::io::BufReader<std::fs::File>, T>,
    writer: &mut WavWriter<std::io::BufWriter<std::fs::File>>,
    settings: StreamSettings,
    to_f32: F,
) -> Result<()>
where
    T: hound::Sample,
    F: Fn(T) -> f32,
{
    for (sample_index, sample) in samples.enumerate() {
        let frame = sample_index as u32 / settings.channels;
        let mut value = to_f32(sample?) * settings.gain;
        value *= fade_factor(
            frame,
            settings.total_frames,
            settings.fade_in_frames,
            settings.fade_out_frames,
        );
        value = value.clamp(-settings.limiter, settings.limiter);
        writer.write_sample((value.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16)?;
    }

    Ok(())
}

fn fade_factor(frame: u32, total_frames: u32, fade_in_frames: u32, fade_out_frames: u32) -> f32 {
    let fade_in = if fade_in_frames > 0 {
        (frame as f32 / fade_in_frames as f32).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let fade_out = if fade_out_frames > 0 && total_frames > 0 {
        let frames_from_end = total_frames.saturating_sub(frame + 1);
        (frames_from_end as f32 / fade_out_frames as f32).clamp(0.0, 1.0)
    } else {
        1.0
    };
    fade_in.min(fade_out)
}

fn seconds_to_frames(seconds: f32, sample_rate: u32) -> u32 {
    (seconds as f64 * sample_rate as f64).round().max(0.0) as u32
}
