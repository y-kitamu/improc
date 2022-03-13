use cgmath::One;
use imgui::im_str;

use crate::Mat4;

use super::{compile_shader, set_float, set_mat4, Shader, UniformVariable};

const SHADER_STEM_NAME: &str = "point";

pub struct PointShader {
    id: u32,
    model_mat: UniformVariable<Mat4>,
    point_size: UniformVariable<f32>,
}

impl PointShader {
    pub fn new() -> Self {
        let id = compile_shader(SHADER_STEM_NAME);
        let model_mat = UniformVariable::new("uModel", Mat4::one());
        let point_size = UniformVariable::new("unitizes", 10.0f32);
        PointShader {
            id,
            model_mat,
            point_size,
        }
    }

    pub fn set_point_size(&mut self, pt_size: f32) {
        self.point_size.value = pt_size;
    }

    pub fn get_point_size(&self) -> f32 {
        self.point_size.value
    }

    pub fn update_model_mat(&mut self, model_mat: UniformVariable<Mat4>) {
        self.model_mat = model_mat;
    }
}

impl Shader for PointShader {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_model_mat(&self) -> &UniformVariable<Mat4> {
        &self.model_mat
    }

    fn set_uniform_variables(
        &self,
        view_mat: &UniformVariable<Mat4>,
        proj_mat: &UniformVariable<Mat4>,
    ) {
        unsafe {
            gl::UseProgram(self.id);
            set_mat4(self.id, self.get_model_mat());
            set_mat4(self.id, view_mat);
            set_mat4(self.id, proj_mat);
            set_float(self.id, &self.point_size);
        }
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui) {
        imgui::Slider::new(im_str!("Point size"))
            .range(1.0..=100.0)
            .build(&ui, &mut self.point_size.value);
    }
}
