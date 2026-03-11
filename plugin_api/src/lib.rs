use std::any::Any;

/**
THIS IS A TRAIT TO MAKE PLUGINS FOR REAMPED
compile a .so with
```
[lib]
crate-type = ["cdylib"]
```
in the Cargo.toml for dynamic load
*/
pub trait AudioPlugin: Send {
    /** Name for the plugin */
    fn name(&self) -> &'static str;
    /** This is for process the audio and make "things" with that */
    fn process(&mut self, left: f32, right: f32) -> (f32, f32);
    /** This is to reset the values you use in ```process(&mut self, left: f32, right: f32)``` */
    fn reset(&mut self) {}
    /** This is to return the plugin to make usable in reamped */
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /** Access for parameters */
    fn params(&self) -> Vec<(&'static str, f64)> {
        vec![]
    }

    /** Access for parameters */
    fn get_param(&self, _name: &str) -> Option<f64> {
        None
    }
    /** Access for parameters */
    fn set_param(&mut self, _name: &str, _value: f64) {}
}

pub type PluginCreateFn = unsafe extern "C" fn() -> *mut dyn AudioPlugin;
