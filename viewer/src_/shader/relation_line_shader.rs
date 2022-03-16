use std::ffi::CString;

use cgmath::Vector4;

use crate::shader::{set_mat4_array, set_vec4};

use super::{compile_shader, image_shader::ImageShader, UniformVariable};

pub struct RelationLineShader {
    pub id: u32,
    pub color: UniformVariable<Vector4<f32>>,
}

impl RelationLineShader {
    pub fn new(shader_path_stem: &str) -> Self {
        let id = compile_shader(shader_path_stem);
        RelationLineShader {
            id,
            color: UniformVariable::new("uColor", Vector4::<f32>::new(1.0, 0.0, 0.0, 1.0)),
        }
    }

    pub fn set_uniform_variables(&self, lhs: &ImageShader, rhs: &ImageShader) {
        let mut model = UniformVariable {
            name: CString::new("uModel").unwrap(),
            value: vec![lhs.model_mat.value, rhs.model_mat.value],
        };
        (&mut model).value[0][0][0] *= 0.5;
        (&mut model).value[0][3][0] *= 0.5;
        (&mut model).value[0][3][0] -= 0.5;
        (&mut model).value[1][0][0] *= 0.5;
        (&mut model).value[1][3][0] *= 0.5;
        (&mut model).value[1][3][0] += 0.5;
        let view = UniformVariable {
            name: CString::new("uView").unwrap(),
            value: vec![lhs.view_mat.value, rhs.view_mat.value],
        };
        let projection = UniformVariable {
            name: CString::new("uProjection").unwrap(),
            value: vec![lhs.projection_mat.value, rhs.projection_mat.value],
        };

        unsafe {
            gl::UseProgram(self.id);
            set_vec4(self.id, &self.color);
            set_mat4_array(self.id, &model);
            set_mat4_array(self.id, &view);
            set_mat4_array(self.id, &projection);
        }
    }
}