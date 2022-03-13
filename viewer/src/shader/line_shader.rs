use cgmath::{One, Vector4};
use imgui::im_str;

use crate::{
    shader::{set_mat4_array, set_vec4},
    Mat4,
};

use super::{compile_shader, Shader, UniformVariable};

const SHADER_STEM_NAME: &str = "line";

pub struct LineShader {
    id: u32,
    model_mats: UniformVariable<Vec<Mat4>>,
    color: UniformVariable<Vector4<f32>>,
    dummy: UniformVariable<Mat4>, // only for implement `Shader` trait.
}

impl LineShader {
    pub fn new() -> Self {
        let id = compile_shader(SHADER_STEM_NAME);
        let mut obj = LineShader {
            id,
            model_mats: UniformVariable::new("uModel", vec![Mat4::one(); 2]),
            color: UniformVariable::new("uColor", Vector4::<f32>::new(1.0, 0.0, 0.0, 1.0)),
            dummy: UniformVariable::new("DUMMY", Mat4::one()),
        };
        obj.update_model_mats(Mat4::one(), Mat4::one());
        obj
    }

    pub fn update_model_mats(&mut self, lhs_mat: Mat4, rhs_mat: Mat4) {
        lhs_mat[0][0] *= 0.5;
        lhs_mat[3][0] = lhs_mat[3][0] * 0.5 - 0.5;
        rhs_mat[0][0] *= 0.5;
        rhs_mat[3][0] = rhs_mat[3][0] * 0.5 + 0.5;
        self.model_mats.value[0] = lhs_mat;
        self.model_mats.value[1] = rhs_mat;
    }
}

impl Shader for LineShader {
    fn get_id(&self) -> u32 {
        self.id
    }

    /// !!!Do not use this function!!!
    fn get_model_mat(&self) -> &UniformVariable<Mat4> {
        &self.dummy
    }

    fn set_uniform_variables(
        &self,
        view_mat: &UniformVariable<Mat4>,
        proj_mat: &UniformVariable<Mat4>,
    ) {
        let id = self.get_id();
        unsafe {
            gl::UseProgram(id);
            set_vec4(self.id, &self.color);
            set_mat4_array(self.id, &self.model_mats);
            super::set_mat4(id, view_mat);
            super::set_mat4(id, proj_mat);
        }
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui) {
        imgui::Slider::new(im_str!("Line Color (R)"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.color.value.x);
        imgui::Slider::new(im_str!("Line Color (G)"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.color.value.y);
        imgui::Slider::new(im_str!("Line Color (B)"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.color.value.z);
        imgui::Slider::new(im_str!("Line Alpha"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.color.value.w);
    }
}
