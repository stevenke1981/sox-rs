use crate::audio::{AudioBuffer, AudioSpec};
use serde::Serialize;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct Report {
    pub path: PathBuf,
    pub spec: AudioSpec,
    pub frames: usize,
    pub duration_seconds: f64,
    pub peak: f32,
    pub rms: f32,
    pub clipped_samples: usize,
}

impl Report {
    pub fn from_audio(path: &Path, audio: &AudioBuffer) -> Self {
        let sum_squares = audio
            .samples
            .iter()
            .map(|sample| f64::from(*sample) * f64::from(*sample))
            .sum::<f64>();
        let rms = if audio.samples.is_empty() {
            0.0
        } else {
            (sum_squares / audio.samples.len() as f64).sqrt() as f32
        };
        let clipped_samples = audio
            .samples
            .iter()
            .filter(|sample| sample.abs() >= 1.0)
            .count();

        Self {
            path: path.to_path_buf(),
            spec: audio.spec,
            frames: audio.frames(),
            duration_seconds: audio.duration_seconds(),
            peak: audio.peak(),
            rms,
            clipped_samples,
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "File: {}", self.path.display())?;
        writeln!(f, "Channels: {}", self.spec.channels)?;
        writeln!(f, "Sample Rate: {} Hz", self.spec.sample_rate)?;
        writeln!(f, "Frames: {}", self.frames)?;
        writeln!(f, "Duration: {:.3} s", self.duration_seconds)?;
        writeln!(f, "Peak: {:.6}", self.peak)?;
        writeln!(f, "RMS: {:.6}", self.rms)?;
        write!(f, "Clipped Samples: {}", self.clipped_samples)
    }
}
