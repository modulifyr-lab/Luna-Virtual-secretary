use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    recording_buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
    sample_rate: u32,
    channels: u16,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            recording_buffer: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            sample_rate: 44100,
            channels: 1,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        self.recording_buffer.lock().unwrap().clear();

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "No default audio input device found".to_string())?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input audio config: {:?}", e))?;

        self.sample_rate = config.sample_rate().0;
        self.channels = config.channels();

        let buffer_clone = Arc::clone(&self.recording_buffer);
        let channels = self.channels;

        let err_fn = |err| eprintln!("Audio stream error: {}", err);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    let mut buf = buffer_clone.lock().unwrap();
                    if channels == 1 {
                        buf.extend_from_slice(data);
                    } else {
                        for chunk in data.chunks_exact(channels as usize) {
                            let sum: f32 = chunk.iter().sum();
                            buf.push(sum / channels as f32);
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    let mut buf = buffer_clone.lock().unwrap();
                    if channels == 1 {
                        for &sample in data {
                            buf.push(sample as f32 / i16::MAX as f32);
                        }
                    } else {
                        for chunk in data.chunks_exact(channels as usize) {
                            let sum: f32 = chunk.iter().map(|&s| s as f32 / i16::MAX as f32).sum();
                            buf.push(sum / channels as f32);
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |data: &[u16], _| {
                    let mut buf = buffer_clone.lock().unwrap();
                    if channels == 1 {
                        for &sample in data {
                            buf.push((sample as f32 - 32768.0) / 32768.0);
                        }
                    } else {
                        for chunk in data.chunks_exact(channels as usize) {
                            let sum: f32 = chunk.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).sum();
                            buf.push(sum / channels as f32);
                        }
                    }
                },
                err_fn,
                None,
            ),
            _ => return Err("Unsupported sample format".to_string()),
        }.map_err(|e| format!("Failed to build input audio stream: {:?}", e))?;

        stream.play().map_err(|e| format!("Failed to play input stream: {:?}", e))?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop(&mut self) -> Vec<f32> {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        let raw_samples = {
            let mut buf = self.recording_buffer.lock().unwrap();
            let samples = buf.clone();
            buf.clear();
            samples
        };

        resample_to_16k(&raw_samples, self.sample_rate)
    }
}

fn resample_to_16k(input: &[f32], input_rate: u32) -> Vec<f32> {
    if input.is_empty() {
        return Vec::new();
    }
    if input_rate == 16000 {
        return input.to_vec();
    }

    let target_rate = 16000.0;
    let ratio = input_rate as f64 / target_rate;
    let target_len = (input.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(target_len);

    for i in 0..target_len {
        let src_idx = i as f64 * ratio;
        let idx_floor = src_idx.floor() as usize;
        let idx_ceil = (idx_floor + 1).min(input.len() - 1);
        let frac = (src_idx - idx_floor as f64) as f32;

        if idx_floor < input.len() {
            let sample = input[idx_floor] * (1.0 - frac) + input[idx_ceil] * frac;
            output.push(sample);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_16k_same() {
        let input = vec![0.1, 0.2, 0.3, 0.4];
        let output = resample_to_16k(&input, 16000);
        assert_eq!(input, output);
    }

    #[test]
    fn test_resample_32k_to_16k() {
        let input = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let output = resample_to_16k(&input, 32000);
        assert_eq!(output.len(), 3);
    }
}
