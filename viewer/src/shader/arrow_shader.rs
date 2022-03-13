use cgmath::{One, Vector4};
use imgui::im_str;

use crate::Mat4;

use super::{compile_shader, set_float, set_mat4, set_vec4, Shader, UniformVariable};

const SHADER_STEM_NAME: &str = "arrow";

pub struct ArrowShader {
    id: u32,
    model_mat: UniformVariable<Mat4>,
    color: UniformVariable<Vector4<f32>>,
    scale: UniformVariable<f32>,
}

impl ArrowShader {
    pub fn new() -> Self {
        let id = compile_shader(SHADER_STEM_NAME);
        ArrowShader {
            id,
            model_mat: UniformVariable::new("uModel", Mat4::one()),
            color: UniformVariable::new("uColor", Vector4::<f32>::new(1.0, 0.0, 0.0, 1.0)),
            scale: UniformVariable::new("uScale", 1.0),
        }
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color.value = Vector4::<f32>::new(r, g, b, a);
    }
}

impl Shader for ArrowShader {
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
        let id = self.get_id();
        unsafe {
            gl::UseProgram(id);
            set_mat4(id, self.get_model_mat());
            set_mat4(id, view_mat);
            set_mat4(id, proj_mat);
            set_vec4(id, &self.color);
            set_float(id, &self.scale);
        }
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui) {
        imgui::Slider::new(im_str!("Line Scale"))
            .range(0.1..=100.0)
            .build(&ui, &mut self.scale.value);

        imgui::Slider::new(im_str!("Arrow Color (R)"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.color.value[0]);
        imgui::Slider::new(im_str!("Arrow Color (G)"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.color.value[1]);
        imgui::Slider::new(im_str!("Arrow Color (B)"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.color.value[2]);
        imgui::Slider::new(im_str!("Arrow Alpha"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.color.value[3]);
    }
}
