use std::any::Any;
use plugin_api::AudioPlugin;


#[derive(Clone, Debug)]
pub struct TruePeakMeter {
    peak: f32,
    last_l: f32,
    last_r: f32,
}

impl TruePeakMeter {
    pub fn new() -> Self {
        Self {
            peak: 0.0,
            last_l: 0.0,
            last_r: 0.0,
        }
    }

    pub fn value(&self) -> f32 {
        self.peak
    }
}

impl AudioPlugin for TruePeakMeter {
    fn name(&self) -> &'static str {
        "True Peak"
    }

    fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        let interp_l = (l + self.last_l) * 0.5;
        let interp_r = (r + self.last_r) * 0.5;

        let peak = l.abs()
            .max(r.abs())
            .max(interp_l.abs())
            .max(interp_r.abs());

        if peak > self.peak {
            self.peak = peak;
        }

        self.last_l = l;
        self.last_r = r;

        (l, r)
    }

    fn reset(&mut self) {
        self.peak = 0.0;
        self.last_l = 0.0;
        self.last_r = 0.0;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_param(&self, name: &str) -> Option<f64> {
        match name {
            "peak" => Some(self.peak as f64),
            "last_l" => Some(self.last_l as f64),
            "last_r" => Some(self.last_r as f64),
            "value" => Some(self.value() as f64),
            _ => None,
        }
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "peak" => self.peak = value as f32,
            "last_l" => self.last_l = value as f32,
            "last_r" => self.last_r = value as f32,
            _ => {}
        }
    }
}