//! Image viewer app.
//! The program structure of this app is based on the MVP architecture.
//! `image_manager` module is Model of MVP,
//! `viewer` module is View of MVP, and
//! `presenter` module is Presenter of MVP.
//! `app` module is user interface.
//! `shader` module prepare and render glsl shader.

/// `$e`(Array of String)で指定したshaderをcompileし、
/// 対応するstruct(`$t`)のobjectのhashmap (key: String, val: `$t`)を返す.
macro_rules! load_shaders {
    ($e:expr, $t:ty) => {{
        let mut shader_map = HashMap::<String, $t>::new();
        $e.iter().for_each(|key| {
            let shader = <$t>::new(&key);
            shader_map.insert(key.to_string(), shader);
        });
        shader_map
    }};
}

pub mod app;
mod image_manager;
mod presenter;
mod shader;
mod vertex;
mod viewer;
