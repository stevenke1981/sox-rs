/// Convert seconds to frame count at the given sample rate.
pub fn seconds_to_frames(seconds: f32, sample_rate: u32) -> usize {
    (seconds as f64 * sample_rate as f64).round().max(0.0) as usize
}
