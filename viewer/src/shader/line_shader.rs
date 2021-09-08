use std::ffi::CString;

use crate::{
    shader::{set_mat4_array, set_vec3},
    Vector3,
};

use super::{compile_shader, image_shader::ImageShader, set_float, set_mat4, UniformVariable};

pub struct LineShader {
    id: u32,
    pub color: UniformVariable<Vector3>,
}

impl LineShader {
    pub fn new(shader_path_stem: &str) -> Self {
        let id = compile_shader(shader_path_stem);
        LineShader {
            id,
            color: UniformVariable::new("uColor", Vector3::new(1.0, 0.0, 0.0)),
        }
    }

    pub fn set_uniform_variables(&self, img_shader: &ImageShader) {
        unsafe {
            gl::UseProgram(self.id);
            set_mat4(self.id, &img_shader.model_mat);
            set_mat4(self.id, &img_shader.view_mat);
            set_mat4(self.id, &img_shader.projection_mat);
            set_vec3(self.id, &self.color);
        }
    }
}
