use cgmath::One;
use sdl2::mouse::MouseWheelDirection;

use crate::{utility::scale_matrix, Mat4};

use super::{compile_shader, Shader, UniformVariable};

const SHADER_STEM_NAME: &str = "image";

/// 画像表示用のshader. model::drawables::Imageで使用される
pub struct ImageShader {
    id: u32,
    model_mat: UniformVariable<Mat4>,
    is_dragging: bool, // 画像をdrag中かどうか
}

impl ImageShader {
    /// Constructor.
    /// Compile fragment shader (./glsl/image.fs) and vertex shader (./glsl/image.vs).
    pub fn new() -> Self {
        let id = compile_shader(SHADER_STEM_NAME);
        let model_mat = UniformVariable::new("uModel", Mat4::one());
        ImageShader {
            id,
            model_mat,
            is_dragging: false,
        }
    }

    /// Adjust model matrix so that aspect ratio of the original image is preserved.
    fn adjust_aspect_ratio(
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
}

impl Shader for ImageShader {
    fn get_id(&self) -> u32 {
        self.id
    }
    fn get_model_mat(&self) -> &UniformVariable<Mat4> {
        &self.model_mat
    }

    fn on_mouse_wheel(&mut self, cx: f32, cy: f32, y: &i32, direction: &MouseWheelDirection) {
        let mut scale = 1.0f32 + *y as f32 / 10.0f32;
        if *direction == MouseWheelDirection::Flipped {
            scale = 1.0f32 / scale;
        }
        self.model_mat.value = scale_matrix(&self.model_mat.value, cx, cy, scale);
    }

    /// If `is_dragging` is true, move image from (x, y) to (x + `xrel`, y + `yrel`)
    fn on_mouse_motion_event(&mut self, xrel: f32, yrel: f32) {
        if self.is_dragging {
            self.model_mat.value[3][0] += xrel;
            self.model_mat.value[3][1] += yrel;
        }
    }

    /// If mouse position is on the image, set `is_dragging` to true.
    fn on_mouse_button_down(&mut self, x: f32, y: f32) {
        let nx = x - self.model_mat.value[3][0];
        let ny = y - self.model_mat.value[3][1];
        self.is_dragging =
            (nx.abs() <= self.model_mat.value[0][0]) && (ny.abs() <= self.model_mat.value[1][1]);
    }

    /// Set `is_dragging` to false.
    fn on_mouse_button_up(&mut self) {
        self.is_dragging = false;
    }
}
