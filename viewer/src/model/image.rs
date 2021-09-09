use std::{collections::HashMap, ffi::c_void};

use image::{DynamicImage, GenericImageView};
use log::warn;

use crate::{draw, shader::image_shader::ImageShader, utility::convert_to_rgb};

use super::{arrow::Arrows, point::Points, point_relation::PointRelations, Drawable};

const DEFAULT_IMAGE_SHADER: &str = "default";

/// 画像の描画に必要な情報、画像上の点の情報を保持するstruct.
/// `points`に保持される点は正規化座標系上の点である。
/// (画像の左下を(-1.0, -1.0)、右上を(1.0, 1.0)で中心を(0, 0)とする座標系)
/// ただし、functionの引数ではimage coordinate(画像のpixel単位の座標)を使用する。
/// `points_vertex`は点をOpenGL描画するためのvao, vboを保持する
/// `point_relation_vertex`は画像間の直線をOptnGLで描画するためのvao, vboを保持する。
pub struct Image {
    pub key: String,
    pub texture_id: u32, // openglのtexture id
    pub image_shader: ImageShader,
    pub width: u32,
    pub height: u32,
    pub points: Points,
    pub arrows: Arrows,
    pub point_relations: HashMap<String, PointRelations>,
}

impl Image {
    /// 画像をtextureに追加する。
    /// 画像のポインタの先頭が画像の左下であると想定している。
    pub fn new(key: &str, image: &DynamicImage) -> Image {
        let image = image.flipv();
        let (format, image) = match image {
            DynamicImage::ImageLuma8(img) => {
                (gl::RGB, DynamicImage::ImageRgb8(convert_to_rgb(&img)))
            }
            DynamicImage::ImageLumaA8(img) => {
                (gl::RGB, DynamicImage::ImageRgb8(convert_to_rgb(&img)))
            }
            DynamicImage::ImageLuma16(img) => {
                (gl::RGB, DynamicImage::ImageRgb8(convert_to_rgb(&img)))
            }
            DynamicImage::ImageLumaA16(img) => {
                (gl::RGB, DynamicImage::ImageRgb8(convert_to_rgb(&img)))
            }
            DynamicImage::ImageRgb8(_)
            | DynamicImage::ImageBgr8(_)
            | DynamicImage::ImageRgb16(_) => (gl::RGB, image.clone()),
            DynamicImage::ImageRgba8(_)
            | DynamicImage::ImageBgra8(_)
            | DynamicImage::ImageRgba16(_) => (gl::RGBA, image.clone()),
        };
        let data = image.as_bytes();

        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                &data[0] as *const u8 as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        println!("Finish register image : key = {}, index = {}", key, texture);
        let image_shader = ImageShader::new(DEFAULT_IMAGE_SHADER);
        Image {
            key: key.to_string(),
            texture_id: texture,
            image_shader,
            width: image.width(),
            height: image.height(),
            points: Points::new(),
            arrows: Arrows::new(),
            point_relations: HashMap::new(),
        }
    }

    /// 画像(`Image`)に登録されている点群,矢印,直線をOpenGLに登録(vao, vboを作成)する
    pub fn build(&mut self) {
        self.points.build();
        self.arrows.build();
        self.point_relations.iter_mut().for_each(|(_key, val)| {
            val.build();
        });
    }

    pub fn draw_objects(
        &mut self,
        vao: u32,
        vertex_num: i32,
        screen_width: u32,
        screen_height: u32,
    ) {
        self.image_shader
            .adjust_aspect_ratio(self.width, self.height, screen_width, screen_height);
        self.image_shader.set_uniform_variables();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, vertex_num);
            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::UseProgram(0);
        }
        self.points.draw(&self.image_shader);
        self.arrows.draw(&self.image_shader);
    }

    pub fn draw_point_relations(&self, other_key: &str) {
        if let Some(rel) = self.point_relations.get(other_key) {
            rel.draw();
        }
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn shader(&self) -> &ImageShader {
        &self.image_shader
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn id(&self) -> u32 {
        self.texture_id
    }

    pub fn w(&self) -> u32 {
        self.width
    }

    pub fn h(&self) -> u32 {
        self.height
    }

    pub fn convert_to_norm_coord(&self, x: f32, y: f32) -> (f32, f32) {
        let x = x / self.width as f32 * 2.0 - 1.0;
        let y = 1.0 - y / self.height as f32 * 2.0;
        (x, y)
    }

    pub fn on_mouse_wheel(&mut self, x: f32, y: f32, scale: f32) {
        self.image_shader.on_mouse_wheel(x, y, scale);
    }

    pub fn on_mouse_button_down(&mut self, fx: f32, fy: f32) {
        self.image_shader.on_mouse_button_down(fx, fy);
    }

    pub fn on_mouse_button_up(&mut self) {
        self.image_shader.on_mouse_button_up();
    }

    pub fn on_mouse_motion_event(&mut self, dx: f32, dy: f32) {
        self.image_shader.on_mouse_motion_event(dx, dy);
    }

    /// 画像に点を追加する
    /// Argument `x` and `y` are treated as point on the image coordinate system.
    /// A value range of `z` is from -1.0 to 1.0.
    /// Argument `r`, `g` and `b` are pixel values range from 0.0 to 1.0.
    pub fn add_point(mut self, x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) -> Image {
        let (x, y) = self.convert_to_norm_coord(x, y);
        self.points.add_point(x, y, z, r, g, b);
        self
    }

    /// 画像に矢印を追加する
    /// Argument `x` and `y` are treated as point on the image coordinate system.
    /// Argument `direction` is radian of the arrow direction.
    pub fn add_arrow(mut self, x: f32, y: f32, direction: f32, length: f32) -> Image {
        let (x, y) = self.convert_to_norm_coord(x, y);
        let length = length / self.width as f32;
        self.arrows.add_arrow(x, y, direction, length);
        self
    }

    /// 画像に他の画像の点との関係(`relation`)を追加する
    /// Argument `x`, `y`, `other_x` and `other_y` are treated as point on
    /// the image coordinate system.
    pub fn add_point_relation(
        mut self,
        x: f32,
        y: f32,
        other_image: &Self,
        other_x: f32,
        other_y: f32,
    ) -> Image {
        let (other_nx, other_ny) = other_image.convert_to_norm_coord(other_x, other_y);
        let (nx, ny) = self.convert_to_norm_coord(x, y);
        if self.points.is_exist_point(nx, ny) {
            let key = other_image.key();
            if !self.point_relations.contains_key(key) {
                self.point_relations
                    .insert(key.to_string(), PointRelations::new());
            }
            self.point_relations
                .get_mut(key)
                .unwrap()
                .add_relation(nx, ny, other_nx, other_ny);
        } else {
            warn!(
                "No point is found in image id = `{}` at (x, y) = ({}, {}). Skip adding relation.",
                self.texture_id, x, y
            );
        }
        self
    }

    pub fn set_point_size(&mut self, pt_size: f32) {
        self.points.set_point_size(pt_size);
    }

    pub fn get_point_size(&self) -> f32 {
        self.points.get_point_size()
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use cgmath::One;

    use crate::{
        shader::{line_shader::LineShader, point_shader::PointShader, UniformVariable},
        Matrix4, Vector3,
    };

    use super::*;

    fn get_image_shader() -> ImageShader {
        ImageShader {
            id: 0,
            model_mat: UniformVariable {
                name: CString::new("uModel").unwrap(),
                value: Matrix4::one(),
            },
            view_mat: UniformVariable {
                name: CString::new("uView").unwrap(),
                value: Matrix4::one(),
            },
            projection_mat: UniformVariable {
                name: CString::new("uProjection").unwrap(),
                value: Matrix4::one(),
            },
            is_dragging: false,
        }
    }

    fn get_points() -> Points {
        Points {
            points: Vec::new(),
            vao: Some(1),
            vbo: Some(2),
            vertex_num: 12,
            shader: PointShader {
                id: 2,
                point_size: UniformVariable {
                    name: CString::new("point_size").unwrap(),
                    value: 10.0,
                },
            },
        }
    }

    fn get_arrows() -> Arrows {
        Arrows {
            vao: Some(2),
            vbo: Some(2),
            vertex_num: 20,
            arrows: Vec::new(),
            shader: LineShader {
                id: 0,
                color: UniformVariable {
                    name: CString::new("uColor").unwrap(),
                    value: Vector3::new(1.0, 0.0, 0.0),
                },
            },
        }
    }

    #[test]
    fn test_image() {
        let mut image = Image {
            key: "default".to_string(),
            texture_id: 0,
            image_shader: get_image_shader(),
            width: 1920,
            height: 1080,
            points: get_points(),
            arrows: get_arrows(),
            point_relations: HashMap::new(),
        };
        assert_eq!(image.id(), 0u32);
        assert_eq!(image.w(), 1920u32);
        assert_eq!(image.h(), 1080u32);
        assert_eq!(image.shader().id, 0u32);
        assert!(!image.shader().is_dragging);

        image.on_mouse_wheel(1.0, -0.5, 1.2);
        assert!((image.image_shader.model_mat.value[0][0] - 1.2).abs() < 1e-5);
        assert!((image.image_shader.model_mat.value[1][1] - 1.2).abs() < 1e-5);
        assert!(
            (image.image_shader.model_mat.value[3][0] + 0.2).abs() < 1e-5,
            "model_mat.value[3][0] = {}",
            image.image_shader.model_mat.value[3][0]
        );
        assert!((image.image_shader.model_mat.value[3][1] - 0.1).abs() < 1e-5);

        image.on_mouse_button_down(-1.3, 1.05);
        assert!(image.image_shader.is_dragging);
        image.on_mouse_motion_event(0.0, -0.05);
        assert!(
            (image.image_shader.model_mat.value[3][1] - 0.05).abs() < 1e-5,
            "model_mat.valule[3][1] = {}",
            image.image_shader.model_mat.value[3][1]
        );
        image.on_mouse_button_up();
        assert!(!image.image_shader.is_dragging);
        image.on_mouse_button_down(0.9, -1.17);
        assert!(!image.image_shader.is_dragging);

        image.set_point_size(20.0);
        assert!((image.get_point_size() - 20.0).abs() < 1e-5);

        let (x, y) = image.convert_to_norm_coord(960.0f32, 540.0f32);
        assert!((x - 0.0).abs() < 1e-5);
        assert!((y - 0.0).abs() < 1e-5);

        let image = image.add_point(1080.0f32, 1080.0f32, 0.1, 1.0, 1.0, 1.0);
        println!("points[0] = {:?}", image.points.points[0].loc);
        assert!(image
            .points
            .is_exist_point(1080.0 / 1920.0 * 2.0 - 1.0, -1.0));

        let other_key = "other";
        let other_img = Image {
            key: other_key.to_string(),
            texture_id: 1,
            image_shader: get_image_shader(),
            width: 1280,
            height: 1080,
            points: get_points(),
            arrows: get_arrows(),
            point_relations: HashMap::new(),
        };
        let image = image.add_point_relation(1200.0, 1080.0, &other_img, 540.0, 240.0);
        assert_eq!(image.point_relations.len(), 0);
        let image = image.add_point_relation(1080.0, 1080.0, &other_img, 540.0, 240.0);
        assert_eq!(image.point_relations.len(), 1);
    }
}
