use cgmath::Vector4;

use super::{
    compile_shader, image_shader::ImageShader, set_float, set_mat4, set_vec4, UniformVariable,
};

pub struct ArrowLineShader {
    pub id: u32,
    pub color: UniformVariable<Vector4<f32>>,
    pub scale: UniformVariable<f32>,
}

impl ArrowLineShader {
    pub fn new(shader_path_stem: &str) -> Self {
        let id = compile_shader(shader_path_stem);
        ArrowLineShader {
            id,
            color: UniformVariable::new("uColor", Vector4::<f32>::new(1.0, 0.0, 0.0, 1.0)),
            scale: UniformVariable::new("uScale", 1.0),
        }
    }

    pub fn set_uniform_variables(&self, img_shader: &ImageShader) {
        unsafe {
            gl::UseProgram(self.id);
            set_mat4(self.id, &img_shader.model_mat);
            set_mat4(self.id, &img_shader.view_mat);
            set_mat4(self.id, &img_shader.projection_mat);
            set_vec4(self.id, &self.color);
            set_float(self.id, &self.scale);
        }
    }
}
