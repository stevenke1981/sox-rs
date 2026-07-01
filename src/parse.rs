use crate::effects::Effect;
use anyhow::{Context, Result, anyhow};

/// Parse a sequence of effect tokens (SoX-style command line arguments)
/// into a vector of `Effect` values.
pub fn parse_effects(tokens: &[String]) -> Result<(Vec<Effect>, bool)> {
    let mut effects = Vec::new();
    let mut stat_json = false;
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].as_str() {
            "gain" | "vol" => {
                let db = parse_next_f32(tokens, &mut i, "gain requires <db>")?;
                effects.push(Effect::GainDb(db));
            }
            "norm" | "normalize" => {
                let target_db = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        value
                    }
                    None => -1.0,
                };
                effects.push(Effect::Normalize { target_db });
            }
            "trim" => {
                let start = parse_next_f32(tokens, &mut i, "trim requires <start-sec>")?;
                let duration = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        Some(value)
                    }
                    None => None,
                };
                effects.push(Effect::Trim {
                    start_sec: start,
                    duration_sec: duration,
                });
            }
            "fade" => {
                let in_sec = parse_next_f32(tokens, &mut i, "fade requires <in-sec>")?;
                let out_sec = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        value
                    }
                    None => 0.0,
                };
                effects.push(Effect::Fade { in_sec, out_sec });
            }
            "reverse" | "reverse-samples" => effects.push(Effect::Reverse),
            "speed" => {
                let factor = parse_next_f32(tokens, &mut i, "speed requires <factor>")?;
                effects.push(Effect::Speed { factor });
            }
            "pad" => {
                let start_sec = parse_next_f32(tokens, &mut i, "pad requires <start-sec>")?;
                let end_sec = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        value
                    }
                    None => 0.0,
                };
                effects.push(Effect::Pad { start_sec, end_sec });
            }
            "silence" => {
                let threshold_db =
                    parse_next_f32(tokens, &mut i, "silence requires <threshold-db>")?;
                let min_duration_sec = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        value
                    }
                    None => 0.1,
                };
                effects.push(Effect::Silence {
                    threshold_db,
                    min_duration_sec,
                });
            }
            "lowpass" => {
                let hz = parse_next_f32(tokens, &mut i, "lowpass requires <hz>")?;
                effects.push(Effect::LowPass { hz });
            }
            "highpass" => {
                let hz = parse_next_f32(tokens, &mut i, "highpass requires <hz>")?;
                effects.push(Effect::HighPass { hz });
            }
            "limiter" => {
                let threshold = match tokens.get(i + 1).and_then(|s| s.parse::<f32>().ok()) {
                    Some(value) => {
                        i += 1;
                        value
                    }
                    None => 0.95,
                };
                effects.push(Effect::Limiter { threshold });
            }
            "rate" | "resample" => {
                let sample_rate = parse_next_u32(tokens, &mut i, "rate requires <hz>")?;
                effects.push(Effect::Rate { sample_rate });
            }
            "channels" | "ch" => {
                let channels = parse_next_u16(tokens, &mut i, "channels requires <count>")?;
                effects.push(Effect::Channels { channels });
            }
            "stat" | "stats" => effects.push(Effect::Stats),
            "stat-json" | "stats-json" => {
                stat_json = true;
                effects.push(Effect::Stats);
            }
            unknown => return Err(anyhow!("unsupported effect '{unknown}'")),
        }
        i += 1;
    }

    Ok((effects, stat_json))
}

fn parse_next_f32(tokens: &[String], i: &mut usize, message: &str) -> Result<f32> {
    *i += 1;
    tokens
        .get(*i)
        .ok_or_else(|| anyhow!(message.to_string()))?
        .parse::<f32>()
        .with_context(|| message.to_string())
}

fn parse_next_u32(tokens: &[String], i: &mut usize, message: &str) -> Result<u32> {
    *i += 1;
    tokens
        .get(*i)
        .ok_or_else(|| anyhow!(message.to_string()))?
        .parse::<u32>()
        .with_context(|| message.to_string())
}

fn parse_next_u16(tokens: &[String], i: &mut usize, message: &str) -> Result<u16> {
    *i += 1;
    tokens
        .get(*i)
        .ok_or_else(|| anyhow!(message.to_string()))?
        .parse::<u16>()
        .with_context(|| message.to_string())
}
