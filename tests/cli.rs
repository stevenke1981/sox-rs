use hound::{SampleFormat, WavSpec, WavWriter};
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[test]
fn converts_with_sox_style_effects() {
    let temp = std::env::temp_dir().join(format!("rust-sox-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).unwrap();
    let input = temp.join("input.wav");
    let output = temp.join("output.wav");
    write_test_wav(&input, 4410, 1, 0.5);

    let status = Command::new(env!("CARGO_BIN_EXE_sox"))
        .arg(&input)
        .arg(&output)
        .args([
            "gain", "-6", "trim", "0", "0.01", "pad", "0.01", "0.01", "speed", "1.5", "rate",
            "22050", "lowpass", "8000", "highpass", "20", "limiter", "0.8",
        ])
        .status()
        .unwrap();

    assert!(status.success());
    let reader = hound::WavReader::open(output).unwrap();
    assert_eq!(reader.spec().sample_rate, 22050);
}

#[test]
fn lists_format_support() {
    let output = Command::new(env!("CARGO_BIN_EXE_sox"))
        .args(["formats"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout).unwrap().contains("flac"));
}

#[test]
fn prints_json_info() {
    let temp = std::env::temp_dir().join(format!("rust-sox-info-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).unwrap();
    let input = temp.join("input.wav");
    write_test_wav(&input, 4410, 1, 0.5);

    let output = Command::new(env!("CARGO_BIN_EXE_sox"))
        .args(["info", "--json"])
        .arg(&input)
        .output()
        .unwrap();

    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("\"sample_rate\""));
}

#[test]
fn concatenates_and_mixes_inputs() {
    let temp = std::env::temp_dir().join(format!("rust-sox-combine-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).unwrap();
    let a = temp.join("a.wav");
    let b = temp.join("b.wav");
    let concat = temp.join("concat.wav");
    let mix = temp.join("mix.wav");
    write_test_wav(&a, 1000, 1, 0.4);
    write_test_wav(&b, 500, 1, 0.2);

    let concat_status = Command::new(env!("CARGO_BIN_EXE_sox"))
        .args(["concat", "-o"])
        .arg(&concat)
        .arg(&a)
        .arg(&b)
        .status()
        .unwrap();
    assert!(concat_status.success());
    assert_eq!(hound::WavReader::open(&concat).unwrap().duration(), 1500);

    let mix_status = Command::new(env!("CARGO_BIN_EXE_sox"))
        .args(["mix", "-o"])
        .arg(&mix)
        .arg(&a)
        .arg(&b)
        .status()
        .unwrap();
    assert!(mix_status.success());
    assert_eq!(hound::WavReader::open(&mix).unwrap().duration(), 1000);
}

#[test]
fn streams_wav_without_full_buffer_pipeline() {
    let temp = std::env::temp_dir().join(format!("rust-sox-stream-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).unwrap();
    let input = temp.join("input.wav");
    let output = temp.join("stream.wav");
    write_test_wav(&input, 44_100, 2, 0.9);

    let status = Command::new(env!("CARGO_BIN_EXE_sox"))
        .args([
            "stream",
            "--gain-db=-3",
            "--fade-in",
            "0.01",
            "--fade-out",
            "0.01",
            "--limiter",
            "0.7",
        ])
        .arg(&input)
        .arg(&output)
        .status()
        .unwrap();

    assert!(status.success());
    let reader = hound::WavReader::open(output).unwrap();
    assert_eq!(reader.spec().channels, 2);
    assert_eq!(reader.duration(), 44_100);
}

#[test]
fn converts_wav_roundtrip_without_external_deps() {
    // Hermetic test: generate WAV → convert with effects → verify output
    let temp = std::env::temp_dir().join(format!("rust-sox-convert-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).unwrap();
    let input = temp.join("input.wav");
    let output = temp.join("output.wav");
    write_test_wav(&input, 4410, 1, 0.5);

    let status = Command::new(env!("CARGO_BIN_EXE_sox"))
        .arg("convert")
        .arg(&input)
        .arg(&output)
        .arg("--normalize")
        .status()
        .unwrap();

    assert!(status.success());
    let reader = hound::WavReader::open(output).unwrap();
    assert_eq!(reader.spec().sample_rate, 44_100);
    assert_eq!(reader.spec().channels, 1);
}

#[test]
fn runs_json_plan() {
    let temp = std::env::temp_dir().join(format!("rust-sox-plan-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).unwrap();
    let input = temp.join("input.wav");
    let output = temp.join("planned.wav");
    let plan = temp.join("plan.json");
    write_test_wav(&input, 4410, 1, 0.5);
    let plan_text = format!(
        r#"{{
  "mode": "convert",
  "inputs": ["{}"],
  "output": "{}",
  "effects": ["trim", "0", "0.02", "silence", "-60", "0.001", "channels", "2"],
  "stat_json": true
}}"#,
        input.display().to_string().replace('\\', "\\\\"),
        output.display().to_string().replace('\\', "\\\\")
    );
    std::fs::File::create(&plan)
        .unwrap()
        .write_all(plan_text.as_bytes())
        .unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_sox"))
        .args(["run-plan"])
        .arg(&plan)
        .output()
        .unwrap();

    assert!(result.status.success());
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("\"channels\": 2")
    );
    assert_eq!(hound::WavReader::open(output).unwrap().spec().channels, 2);
}

#[test]
fn synthesizes_audio_with_effects() {
    let temp = std::env::temp_dir().join(format!("rust-sox-synth-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).unwrap();
    let output = temp.join("tone.wav");

    let result = Command::new(env!("CARGO_BIN_EXE_sox"))
        .args([
            "synth",
            "--duration",
            "0.1",
            "--freq",
            "880",
            "--channels",
            "2",
            "--waveform",
            "triangle",
            "--fade",
            "0.01",
            "--stat-json",
        ])
        .arg(&output)
        .output()
        .unwrap();

    assert!(result.status.success());
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("\"sample_rate\": 44100")
    );
    let reader = hound::WavReader::open(output).unwrap();
    assert_eq!(reader.spec().channels, 2);
    assert_eq!(reader.duration(), 4410);
}

fn write_test_wav(path: &Path, frames: usize, channels: u16, amplitude: f32) {
    let spec = WavSpec {
        channels,
        sample_rate: 44_100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec).unwrap();
    for index in 0..frames {
        let phase = index as f32 / 44_100.0 * 440.0 * std::f32::consts::TAU;
        let sample = (phase.sin() * i16::MAX as f32 * amplitude) as i16;
        for _ in 0..channels {
            writer.write_sample(sample).unwrap();
        }
    }
    writer.finalize().unwrap();
}
