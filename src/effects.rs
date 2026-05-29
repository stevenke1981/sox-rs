use crate::audio::AudioBuffer;
use anyhow::{Result, bail};

#[derive(Debug, Clone)]
pub enum Effect {
    GainDb(f32),
    Normalize {
        target_db: f32,
    },
    Trim {
        start_sec: f32,
        duration_sec: Option<f32>,
    },
    Fade {
        in_sec: f32,
        out_sec: f32,
    },
    Reverse,
    Speed {
        factor: f32,
    },
    Pad {
        start_sec: f32,
        end_sec: f32,
    },
    Silence {
        threshold_db: f32,
        min_duration_sec: f32,
    },
    LowPass {
        hz: f32,
    },
    HighPass {
        hz: f32,
    },
    Limiter {
        threshold: f32,
    },
    Rate {
        sample_rate: u32,
    },
    Channels {
        channels: u16,
    },
    Stats,
}

#[derive(Debug, Clone)]
pub struct EffectChain {
    effects: Vec<Effect>,
}

impl EffectChain {
    pub fn new(effects: Vec<Effect>) -> Self {
        Self { effects }
    }

    pub fn apply(&self, audio: &mut AudioBuffer) -> Result<()> {
        for effect in &self.effects {
            match *effect {
                Effect::GainDb(db) => gain_db(audio, db),
                Effect::Normalize { target_db } => normalize(audio, target_db),
                Effect::Trim {
                    start_sec,
                    duration_sec,
                } => trim(audio, start_sec, duration_sec)?,
                Effect::Fade { in_sec, out_sec } => fade(audio, in_sec, out_sec)?,
                Effect::Reverse => reverse(audio),
                Effect::Speed { factor } => speed(audio, factor)?,
                Effect::Pad { start_sec, end_sec } => pad(audio, start_sec, end_sec)?,
                Effect::Silence {
                    threshold_db,
                    min_duration_sec,
                } => trim_silence(audio, threshold_db, min_duration_sec)?,
                Effect::LowPass { hz } => lowpass(audio, hz)?,
                Effect::HighPass { hz } => highpass(audio, hz)?,
                Effect::Limiter { threshold } => limiter(audio, threshold)?,
                Effect::Rate { sample_rate } => rate(audio, sample_rate)?,
                Effect::Channels { channels } => convert_channels(audio, channels)?,
                Effect::Stats => {}
            }
        }
        Ok(())
    }

    pub fn wants_stats(&self) -> bool {
        self.effects
            .iter()
            .any(|effect| matches!(effect, Effect::Stats))
    }
}

fn gain_db(audio: &mut AudioBuffer, db: f32) {
    let factor = 10.0_f32.powf(db / 20.0);
    for sample in &mut audio.samples {
        *sample *= factor;
    }
}

fn normalize(audio: &mut AudioBuffer, target_db: f32) {
    let peak = audio.peak();
    if peak == 0.0 {
        return;
    }
    let target = 10.0_f32.powf(target_db / 20.0);
    let factor = target / peak;
    for sample in &mut audio.samples {
        *sample *= factor;
    }
}

fn trim(audio: &mut AudioBuffer, start_sec: f32, duration_sec: Option<f32>) -> Result<()> {
    if start_sec < 0.0 {
        bail!("trim start must be >= 0");
    }
    if duration_sec.is_some_and(|duration| duration < 0.0) {
        bail!("trim duration must be >= 0");
    }

    let channels = usize::from(audio.spec.channels);
    let start_frame = seconds_to_frames(start_sec, audio.spec.sample_rate);
    let end_frame = duration_sec
        .map(|duration| start_frame + seconds_to_frames(duration, audio.spec.sample_rate))
        .unwrap_or_else(|| audio.frames())
        .min(audio.frames());

    if start_frame >= audio.frames() || start_frame >= end_frame {
        audio.samples.clear();
        return Ok(());
    }

    let start = start_frame * channels;
    let end = end_frame * channels;
    audio.samples = audio.samples[start..end].to_vec();
    Ok(())
}

fn fade(audio: &mut AudioBuffer, in_sec: f32, out_sec: f32) -> Result<()> {
    if in_sec < 0.0 || out_sec < 0.0 {
        bail!("fade durations must be >= 0");
    }
    let channels = usize::from(audio.spec.channels);
    let frames = audio.frames();
    let fade_in_frames = seconds_to_frames(in_sec, audio.spec.sample_rate).min(frames);
    let fade_out_frames = seconds_to_frames(out_sec, audio.spec.sample_rate).min(frames);

    for frame in 0..fade_in_frames {
        let factor = frame as f32 / fade_in_frames.max(1) as f32;
        scale_frame(audio, frame, channels, factor);
    }

    for n in 0..fade_out_frames {
        let frame = frames - fade_out_frames + n;
        let factor = 1.0 - (n as f32 / fade_out_frames.max(1) as f32);
        scale_frame(audio, frame, channels, factor);
    }

    Ok(())
}

fn reverse(audio: &mut AudioBuffer) {
    let channels = usize::from(audio.spec.channels);
    let mut reversed = Vec::with_capacity(audio.samples.len());
    for frame in audio.samples.chunks_exact(channels).rev() {
        reversed.extend_from_slice(frame);
    }
    audio.samples = reversed;
}

fn speed(audio: &mut AudioBuffer, factor: f32) -> Result<()> {
    if factor <= 0.0 {
        bail!("speed factor must be > 0");
    }
    if factor == 1.0 || audio.samples.is_empty() {
        return Ok(());
    }

    let channels = usize::from(audio.spec.channels);
    let old_frames = audio.frames();
    if old_frames <= 1 {
        return Ok(());
    }

    let new_frames = ((old_frames as f64) / factor as f64).round().max(1.0) as usize;
    let mut output = vec![0.0; new_frames * channels];

    for new_frame in 0..new_frames {
        let source_pos = new_frame as f64 * factor as f64;
        let left = source_pos.floor().min((old_frames - 1) as f64) as usize;
        let right = (left + 1).min(old_frames - 1);
        let fraction = (source_pos - left as f64) as f32;

        for channel in 0..channels {
            let a = audio.samples[left * channels + channel];
            let b = audio.samples[right * channels + channel];
            output[new_frame * channels + channel] = a + (b - a) * fraction;
        }
    }

    audio.samples = output;
    Ok(())
}

fn pad(audio: &mut AudioBuffer, start_sec: f32, end_sec: f32) -> Result<()> {
    if start_sec < 0.0 || end_sec < 0.0 {
        bail!("pad durations must be >= 0");
    }

    let channels = usize::from(audio.spec.channels);
    let start_samples = seconds_to_frames(start_sec, audio.spec.sample_rate) * channels;
    let end_samples = seconds_to_frames(end_sec, audio.spec.sample_rate) * channels;
    let mut output = Vec::with_capacity(start_samples + audio.samples.len() + end_samples);
    output.resize(start_samples, 0.0);
    output.extend_from_slice(&audio.samples);
    output.resize(output.len() + end_samples, 0.0);
    audio.samples = output;
    Ok(())
}

fn trim_silence(audio: &mut AudioBuffer, threshold_db: f32, min_duration_sec: f32) -> Result<()> {
    if min_duration_sec < 0.0 {
        bail!("silence minimum duration must be >= 0");
    }
    if audio.samples.is_empty() {
        return Ok(());
    }

    let channels = usize::from(audio.spec.channels);
    let threshold = 10.0_f32.powf(threshold_db / 20.0);
    let min_frames = seconds_to_frames(min_duration_sec, audio.spec.sample_rate);
    let silent = |frame: &[f32]| frame.iter().all(|sample| sample.abs() <= threshold);

    let frames: Vec<&[f32]> = audio.samples.chunks_exact(channels).collect();
    let leading = contiguous_silence(&frames, min_frames, &silent, false);
    let trailing = contiguous_silence(&frames, min_frames, &silent, true);
    let keep_start = leading.min(frames.len());
    let keep_end = frames.len().saturating_sub(trailing);

    if keep_start >= keep_end {
        audio.samples.clear();
        return Ok(());
    }

    audio.samples = audio.samples[keep_start * channels..keep_end * channels].to_vec();
    Ok(())
}

fn lowpass(audio: &mut AudioBuffer, hz: f32) -> Result<()> {
    if hz <= 0.0 {
        bail!("lowpass frequency must be > 0");
    }
    let channels = usize::from(audio.spec.channels);
    let dt = 1.0 / audio.spec.sample_rate as f32;
    let rc = 1.0 / (std::f32::consts::TAU * hz);
    let alpha = dt / (rc + dt);
    let mut state = vec![0.0; channels];

    for frame in audio.samples.chunks_exact_mut(channels) {
        for (channel, sample) in frame.iter_mut().enumerate() {
            state[channel] += alpha * (*sample - state[channel]);
            *sample = state[channel];
        }
    }

    Ok(())
}

fn highpass(audio: &mut AudioBuffer, hz: f32) -> Result<()> {
    if hz <= 0.0 {
        bail!("highpass frequency must be > 0");
    }
    let channels = usize::from(audio.spec.channels);
    let dt = 1.0 / audio.spec.sample_rate as f32;
    let rc = 1.0 / (std::f32::consts::TAU * hz);
    let alpha = rc / (rc + dt);
    let mut prev_input = vec![0.0; channels];
    let mut prev_output = vec![0.0; channels];

    for frame in audio.samples.chunks_exact_mut(channels) {
        for (channel, sample) in frame.iter_mut().enumerate() {
            let input = *sample;
            let output = alpha * (prev_output[channel] + input - prev_input[channel]);
            prev_input[channel] = input;
            prev_output[channel] = output;
            *sample = output;
        }
    }

    Ok(())
}

fn limiter(audio: &mut AudioBuffer, threshold: f32) -> Result<()> {
    if !(0.0..=1.0).contains(&threshold) {
        bail!("limiter threshold must be between 0 and 1");
    }
    for sample in &mut audio.samples {
        *sample = sample.clamp(-threshold, threshold);
    }
    Ok(())
}

fn rate(audio: &mut AudioBuffer, sample_rate: u32) -> Result<()> {
    if sample_rate == 0 {
        bail!("sample rate must be > 0");
    }
    if sample_rate == audio.spec.sample_rate || audio.samples.is_empty() {
        audio.spec.sample_rate = sample_rate;
        return Ok(());
    }

    let channels = usize::from(audio.spec.channels);
    let old_frames = audio.frames();
    if old_frames <= 1 {
        audio.spec.sample_rate = sample_rate;
        return Ok(());
    }

    let ratio = sample_rate as f64 / audio.spec.sample_rate as f64;
    let new_frames = ((old_frames as f64) * ratio).round().max(1.0) as usize;
    let mut output = vec![0.0; new_frames * channels];

    for new_frame in 0..new_frames {
        let source_pos = new_frame as f64 / ratio;
        let left = source_pos.floor() as usize;
        let right = (left + 1).min(old_frames - 1);
        let fraction = (source_pos - left as f64) as f32;

        for channel in 0..channels {
            let a = audio.samples[left * channels + channel];
            let b = audio.samples[right * channels + channel];
            output[new_frame * channels + channel] = a + (b - a) * fraction;
        }
    }

    audio.spec.sample_rate = sample_rate;
    audio.samples = output;
    Ok(())
}

fn convert_channels(audio: &mut AudioBuffer, target_channels: u16) -> Result<()> {
    if target_channels == 0 {
        bail!("channel count must be > 0");
    }
    if target_channels == audio.spec.channels {
        return Ok(());
    }

    let source_channels = usize::from(audio.spec.channels);
    let target_channels_usize = usize::from(target_channels);
    let frames = audio.frames();
    let mut output = Vec::with_capacity(frames * target_channels_usize);

    for frame in audio.samples.chunks_exact(source_channels) {
        if target_channels == 1 {
            output.push(frame.iter().sum::<f32>() / source_channels as f32);
        } else if source_channels == 1 {
            output.extend(std::iter::repeat_n(frame[0], target_channels_usize));
        } else {
            for channel in 0..target_channels_usize {
                output.push(frame[channel.min(source_channels - 1)]);
            }
        }
    }

    audio.spec.channels = target_channels;
    audio.samples = output;
    Ok(())
}

fn scale_frame(audio: &mut AudioBuffer, frame: usize, channels: usize, factor: f32) {
    let start = frame * channels;
    for sample in &mut audio.samples[start..start + channels] {
        *sample *= factor;
    }
}

fn seconds_to_frames(seconds: f32, sample_rate: u32) -> usize {
    (seconds as f64 * sample_rate as f64).round().max(0.0) as usize
}

fn contiguous_silence(
    frames: &[&[f32]],
    min_frames: usize,
    silent: &dyn Fn(&[f32]) -> bool,
    reverse: bool,
) -> usize {
    let mut count = 0;
    let iter: Box<dyn Iterator<Item = &&[f32]>> = if reverse {
        Box::new(frames.iter().rev())
    } else {
        Box::new(frames.iter())
    };

    for frame in iter {
        if silent(frame) {
            count += 1;
        } else {
            break;
        }
    }

    if count >= min_frames { count } else { 0 }
}
