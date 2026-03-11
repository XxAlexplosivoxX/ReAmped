use plugin_api::AudioPlugin;
// use std::any::Any;

pub struct PluginChain {
    plugins: Vec<Box<dyn AudioPlugin>>,
}

impl PluginChain {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn add(&mut self, plugin: Box<dyn AudioPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn process(&mut self, mut l: f32, mut r: f32) -> (f32, f32) {
        for p in self.plugins.iter_mut() {
            let (nl, nr) = p.process(l, r);
            l = nl;
            r = nr;
        }

        (l, r)
    }

    pub fn plugins_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn AudioPlugin>> {
        self.plugins.iter_mut()
    }
    pub fn collect_values(&mut self) -> Vec<(String, f32)> {
        let mut out = Vec::new();

        for p in &mut self.plugins {
            if let Some(v) = p.get_param("value") {
                out.push((p.name().to_string(), v as f32));
            }
        }

        out
    }
    // ================= acceso a parámetros por índice =================
    pub fn get_plugin(&mut self, index: usize) -> Option<&mut Box<dyn AudioPlugin>> {
        self.plugins.get_mut(index)
    }

    // ================= acceso a plugin por tipo =================
    pub fn get_plugin_of_type<T: AudioPlugin + 'static>(&mut self) -> Option<&mut T> {
        for p in self.plugins.iter_mut() {
            if let Some(p_typed) = p.as_any_mut().downcast_mut::<T>() {
                return Some(p_typed);
            }
        }
        None
    }
}
