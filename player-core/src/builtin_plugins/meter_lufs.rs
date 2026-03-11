use std::any::Any;
use plugin_api::AudioPlugin;

#[derive(Clone, Debug)]
pub struct LufsMeter {
    energy: f64,
    count: usize,
}

impl LufsMeter {
    pub fn new() -> Self {
        Self {
            energy: 0.0,
            count: 0,
        }
    }

    pub fn value(&self) -> f32 {
        if self.count == 0 {
            return -70.0;
        }
        let mean = (self.energy / self.count as f64).max(1e-12);
        (-0.691 + 10.0 * mean.log10()) as f32
    }
}

impl AudioPlugin for LufsMeter {
    fn name(&self) -> &'static str {
        "LUFS Meter"
    }

    fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        let l = l as f64;
        let r = r as f64;
        self.energy += (l * l + r * r) * 0.5;
        self.count += 1;
        (l as f32, r as f32)
    }

    fn reset(&mut self) {
        self.energy = 0.0;
        self.count = 0;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_param(&self, name: &str) -> Option<f64> {
        match name {
            "energy" => Some(self.energy),
            "count" => Some(self.count as f64),
            "value" => Some(self.value() as f64),
            _ => None,
        }
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "energy" => self.energy = value,
            "count" => self.count = value as usize,
            _ => {}
        }
    }
}