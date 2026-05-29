use crate::{
    audio::{AudioBuffer, AudioSpec},
    cli::{SynthArgs, Waveform},
};
use anyhow::{Result, bail};

pub fn render(args: &SynthArgs) -> Result<AudioBuffer> {
    if args.duration < 0.0 {
        bail!("synth duration must be >= 0");
    }
    if args.freq < 0.0 {
        bail!("synth frequency must be >= 0");
    }
    if args.rate == 0 {
        bail!("synth sample rate must be > 0");
    }
    if args.channels == 0 {
        bail!("synth channel count must be > 0");
    }
    if !(0.0..=1.0).contains(&args.amplitude) {
        bail!("synth amplitude must be between 0 and 1");
    }

    let frames = (args.duration as f64 * args.rate as f64).round() as usize;
    let channels = usize::from(args.channels);
    let mut samples = Vec::with_capacity(frames * channels);
    let mut noise = Noise::new(0x5255_5354_534f_5821);

    for frame in 0..frames {
        let t = frame as f32 / args.rate as f32;
        let sample = waveform_sample(args.waveform, args.freq, t, &mut noise) * args.amplitude;
        samples.extend(std::iter::repeat_n(sample, channels));
    }

    Ok(AudioBuffer {
        spec: AudioSpec {
            sample_rate: args.rate,
            channels: args.channels,
        },
        samples,
    })
}

fn waveform_sample(waveform: Waveform, freq: f32, t: f32, noise: &mut Noise) -> f32 {
    let phase = (freq * t).fract();
    match waveform {
        Waveform::Sine => (phase * std::f32::consts::TAU).sin(),
        Waveform::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
        Waveform::Saw => 2.0 * phase - 1.0,
        Waveform::Noise => noise.next(),
        Waveform::Silence => 0.0,
    }
}

struct Noise {
    state: u64,
}

impl Noise {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let value = (self.state >> 32) as u32;
        (value as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}
