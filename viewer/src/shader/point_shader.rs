use std::ffi::CString;

use super::{compile_shader, image_shader::ImageShader, set_float, set_mat4, UniformVariable};

pub struct PointShader {
    id: u32,
    pub point_size: UniformVariable<f32>,
}

impl PointShader {
    pub fn new(shader_path_stem: &str) -> Self {
        let id = compile_shader(shader_path_stem);
        let point_size = UniformVariable {
            name: CString::new("uPointSize").unwrap(),
            value: 10.0f32,
        };
        PointShader { id, point_size }
    }

    pub fn set_uniform_variables(&self, img_shader: &ImageShader) {
        unsafe {
            gl::UseProgram(self.id);
            set_mat4(self.id, &img_shader.model_mat);
            set_mat4(self.id, &img_shader.view_mat);
            set_mat4(self.id, &img_shader.projection_mat);
            set_float(self.id, &self.point_size);
        }
    }

    pub fn get_shader_id(&self) -> u32 {
        return self.id;
    }
}
