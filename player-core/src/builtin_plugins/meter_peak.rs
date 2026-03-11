use std::any::Any;
use plugin_api::AudioPlugin;

#[derive(Clone, Debug)]
pub struct PeakMeter {
    peak: f32,
}

impl PeakMeter {
    pub fn new() -> Self {
        Self { peak: 0.0 }
    }

    pub fn value(&self) -> f32 {
        self.peak
    }
}

impl AudioPlugin for PeakMeter {
    fn name(&self) -> &'static str {
        "Peak Meter"
    }

    fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        let p = l.abs().max(r.abs());
        if p > self.peak {
            self.peak = p;
        }
        (l, r)
    }

    fn reset(&mut self) {
        self.peak = 0.0;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_param(&self, name: &str) -> Option<f64> {
        match name {
            "peak" => Some(self.peak as f64),
            "value" => Some(self.value() as f64),
            _ => None,
        }
    }

    fn set_param(&mut self, name: &str, value: f64) {
        if name == "peak" {
            self.peak = value as f32;
        }
    }
}