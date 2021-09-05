//! Image viewer app.
//! The program structure of this app is based on the MVP architecture.
//! `model` module is Model of MVP,
//! `view` module is View of MVP, and
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

type Vector3 = cgmath::Vector3<f32>;
type Matrix4 = cgmath::Matrix4<f32>;

pub mod app;
mod model;
mod presenter;
mod shader;
mod utility;
mod view;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    struct Test {
        key: String,
    }

    impl Test {
        fn new(key: &str) -> Self {
            Test {
                key: key.to_string(),
            }
        }
    }

    #[test]
    fn test_load_shader_macro() {
        let map = load_shaders!(vec!["foo", "bar"], Test);
        assert_eq!(&map["foo"].key, "foo");
        assert_eq!(&map["bar"].key, "bar");
    }
}
