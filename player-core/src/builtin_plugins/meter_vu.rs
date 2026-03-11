use std::any::Any;
use plugin_api::AudioPlugin;

#[derive(Clone, Debug)]
pub struct VuMeter {
    value: f32,
    alpha: f32,
}

impl VuMeter {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            alpha: 0.05,
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

impl AudioPlugin for VuMeter {
    fn name(&self) -> &'static str {
        "VU Meter"
    }

    fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        let mono = ((l * l + r * r) * 0.5).sqrt();
        self.value = self.alpha * mono + (1.0 - self.alpha) * self.value;
        (l, r)
    }

    fn reset(&mut self) {
        self.value = 0.0;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_param(&self, name: &str) -> Option<f64> {
        match name {
            "value" => Some(self.value as f64),
            "alpha" => Some(self.alpha as f64),
            _ => None,
        }
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "value" => self.value = value as f32,
            "alpha" => self.alpha = value as f32,
            _ => {}
        }
    }
}