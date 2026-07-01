use rust_sox::audio::{AudioBuffer, AudioSpec};

#[allow(dead_code)]
pub fn test_buffer(frames: usize, channels: u16, sample_rate: u32) -> AudioBuffer {
    AudioBuffer {
        spec: AudioSpec {
            sample_rate,
            channels,
        },
        samples: vec![0.0; frames * channels as usize],
    }
}

pub fn test_buffer_with_samples(
    frames: usize,
    channels: u16,
    sample_rate: u32,
    amplitude: f32,
) -> AudioBuffer {
    let mut samples = Vec::with_capacity(frames * channels as usize);
    for frame in 0..frames {
        let t = frame as f32 / sample_rate as f32;
        let value = (t * 440.0 * std::f32::consts::TAU).sin() * amplitude;
        for _ in 0..channels {
            samples.push(value);
        }
    }
    AudioBuffer {
        spec: AudioSpec {
            sample_rate,
            channels,
        },
        samples,
    }
}

pub fn approx_eq(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

pub fn assert_buffer_eq(actual: &[f32], expected: &[f32], tolerance: f32, msg: &str) {
    assert_eq!(actual.len(), expected.len(), "{msg}: length mismatch");
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!(
            approx_eq(*a, *e, tolerance),
            "{msg}: at index {i}, got {a}, expected {e}"
        );
    }
}
