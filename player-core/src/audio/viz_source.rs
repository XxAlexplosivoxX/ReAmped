use std::sync::{Arc, Mutex};

pub type SharedSamples = Arc<Mutex<Vec<f32>>>;

pub struct Visualizer {
    samples: SharedSamples,
    channel_buf: Vec<f32>,
    max_len: usize,
}

impl Visualizer {
    pub fn new(samples: SharedSamples, channels: usize) -> Self {
        Self {
            samples,
            channel_buf: Vec::with_capacity(channels),
            max_len: 4096,
        }
    }

    pub fn push_sample(&mut self, sample: f32, channels: usize) {
        self.channel_buf.push(sample);

        if self.channel_buf.len() == channels {
            let mono =
                self.channel_buf.iter().sum::<f32>() / channels as f32;

            let mut buf = self.samples.lock().unwrap();
            buf.push(mono);

            if buf.len() > self.max_len {
                let excess = buf.len() - self.max_len;
                buf.drain(..excess);
            }

            self.channel_buf.clear();
        }
    }
}
