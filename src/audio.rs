use anyhow::{Context, Result, anyhow, bail};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use serde::Serialize;
use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct AudioSpec {
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub spec: AudioSpec,
    pub samples: Vec<f32>,
}

impl AudioBuffer {
    pub fn read(path: &Path) -> Result<Self> {
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
        {
            return Self::read_wav(path);
        }

        Self::read_with_symphonia(path)
    }

    pub fn read_wav(path: &Path) -> Result<Self> {
        let mut reader =
            WavReader::open(path).with_context(|| "unsupported or invalid WAV file".to_string())?;
        let spec = reader.spec();
        if spec.channels == 0 {
            bail!("WAV file has zero channels");
        }

        let samples = match spec.sample_format {
            SampleFormat::Float => reader
                .samples::<f32>()
                .map(|sample| {
                    sample
                        .map(|value| value.clamp(-1.0, 1.0))
                        .map_err(anyhow::Error::from)
                })
                .collect::<Result<Vec<_>>>()?,
            SampleFormat::Int => read_int_samples(&mut reader, spec.bits_per_sample)?,
        };

        Ok(Self {
            spec: AudioSpec {
                sample_rate: spec.sample_rate,
                channels: spec.channels,
            },
            samples,
        })
    }

    pub fn write_wav(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let spec = WavSpec {
            channels: self.spec.channels,
            sample_rate: self.spec.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(path, spec)?;
        for sample in &self.samples {
            let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
            writer.write_sample(value)?;
        }
        writer.finalize()?;
        Ok(())
    }

    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.spec.channels)
    }

    pub fn duration_seconds(&self) -> f64 {
        self.frames() as f64 / self.spec.sample_rate as f64
    }

    pub fn peak(&self) -> f32 {
        self.samples
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0_f32, f32::max)
    }

    fn read_with_symphonia(path: &Path) -> Result<Self> {
        let source = Box::new(File::open(path)?);
        let media = MediaSourceStream::new(source, Default::default());
        let mut hint = Hint::new();
        if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
            hint.with_extension(extension);
        }

        let probed = symphonia::default::get_probe().format(
            &hint,
            media,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;
        let mut format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|track| track.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow!("no supported audio track found"))?
            .clone();
        let sample_rate = track
            .codec_params
            .sample_rate
            .ok_or_else(|| anyhow!("codec did not report a sample rate"))?;
        let channels = track
            .codec_params
            .channels
            .ok_or_else(|| anyhow!("codec did not report a channel layout"))?
            .count() as u16;

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())?;
        let mut samples = Vec::new();
        let mut sample_buffer: Option<SampleBuffer<f32>> = None;

        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(err))
                    if err.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(SymphoniaError::ResetRequired) => bail!("codec reset required"),
                Err(err) => return Err(err.into()),
            };

            if packet.track_id() != track.id {
                continue;
            }

            let decoded = match decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(err) => return Err(err.into()),
            };

            if sample_buffer
                .as_ref()
                .is_none_or(|buffer| buffer.capacity() < decoded.capacity())
            {
                sample_buffer = Some(SampleBuffer::<f32>::new(
                    decoded.capacity() as u64,
                    *decoded.spec(),
                ));
            }

            let buffer = sample_buffer.as_mut().expect("sample buffer initialized");
            buffer.copy_interleaved_ref(decoded);
            samples.extend_from_slice(buffer.samples());
        }

        Ok(Self {
            spec: AudioSpec {
                sample_rate,
                channels,
            },
            samples,
        })
    }
}

fn read_int_samples(
    reader: &mut WavReader<std::io::BufReader<std::fs::File>>,
    bits: u16,
) -> Result<Vec<f32>> {
    match bits {
        0 => bail!("WAV file has zero bits per sample"),
        1..=8 => {
            let max = ((1_i32 << (bits - 1)) - 1) as f32;
            reader
                .samples::<i8>()
                .map(|sample| {
                    sample
                        .map(|value| value as f32 / max)
                        .map_err(anyhow::Error::from)
                })
                .collect()
        }
        9..=16 => {
            let max = ((1_i32 << (bits - 1)) - 1) as f32;
            reader
                .samples::<i16>()
                .map(|sample| {
                    sample
                        .map(|value| value as f32 / max)
                        .map_err(anyhow::Error::from)
                })
                .collect()
        }
        17..=24 => {
            let max = ((1_i64 << (bits - 1)) - 1) as f32;
            reader
                .samples::<i32>()
                .map(|sample| {
                    sample
                        .map(|value| value as f32 / max)
                        .map_err(anyhow::Error::from)
                })
                .collect()
        }
        25..=32 => {
            let max = ((1_i64 << (bits - 1)) - 1) as f32;
            reader
                .samples::<i32>()
                .map(|sample| {
                    sample
                        .map(|value| value as f32 / max)
                        .map_err(anyhow::Error::from)
                })
                .collect()
        }
        _ => Err(anyhow!("unsupported integer WAV depth: {bits}")),
    }
}
