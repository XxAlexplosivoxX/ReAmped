use std::any::Any;

use plugin_api::AudioPlugin; // import usage

#[derive(Clone, Debug)]
pub struct RmsMeter {
    energy: f64,
    count: usize,
}

impl RmsMeter {
    pub fn new() -> Self {
        Self {
            energy: 0.0,
            count: 0,
        }
    }

    pub fn value(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }

        let mean = self.energy / self.count as f64;

        mean.sqrt() as f32
    }
}

impl AudioPlugin for RmsMeter {
    fn name(&self) -> &'static str {
        "RMS Meter"
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

    fn get_param(&self, _name: &str) -> Option<f64> {
        match _name {
            "count" => Some(self.count as f64),
            "energy" => Some(self.energy),
            "value" => Some(self.value() as f64),
            _ => None,
        }
    }
    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "count" => self.count = value as usize,
            "energy" => self.energy = value,
            _ => {
                // nada pq no hay parámetro xdd
            }
        }
    }
}
