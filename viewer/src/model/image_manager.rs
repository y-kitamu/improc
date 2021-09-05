use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use image::DynamicImage;
use log::warn;

use crate::{define_gl_primitive, draw};

use super::{create_simple_vertex, image::Image, GLPrimitive};

/// Textureに登録した画像を管理する。
/// 画像は左下が原点(pointerの開始地点)になるように、適当にflipする
/// 外部から`model` moduleにaccessするためのinterface. (`ImageManager`以外はprivateにする)
pub struct ImageManager {
    images: HashMap<String, Image>,
    vao: Option<u32>,
    vbo: Option<u32>,
    vertex_num: i32,
}

define_gl_primitive!(ImageManager);

impl ImageManager {
    pub fn new() -> ImageManager {
        let (vao, vbo, vertex_num) = create_simple_vertex();
        ImageManager {
            images: HashMap::new(),
            vao: Some(vao),
            vbo: Some(vbo),
            vertex_num,
        }
    }

    pub fn build(mut self) -> Self {
        self.images.iter_mut().for_each(|(_key, val)| {
            val.build();
        });
        self
    }

    pub fn draw_image(&self, img_key: &str) {
        let image = self.images.get(img_key).unwrap();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, image.id());
            draw!(self, gl::TRIANGLES);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn draw_points(&self, img_key: &str) {
        self.images.get(img_key).unwrap().draw_objects();
    }

    pub fn draw_point_relations(&self, lhs_key: &str, rhs_key: &str) {
        self.images
            .get(lhs_key)
            .unwrap()
            .draw_point_relations(rhs_key);
    }

    pub fn load_image(&mut self, path: &Path, id: &str, vflip: bool) -> Result<()> {
        let mut image = image::open(path)?;
        if vflip {
            image = image.flipv();
        }
        self.add_image(&image, id);
        Ok(())
    }

    /// 画像をtextureに追加する。
    /// 画像のポインタの先頭が画像の左下であると想定している。
    pub fn add_image(&mut self, image: &DynamicImage, key: &str) {
        if self.images.contains_key(key) {
            warn!(
                "Image key {} already exist in `images`. Skip add image.",
                key
            );
            return;
        }
        self.images.insert(key.to_string(), Image::new(key, image));
    }

    /// `ImageManager`に登録済みの画像のkeyの一覧を取得する
    pub fn get_image_keys(&self) -> std::collections::hash_map::Keys<String, Image> {
        self.images.keys()
    }

    /// `key`で指定した画像のtexture id(OpenGLの`gl::BindTexture`で指定するid)を取得する
    pub fn get_texture_id(&self, key: &str) -> u32 {
        match self.images.get(key) {
            Some(image) => image.id(),
            None => 0,
        }
    }

    /// `key`で指定した画像のtexture size(画像サイズ)を取得する
    pub fn get_texture_image_size(&self, key: &str) -> (u32, u32) {
        match self.images.get(key) {
            Some(image) => (image.w(), image.h()),
            None => (1u32, 1u32),
        }
    }

    /// add point (`x`, `y`, `z`) to image of `image_id`.
    /// Argument `x` and `y` are treated as point on the image coordinate system.
    /// A value range of `z` is from -1.0 to 1.0.
    /// Argument `r`, `g` and `b` are pixel values range from 0.0 to 1.0.
    pub fn add_point(&mut self, image_id: &str, x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) {
        let image = self.images.remove(image_id).unwrap();
        let image = image.add_point(x, y, z, r, g, b);
        self.images.insert(image_id.to_string(), image);
    }

    pub fn add_point_relation(
        &mut self,
        lhs_key: &str,
        lx: f32,
        ly: f32,
        rhs_key: &str,
        rx: f32,
        ry: f32,
    ) {
        let image = self.images.remove(lhs_key).unwrap().add_point_relation(
            lx,
            ly,
            self.images.get(rhs_key).unwrap(),
            rx,
            ry,
        );
        self.images.insert(lhs_key.to_string(), image);
        let image = self.images.remove(rhs_key).unwrap().add_point_relation(
            rx,
            ry,
            self.images.get(lhs_key).unwrap(),
            lx,
            ly,
        );
        self.images.insert(rhs_key.to_string(), image);
    }
}

#[cfg(test)]
mod tests {
    use super::super::{arrow::Arrows, point::Points};

    use super::*;

    #[test]
    fn test_image_manager() {
        let mut manager = ImageManager {
            images: HashMap::new(),
            vao: None,
            vbo: None,
            vertex_num: 0,
        };

        assert!(manager.images.is_empty());

        assert_eq!(manager.get_image_keys().len(), 0);
    }
}
