use std::path::Path;
use std::{collections::HashMap, os::raw::c_void};

use anyhow::Result;
use image::{DynamicImage, GenericImageView};
use log::warn;

use crate::vertex::Vertex;

use super::image::Image;

/// Textureに登録した画像を管理する。
/// 画像は左下が原点(pointerの開始地点)になるように、適当にflipする
pub struct ImageManager {
    images: HashMap<String, Image>,
    is_build: bool,
}

impl ImageManager {
    pub fn new() -> ImageManager {
        let image_manager = ImageManager {
            images: HashMap::new(),
            is_build: false,
        };
        image_manager
    }

    pub fn build(self) -> Self {
        self.build_points_vertex().build_point_relation()
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
    pub fn add_image(&mut self, image: &DynamicImage, id: &str) {
        let image = image.flipv();
        let id = id.to_string();
        if self.images.contains_key(&id) {
            warn!(
                "Image key {} already exist in `images`. Skip add image.",
                id
            );
            return;
        }
        let format = match image {
            image::DynamicImage::ImageLuma8(_) => gl::RED,
            image::DynamicImage::ImageLumaA8(_) => gl::RG,
            image::DynamicImage::ImageRgb8(_) => gl::RGB,
            image::DynamicImage::ImageRgba8(_) => gl::RGBA,
            image::DynamicImage::ImageBgr8(_) => gl::RGB,
            image::DynamicImage::ImageBgra8(_) => gl::RGBA,
            _ => gl::RGB,
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
        println!("Finish register image : id = {}, index = {}", id, texture);
        self.images.insert(
            id.clone(),
            Image::new(&id, texture, image.width(), image.height()),
        );
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

    /// `key`で指定した画像の頂点情報(`Vertex`)を取得する
    pub fn get_points_vertex(&self, key: &str) -> &Option<Vertex> {
        // TODO: refactor (is_buildは`get_points_vertex`のoption判定で十分?)
        if !self.is_build {
            warn!("`ImageManager` has not been built. `build_points_vertex` should be called.")
        }
        self.images.get(key).unwrap().get_points_vertex()
    }

    /// `lhs_key`, `rhs_key`で指定した画像間のpoint relationのVertexを取得する
    /// `lhs_key`, `rhs_key`の順番を逆にすると正しく表示されなくなるので注意する。
    pub fn get_point_relation(&self, lhs_key: &str, rhs_key: &str) -> Option<&Vertex> {
        self.images
            .get(lhs_key)
            .unwrap()
            .get_point_relation_vertex(rhs_key)
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

    /// `ImageManager`に登録されている画像の点群をOpenGLに登録する
    fn build_points_vertex(mut self) -> Self {
        self.images
            .iter_mut()
            .for_each(|(_, image)| image.build_points_vertex());
        self.is_build = true;
        self
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

    fn build_point_relation(mut self) -> Self {
        let keys: Vec<String> = self.get_image_keys().map(|val| val.to_string()).collect();
        for target_key in &keys {
            for (img_key, img) in &mut self.images {
                if img_key != target_key {
                    img.build_point_relation(target_key);
                }
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_manager() {
        let manager = ImageManager::new();
        assert!(manager.images.is_empty());
        assert!(!manager.is_build);
    }
}
