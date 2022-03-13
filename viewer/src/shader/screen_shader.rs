use cgmath::One;

use crate::Mat4;

use super::{compile_shader, Shader, UniformVariable};

const SHADER_STEM_NAME: &str = "screen";

/// (off screen renderingの)画面表示用のshader
pub struct ScreenShader {
    id: u32,
    dummy: UniformVariable<Mat4>, // Dummy data used in trait method.
}

impl ScreenShader {
    pub fn new() -> Self {
        let id = compile_shader(SHADER_STEM_NAME);
        ScreenShader {
            id,
            dummy: UniformVariable::new("dummy", Mat4::one()),
        }
    }
}

impl Shader for ScreenShader {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_model_mat(&self) -> &UniformVariable<Mat4> {
        &self.dummy
    }
}
