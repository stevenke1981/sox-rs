use anyhow::Result;
use rust_sox::audio::{AudioBuffer, AudioSpec};
use rust_sox::effects::{Effect, EffectChain};
use std::f32::consts::TAU;

mod common;
use common::*;

fn constant_buffer(frames: usize, channels: u16, sample_rate: u32, value: f32) -> AudioBuffer {
    AudioBuffer {
        spec: AudioSpec {
            sample_rate,
            channels,
        },
        samples: vec![value; frames * channels as usize],
    }
}

// ---------------------------------------------------------------------------
// GainDb
// ---------------------------------------------------------------------------

#[test]
fn test_gain_0db_identity() -> Result<()> {
    let mut buf = constant_buffer(10, 1, 100, 0.5);
    EffectChain::new(vec![Effect::GainDb(0.0)]).apply(&mut buf)?;
    assert!(buf.samples.iter().all(|&s| (s - 0.5).abs() < 1e-6));
    Ok(())
}

#[test]
fn test_gain_plus_6db_doubles_amplitude() -> Result<()> {
    let mut buf = constant_buffer(10, 1, 100, 0.5);
    EffectChain::new(vec![Effect::GainDb(6.0)]).apply(&mut buf)?;
    assert!((buf.peak() - 1.0).abs() < 0.01);
    Ok(())
}

#[test]
fn test_gain_minus_6db_halves_amplitude() -> Result<()> {
    let mut buf = constant_buffer(10, 1, 100, 1.0);
    EffectChain::new(vec![Effect::GainDb(-6.0)]).apply(&mut buf)?;
    assert!((buf.peak() - 0.5).abs() < 0.01);
    Ok(())
}

#[test]
fn test_gain_very_negative_approaches_zero() -> Result<()> {
    let mut buf = constant_buffer(10, 1, 100, 1.0);
    EffectChain::new(vec![Effect::GainDb(-100.0)]).apply(&mut buf)?;
    assert!(buf.peak() < 0.0001);
    Ok(())
}

#[test]
fn test_gain_stereo_applies_to_all_channels() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 2,
        },
        samples: vec![0.2, 0.4, 0.6, 0.8],
    };
    EffectChain::new(vec![Effect::GainDb(6.0)]).apply(&mut buf)?;
    for (i, &s) in buf.samples.iter().enumerate() {
        let orig = [0.2, 0.4, 0.6, 0.8][i];
        assert!((s - orig * 2.0).abs() < 0.01, "channel {i}: got {s}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Normalize
// ---------------------------------------------------------------------------

#[test]
fn test_normalize_to_target_db() -> Result<()> {
    let mut buf = constant_buffer(100, 2, 44100, 0.5);
    EffectChain::new(vec![Effect::Normalize { target_db: -6.0 }]).apply(&mut buf)?;
    let expected = 10.0_f32.powf(-6.0 / 20.0);
    assert!((buf.peak() - expected).abs() < 0.001);
    Ok(())
}

#[test]
fn test_normalize_silent_audio_is_noop() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 44100,
            channels: 1,
        },
        samples: vec![0.0; 100],
    };
    EffectChain::new(vec![Effect::Normalize { target_db: -6.0 }]).apply(&mut buf)?;
    assert!(buf.samples.iter().all(|&s| s == 0.0));
    Ok(())
}

#[test]
fn test_normalize_already_normalized_is_unchanged() -> Result<()> {
    let target_db = -6.0;
    let target = 10.0_f32.powf(target_db / 20.0);
    let mut buf = constant_buffer(100, 1, 44100, target);
    let original = buf.clone();
    EffectChain::new(vec![Effect::Normalize { target_db }]).apply(&mut buf)?;
    assert_eq!(buf.samples, original.samples);
    Ok(())
}

// ---------------------------------------------------------------------------
// Trim
// ---------------------------------------------------------------------------

#[test]
fn test_trim_from_start() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: (0..10).map(|i| i as f32).collect(),
    };
    EffectChain::new(vec![Effect::Trim {
        start_sec: 0.3,
        duration_sec: None,
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, vec![3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
    Ok(())
}

#[test]
fn test_trim_with_duration() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: (0..10).map(|i| i as f32).collect(),
    };
    EffectChain::new(vec![Effect::Trim {
        start_sec: 0.2,
        duration_sec: Some(0.4),
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, vec![2.0, 3.0, 4.0, 5.0]);
    Ok(())
}

#[test]
fn test_trim_beyond_end_returns_empty() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: (0..10).map(|i| i as f32).collect(),
    };
    EffectChain::new(vec![Effect::Trim {
        start_sec: 2.0,
        duration_sec: None,
    }])
    .apply(&mut buf)?;
    assert!(buf.samples.is_empty());
    Ok(())
}

#[test]
fn test_trim_zero_duration_returns_empty() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: (0..10).map(|i| i as f32).collect(),
    };
    EffectChain::new(vec![Effect::Trim {
        start_sec: 0.0,
        duration_sec: Some(0.0),
    }])
    .apply(&mut buf)?;
    assert!(buf.samples.is_empty());
    Ok(())
}

#[test]
fn test_trim_negative_start_errors() {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: vec![0.0; 10],
    };
    let result = EffectChain::new(vec![Effect::Trim {
        start_sec: -1.0,
        duration_sec: None,
    }])
    .apply(&mut buf);
    assert!(result.is_err());
}

#[test]
fn test_trim_stereo_preserves_channels() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 2,
        },
        samples: vec![1.0, 10.0, 2.0, 20.0, 3.0, 30.0, 4.0, 40.0],
    };
    EffectChain::new(vec![Effect::Trim {
        start_sec: 0.1,
        duration_sec: Some(0.2),
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.spec.channels, 2);
    assert_eq!(buf.samples, vec![2.0, 20.0, 3.0, 30.0]);
    Ok(())
}

// ---------------------------------------------------------------------------
// Fade
// ---------------------------------------------------------------------------

#[test]
fn test_fade_in_ramps_correctly() -> Result<()> {
    let mut buf = constant_buffer(10, 1, 10, 1.0);
    EffectChain::new(vec![Effect::Fade {
        in_sec: 0.5,
        out_sec: 0.0,
    }])
    .apply(&mut buf)?;
    let expected: Vec<f32> = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.0, 1.0, 1.0, 1.0];
    assert_buffer_eq(&buf.samples, &expected, 1e-6, "fade in");
    Ok(())
}

#[test]
fn test_fade_out_ramps_correctly() -> Result<()> {
    let mut buf = constant_buffer(10, 1, 10, 1.0);
    EffectChain::new(vec![Effect::Fade {
        in_sec: 0.0,
        out_sec: 0.5,
    }])
    .apply(&mut buf)?;
    let expected: Vec<f32> = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.8, 0.6, 0.4, 0.2];
    assert_buffer_eq(&buf.samples, &expected, 1e-6, "fade out");
    Ok(())
}

#[test]
fn test_fade_both_ends() -> Result<()> {
    let mut buf = constant_buffer(10, 2, 10, 1.0);
    EffectChain::new(vec![Effect::Fade {
        in_sec: 0.5,
        out_sec: 0.5,
    }])
    .apply(&mut buf)?;
    let expected: Vec<f32> = vec![
        0.0, 0.0, 0.2, 0.2, 0.4, 0.4, 0.6, 0.6, 0.8, 0.8, 1.0, 1.0, 0.8, 0.8, 0.6, 0.6, 0.4, 0.4,
        0.2, 0.2,
    ];
    assert_buffer_eq(&buf.samples, &expected, 1e-6, "fade both");
    Ok(())
}

#[test]
fn test_fade_zero_durations_is_noop() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: vec![0.3, 0.6, 0.9],
    };
    let original = buf.clone();
    EffectChain::new(vec![Effect::Fade {
        in_sec: 0.0,
        out_sec: 0.0,
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, original.samples);
    Ok(())
}

#[test]
fn test_fade_negative_durations_error() {
    let mut buf = constant_buffer(10, 1, 10, 1.0);
    let result = EffectChain::new(vec![Effect::Fade {
        in_sec: -0.1,
        out_sec: 0.0,
    }])
    .apply(&mut buf);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Reverse
// ---------------------------------------------------------------------------

#[test]
fn test_reverse_changes_order() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 2,
        },
        samples: vec![1.0, 10.0, 2.0, 20.0, 3.0, 30.0],
    };
    EffectChain::new(vec![Effect::Reverse]).apply(&mut buf)?;
    assert_eq!(buf.samples, vec![3.0, 30.0, 2.0, 20.0, 1.0, 10.0]);
    Ok(())
}

#[test]
fn test_reverse_twice_is_identity() -> Result<()> {
    let mut buf = test_buffer_with_samples(50, 2, 44100, 0.5);
    let original = buf.clone();
    EffectChain::new(vec![Effect::Reverse, Effect::Reverse]).apply(&mut buf)?;
    assert_eq!(buf.samples, original.samples);
    Ok(())
}

#[test]
fn test_reverse_empty_buffer_is_noop() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 44100,
            channels: 2,
        },
        samples: vec![],
    };
    EffectChain::new(vec![Effect::Reverse]).apply(&mut buf)?;
    assert!(buf.samples.is_empty());
    Ok(())
}

// ---------------------------------------------------------------------------
// Speed
// ---------------------------------------------------------------------------

#[test]
fn test_speed_half_doubles_frames() -> Result<()> {
    let mut buf = test_buffer_with_samples(100, 1, 100, 0.5);
    EffectChain::new(vec![Effect::Speed { factor: 0.5 }]).apply(&mut buf)?;
    assert_eq!(buf.frames(), 200);
    Ok(())
}

#[test]
fn test_speed_double_halves_frames() -> Result<()> {
    let mut buf = test_buffer_with_samples(100, 1, 100, 0.5);
    EffectChain::new(vec![Effect::Speed { factor: 2.0 }]).apply(&mut buf)?;
    assert_eq!(buf.frames(), 50);
    Ok(())
}

#[test]
fn test_speed_one_is_identity() -> Result<()> {
    let mut buf = test_buffer_with_samples(100, 2, 44100, 0.5);
    let original = buf.clone();
    EffectChain::new(vec![Effect::Speed { factor: 1.0 }]).apply(&mut buf)?;
    assert_eq!(buf.samples, original.samples);
    Ok(())
}

#[test]
fn test_speed_negative_factor_errors() {
    let mut buf = test_buffer_with_samples(100, 1, 100, 0.5);
    let result = EffectChain::new(vec![Effect::Speed { factor: -1.0 }]).apply(&mut buf);
    assert!(result.is_err());
}

#[test]
fn test_speed_zero_factor_errors() {
    let mut buf = test_buffer_with_samples(100, 1, 100, 0.5);
    let result = EffectChain::new(vec![Effect::Speed { factor: 0.0 }]).apply(&mut buf);
    assert!(result.is_err());
}

#[test]
fn test_speed_stereo_preserves_channels() -> Result<()> {
    let mut buf = test_buffer_with_samples(100, 2, 100, 0.5);
    EffectChain::new(vec![Effect::Speed { factor: 0.5 }]).apply(&mut buf)?;
    assert_eq!(buf.spec.channels, 2);
    assert_eq!(buf.frames(), 200);
    assert_eq!(buf.samples.len(), 400);
    Ok(())
}

// ---------------------------------------------------------------------------
// Pad
// ---------------------------------------------------------------------------

#[test]
fn test_pad_start_adds_silence() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: vec![1.0, 2.0, 3.0],
    };
    EffectChain::new(vec![Effect::Pad {
        start_sec: 0.2,
        end_sec: 0.0,
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, vec![0.0, 0.0, 1.0, 2.0, 3.0]);
    Ok(())
}

#[test]
fn test_pad_end_adds_silence() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 2,
        },
        samples: vec![1.0, 2.0, 3.0, 4.0],
    };
    EffectChain::new(vec![Effect::Pad {
        start_sec: 0.0,
        end_sec: 0.1,
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, vec![1.0, 2.0, 3.0, 4.0, 0.0, 0.0]);
    Ok(())
}

#[test]
fn test_pad_both_sides() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: vec![5.0],
    };
    EffectChain::new(vec![Effect::Pad {
        start_sec: 0.1,
        end_sec: 0.2,
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, vec![0.0, 5.0, 0.0, 0.0]);
    Ok(())
}

#[test]
fn test_pad_zero_is_noop() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 2,
        },
        samples: vec![0.5, -0.5],
    };
    let original = buf.clone();
    EffectChain::new(vec![Effect::Pad {
        start_sec: 0.0,
        end_sec: 0.0,
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, original.samples);
    Ok(())
}

// ---------------------------------------------------------------------------
// Silence (trim_silence)
// ---------------------------------------------------------------------------

#[test]
fn test_trim_silence_removes_leading_silence() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0],
    };
    EffectChain::new(vec![Effect::Silence {
        threshold_db: -60.0,
        min_duration_sec: 0.1,
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, vec![1.0, 2.0, 3.0]);
    Ok(())
}

#[test]
fn test_trim_silence_removes_trailing_silence() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: vec![1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0],
    };
    EffectChain::new(vec![Effect::Silence {
        threshold_db: -60.0,
        min_duration_sec: 0.2,
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, vec![1.0, 2.0, 3.0]);
    Ok(())
}

#[test]
fn test_trim_silence_no_silence_is_noop() -> Result<()> {
    let samples: Vec<f32> = (0..10).map(|i| (i as f32 + 1.0) * 0.1).collect();
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: samples.clone(),
    };
    EffectChain::new(vec![Effect::Silence {
        threshold_db: -60.0,
        min_duration_sec: 0.1,
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, samples);
    Ok(())
}

#[test]
fn test_trim_silence_all_silent_returns_empty() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: vec![0.0; 10],
    };
    EffectChain::new(vec![Effect::Silence {
        threshold_db: -60.0,
        min_duration_sec: 0.1,
    }])
    .apply(&mut buf)?;
    assert!(buf.samples.is_empty());
    Ok(())
}

#[test]
fn test_trim_silence_below_min_duration_is_noop() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: vec![0.0, 0.0, 1.0],
    };
    EffectChain::new(vec![Effect::Silence {
        threshold_db: -60.0,
        min_duration_sec: 1.0,
    }])
    .apply(&mut buf)?;
    assert_eq!(buf.samples, vec![0.0, 0.0, 1.0]);
    Ok(())
}

// ---------------------------------------------------------------------------
// LowPass
// ---------------------------------------------------------------------------

#[test]
fn test_lowpass_dc_preserved() -> Result<()> {
    let mut buf = constant_buffer(1000, 1, 100, 1.0);
    EffectChain::new(vec![Effect::LowPass { hz: 20.0 }]).apply(&mut buf)?;
    let last = buf.samples[buf.samples.len() - 1];
    assert!((last - 1.0).abs() < 0.01, "DC not preserved: {last}");
    Ok(())
}

#[test]
fn test_lowpass_high_freq_attenuated() -> Result<()> {
    let sample_rate = 1000;
    let signal_hz = 100.0;
    let mut samples = Vec::with_capacity(sample_rate as usize);
    for i in 0..sample_rate {
        let t = i as f32 / sample_rate as f32;
        samples.push((t * signal_hz * TAU).sin());
    }
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate,
            channels: 1,
        },
        samples,
    };
    let peak_before = buf.peak();
    EffectChain::new(vec![Effect::LowPass { hz: 20.0 }]).apply(&mut buf)?;
    assert!(buf.peak() < peak_before * 0.5);
    Ok(())
}

#[test]
fn test_lowpass_zero_hz_errors() {
    let mut buf = constant_buffer(10, 1, 100, 1.0);
    let result = EffectChain::new(vec![Effect::LowPass { hz: 0.0 }]).apply(&mut buf);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// HighPass
// ---------------------------------------------------------------------------

#[test]
fn test_highpass_dc_removed() -> Result<()> {
    let mut buf = constant_buffer(1000, 1, 100, 1.0);
    EffectChain::new(vec![Effect::HighPass { hz: 20.0 }]).apply(&mut buf)?;
    let last = buf.samples[buf.samples.len() - 1];
    assert!(last.abs() < 0.01, "DC not removed: {last}");
    Ok(())
}

#[test]
fn test_highpass_high_freq_preserved() -> Result<()> {
    let sample_rate = 1000;
    let signal_hz = 100.0;
    let mut samples = Vec::with_capacity(sample_rate as usize);
    for i in 0..sample_rate {
        let t = i as f32 / sample_rate as f32;
        samples.push((t * signal_hz * TAU).sin());
    }
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate,
            channels: 1,
        },
        samples,
    };
    let peak_before = buf.peak();
    EffectChain::new(vec![Effect::HighPass { hz: 5.0 }]).apply(&mut buf)?;
    assert!(buf.peak() > peak_before * 0.8);
    Ok(())
}

#[test]
fn test_highpass_zero_hz_errors() {
    let mut buf = constant_buffer(10, 1, 100, 1.0);
    let result = EffectChain::new(vec![Effect::HighPass { hz: 0.0 }]).apply(&mut buf);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Limiter
// ---------------------------------------------------------------------------

#[test]
fn test_limiter_clips_above_threshold() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 1,
        },
        samples: vec![0.0, 0.3, 0.6, 0.9, 1.0, -0.7, -1.0],
    };
    EffectChain::new(vec![Effect::Limiter { threshold: 0.5 }]).apply(&mut buf)?;
    assert_eq!(buf.samples, vec![0.0, 0.3, 0.5, 0.5, 0.5, -0.5, -0.5]);
    Ok(())
}

#[test]
fn test_limiter_below_threshold_unchanged() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 2,
        },
        samples: vec![0.1, -0.2, 0.3, -0.4],
    };
    let original = buf.clone();
    EffectChain::new(vec![Effect::Limiter { threshold: 0.5 }]).apply(&mut buf)?;
    assert_eq!(buf.samples, original.samples);
    Ok(())
}

#[test]
fn test_limiter_threshold_zero_silences_all() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 2,
        },
        samples: vec![0.5, -0.5, 0.3, -0.3],
    };
    EffectChain::new(vec![Effect::Limiter { threshold: 0.0 }]).apply(&mut buf)?;
    assert!(buf.samples.iter().all(|&s| s == 0.0));
    Ok(())
}

#[test]
fn test_limiter_threshold_one_is_noop() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 1,
        },
        samples: vec![-0.5, 0.0, 0.5],
    };
    let original = buf.clone();
    EffectChain::new(vec![Effect::Limiter { threshold: 1.0 }]).apply(&mut buf)?;
    assert_eq!(buf.samples, original.samples);
    Ok(())
}

#[test]
fn test_limiter_invalid_threshold_errors() {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 1,
        },
        samples: vec![0.5],
    };
    let result = EffectChain::new(vec![Effect::Limiter { threshold: 1.5 }]).apply(&mut buf);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Rate
// ---------------------------------------------------------------------------

#[test]
fn test_rate_half_reduces_frames() -> Result<()> {
    let mut buf = test_buffer_with_samples(100, 2, 44100, 0.5);
    EffectChain::new(vec![Effect::Rate { sample_rate: 22050 }]).apply(&mut buf)?;
    assert_eq!(buf.spec.sample_rate, 22050);
    assert_eq!(buf.frames(), 50);
    Ok(())
}

#[test]
fn test_rate_double_increases_frames() -> Result<()> {
    let mut buf = test_buffer_with_samples(50, 1, 22050, 0.5);
    EffectChain::new(vec![Effect::Rate { sample_rate: 44100 }]).apply(&mut buf)?;
    assert_eq!(buf.spec.sample_rate, 44100);
    assert_eq!(buf.frames(), 100);
    Ok(())
}

#[test]
fn test_rate_same_is_noop() -> Result<()> {
    let mut buf = test_buffer_with_samples(100, 2, 44100, 0.5);
    let original = buf.clone();
    EffectChain::new(vec![Effect::Rate { sample_rate: 44100 }]).apply(&mut buf)?;
    assert_eq!(buf.spec.sample_rate, original.spec.sample_rate);
    assert_eq!(buf.spec.channels, original.spec.channels);
    assert_eq!(buf.samples, original.samples);
    Ok(())
}

#[test]
fn test_rate_zero_errors() {
    let mut buf = test_buffer_with_samples(10, 1, 44100, 0.5);
    let result = EffectChain::new(vec![Effect::Rate { sample_rate: 0 }]).apply(&mut buf);
    assert!(result.is_err());
}

#[test]
fn test_rate_stereo_preserves_channels() -> Result<()> {
    let mut buf = test_buffer_with_samples(100, 2, 44100, 0.5);
    EffectChain::new(vec![Effect::Rate { sample_rate: 22050 }]).apply(&mut buf)?;
    assert_eq!(buf.spec.channels, 2);
    Ok(())
}

// ---------------------------------------------------------------------------
// Channels
// ---------------------------------------------------------------------------

#[test]
fn test_channels_stereo_to_mono() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 2,
        },
        samples: vec![1.0, 3.0, 2.0, 4.0, 5.0, 7.0],
    };
    EffectChain::new(vec![Effect::Channels { channels: 1 }]).apply(&mut buf)?;
    assert_eq!(buf.spec.channels, 1);
    assert_eq!(buf.samples, vec![2.0, 3.0, 6.0]);
    Ok(())
}

#[test]
fn test_channels_mono_to_stereo() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 1,
        },
        samples: vec![1.0, 2.0, 3.0],
    };
    EffectChain::new(vec![Effect::Channels { channels: 2 }]).apply(&mut buf)?;
    assert_eq!(buf.spec.channels, 2);
    assert_eq!(buf.samples, vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
    Ok(())
}

#[test]
fn test_channels_same_is_noop() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 2,
        },
        samples: vec![1.0, -1.0, 0.5, -0.5],
    };
    let original = buf.clone();
    EffectChain::new(vec![Effect::Channels { channels: 2 }]).apply(&mut buf)?;
    assert_eq!(buf.samples, original.samples);
    assert_eq!(buf.spec.channels, 2);
    Ok(())
}

#[test]
fn test_channels_zero_errors() {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 2,
        },
        samples: vec![1.0, 2.0],
    };
    let result = EffectChain::new(vec![Effect::Channels { channels: 0 }]).apply(&mut buf);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

#[test]
fn test_stats_is_noop() -> Result<()> {
    let mut buf = test_buffer_with_samples(100, 2, 44100, 0.5);
    let original = buf.clone();
    EffectChain::new(vec![Effect::Stats]).apply(&mut buf)?;
    assert_eq!(buf.samples, original.samples);
    assert_eq!(buf.spec.sample_rate, original.spec.sample_rate);
    assert_eq!(buf.spec.channels, original.spec.channels);
    Ok(())
}

// ---------------------------------------------------------------------------
// EffectChain
// ---------------------------------------------------------------------------

#[test]
fn test_chain_multiple_effects_apply_in_order() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: vec![0.0, 0.5, 1.0],
    };
    EffectChain::new(vec![Effect::GainDb(6.0), Effect::Reverse]).apply(&mut buf)?;
    let factor = 10.0_f32.powf(6.0 / 20.0);
    assert!((buf.samples[0] - 1.0 * factor).abs() < 1e-6);
    assert!((buf.samples[1] - 0.5 * factor).abs() < 1e-6);
    assert_eq!(buf.samples[2], 0.0);
    Ok(())
}

#[test]
fn test_chain_empty_is_noop() -> Result<()> {
    let mut buf = test_buffer_with_samples(100, 2, 44100, 0.5);
    let original = buf.clone();
    EffectChain::new(vec![]).apply(&mut buf)?;
    assert_eq!(buf.samples, original.samples);
    assert_eq!(buf.spec.sample_rate, original.spec.sample_rate);
    assert_eq!(buf.spec.channels, original.spec.channels);
    Ok(())
}

#[test]
fn test_chain_builder_with_appends_effects() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 10,
            channels: 1,
        },
        samples: vec![1.0, 2.0, 3.0],
    };
    EffectChain::new(vec![Effect::Reverse])
        .with(Effect::GainDb(-6.0))
        .apply(&mut buf)?;
    // First reverse: [3.0, 2.0, 1.0], then gain -6dB: multiply by ~0.5
    assert!((buf.samples[0] - 1.5).abs() < 0.01);
    assert!((buf.samples[1] - 1.0).abs() < 0.01);
    assert!((buf.samples[2] - 0.5).abs() < 0.01);
    Ok(())
}

#[test]
fn test_wants_stats_true_with_stats() {
    let chain = EffectChain::new(vec![Effect::GainDb(3.0), Effect::Stats]);
    assert!(chain.wants_stats());
}

#[test]
fn test_wants_stats_false_without_stats() {
    let chain = EffectChain::new(vec![Effect::GainDb(3.0), Effect::Reverse]);
    assert!(!chain.wants_stats());
}

// ---------------------------------------------------------------------------
// Edge cases across effects
// ---------------------------------------------------------------------------

#[test]
fn test_empty_buffer_through_all_effects() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 44100,
            channels: 2,
        },
        samples: vec![],
    };
    EffectChain::new(vec![
        Effect::GainDb(6.0),
        Effect::Normalize { target_db: -3.0 },
        Effect::Trim {
            start_sec: 0.0,
            duration_sec: None,
        },
        Effect::Reverse,
        Effect::Speed { factor: 1.5 },
        Effect::Pad {
            start_sec: 0.0,
            end_sec: 0.0,
        },
        Effect::Silence {
            threshold_db: -60.0,
            min_duration_sec: 0.1,
        },
        Effect::LowPass { hz: 100.0 },
        Effect::HighPass { hz: 20.0 },
        Effect::Limiter { threshold: 0.9 },
        Effect::Rate { sample_rate: 22050 },
        Effect::Channels { channels: 1 },
    ])
    .apply(&mut buf)?;
    assert!(buf.samples.is_empty());
    Ok(())
}

#[test]
fn test_single_frame_behaviors() -> Result<()> {
    let mut buf = AudioBuffer {
        spec: AudioSpec {
            sample_rate: 100,
            channels: 1,
        },
        samples: vec![0.5],
    };
    EffectChain::new(vec![
        Effect::GainDb(6.0),
        Effect::Speed { factor: 2.0 },
        Effect::Rate { sample_rate: 200 },
    ])
    .apply(&mut buf)?;
    assert_eq!(buf.frames(), 1);
    assert_eq!(buf.spec.sample_rate, 200);
    Ok(())
}
