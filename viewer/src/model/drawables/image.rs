use std::{cell::Cell, ffi::c_void};

use image::{DynamicImage, EncodableLayout};

use crate::{
    model::{drawables::create_simple_vertex, Drawable},
    shader::{image_shader::ImageShader, Shader},
};

/// 画像の描画に必要な情報、画像上の点の情報を保持するstruct.
/// `points`に保持される点は正規化座標系上の点である。
/// (画像の左下を(-1.0, -1.0)、右上を(1.0, 1.0)で中心を(0, 0)とする座標系)
/// ただし、functionの引数ではimage coordinate(画像のpixel単位の座標)を使用する。
pub struct Image {
    key: String,                   // display name of the image.
    texture_id: u32,               // texture id of the image.
    vao: u32,                      // vertex array object id of the image.
    vertex_num: u32,               // Number of vertex in the image.
    shader: Cell<Box<dyn Shader>>, // shader object
    width: u32,                    // image width
    height: u32,                   // image height
    draw_flag: bool,               // If true draw object, else not.
    associated_drawables: Vec<Box<dyn Drawable>>,
}

impl Image {
    /// 画像をtextureに追加する。
    pub fn new(key: &str, image: &DynamicImage) -> Box<Image> {
        let image = image.flipv().to_rgb8(); // ポインタの先頭が画像の左下に来るようにflip, rgbに変換
        let format = gl::RGB;
        let data = image.as_bytes();
        // create texture
        let mut texture_id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
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
        // create vetex
        let (vao, _, vertex_num) = create_simple_vertex();
        println!("Register image : key = {}, index = {}", key, texture_id);
        Box::new(Image {
            key: key.to_string(),
            texture_id,
            vao,
            vertex_num,
            shader: Cell::new(Box::new(ImageShader::new())),
            width: image.width(),
            height: image.height(),
            draw_flag: false,
            associated_drawables: Vec::new(),
        })
    }

    pub fn convert_to_norm_coord(&self, x: f32, y: f32) -> (f32, f32) {
        let x = x / self.width as f32 * 2.0 - 1.0;
        let y = 1.0 - y / self.height as f32 * 2.0;
        (x, y)
    }
}

impl Drawable for Image {
    fn get_drawable_type(&self) -> super::DrawableType {
        super::DrawableType::Image
    }

    fn get_vertex_num(&self) -> u32 {
        self.vertex_num
    }

    fn get_draw_type(&self) -> gl::types::GLenum {
        gl::TRIANGLES
    }

    fn get_model_mat(&mut self) -> crate::Mat4 {
        self.shader.get_mut().get_model_mat().value.clone()
    }

    fn get_mut_shader(&mut self) -> &mut Box<dyn crate::shader::Shader> {
        self.shader.get_mut()
    }

    fn get_associated_drawables(&mut self) -> &Vec<Box<dyn Drawable>> {
        &self.associated_drawables
    }

    fn get_mut_associated_drawables(&mut self) -> &mut Vec<Box<dyn Drawable>> {
        &mut self.associated_drawables
    }

    fn is_draw(&self) -> bool {
        self.draw_flag
    }

    fn set_is_draw(&mut self, flag: bool) {
        self.draw_flag = flag
    }

    fn get_vao(&self) -> u32 {
        self.vao
    }

    fn get_texture_id(&self) -> u32 {
        self.texture_id
    }
}
