use cgmath::One;
use gl;

use std::ffi::CString;
use std::str;

use crate::{utility::scale_matrix, Matrix4};

use super::{compile_shader, set_mat4, UniformVariable};

pub struct ImageShader {
    pub id: u32,
    pub model_mat: UniformVariable<Matrix4>,
    pub view_mat: UniformVariable<Matrix4>,
    pub projection_mat: UniformVariable<Matrix4>,
    pub is_dragging: bool, // 画像をdrag中かどうか
}

impl ImageShader {
    pub fn new(shader_path_stem: &str) -> Self {
        let id = compile_shader(shader_path_stem);
        let model_mat = UniformVariable {
            name: CString::new("uModel").unwrap(),
            value: Matrix4::one(),
        };
        let view_mat = UniformVariable {
            name: CString::new("uView").unwrap(),
            value: Matrix4::one(),
        };
        let projection_mat = UniformVariable {
            name: CString::new("uProjection").unwrap(),
            value: Matrix4::one(),
        };
        ImageShader {
            id,
            model_mat,
            view_mat,
            projection_mat,
            is_dragging: false,
        }
    }

    pub fn change_shader_program(&mut self, shader_path_stem: &str) {
        self.id = compile_shader(shader_path_stem);
    }

    /// 元画像のaspect ratioが保存されるようにmodel matrixを調整する
    pub fn adjust_aspect_ratio(
        &mut self,
        image_width: u32,
        image_height: u32,
        screen_width: u32,
        screen_height: u32,
    ) {
        let aspect_ratio =
            image_height as f32 * screen_width as f32 / (image_width as f32 * screen_height as f32);
        match aspect_ratio < 1.0f32 {
            true => {
                self.model_mat.value[1][1] = self.model_mat.value[0][0] * aspect_ratio;
            }
            false => {
                self.model_mat.value[0][0] = self.model_mat.value[1][1] / aspect_ratio;
            }
        }
    }

    pub fn on_mouse_wheel(&mut self, cx: f32, cy: f32, scale: f32) {
        self.model_mat.value = scale_matrix(&self.model_mat.value, cx, cy, scale);
    }

    ///
    pub fn on_mouse_motion_event(&mut self, xrel: f32, yrel: f32) {
        if self.is_dragging {
            self.model_mat.value[3][0] += xrel;
            self.model_mat.value[3][1] += yrel;
        }
    }

    /// mouseが画像をクリックしたか判定する
    pub fn on_mouse_button_down(
        &mut self,
        x: f32, // -1.0 to 1.0
        y: f32, // -1.0 to 1.0
    ) {
        let nx = x - self.model_mat.value[3][0];
        let ny = y - self.model_mat.value[3][1];
        self.is_dragging =
            (nx.abs() <= self.model_mat.value[0][0]) && (ny.abs() <= self.model_mat.value[1][1]);
    }

    pub fn on_mouse_button_up(&mut self) {
        self.is_dragging = false;
    }

    /// glslのuniform変数をセットする
    pub fn set_uniform_variables(&self) {
        unsafe {
            gl::UseProgram(self.id);
            set_mat4(self.id, &self.model_mat);
            set_mat4(self.id, &self.view_mat);
            set_mat4(self.id, &self.projection_mat);
        }
    }

    pub fn get_shader_id(&self) -> u32 {
        self.id
    }
}
