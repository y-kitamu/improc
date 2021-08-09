use std::{collections::HashMap, os::raw::c_void};
use std::{mem, path::Path};

use anyhow::Result;
use cgmath::Point3;
use gl::types::{GLfloat, GLsizei, GLsizeiptr};
use image::{DynamicImage, GenericImageView};
use log::warn;

use crate::vertex::Vertex;

struct Image {
    image_texture_id: u32,
    width: u32,
    height: u32,
    points: Vec<Point>,
    points_vertex: Option<Vertex>,
}

impl Image {
    pub fn new(image_texture_id: u32, image_width: u32, image_height: u32) -> Image {
        Image {
            image_texture_id,
            width: image_width,
            height: image_height,
            points: Vec::new(),
            points_vertex: Option::None,
        }
    }

    pub fn add_point(mut self, point: Point) -> Image {
        self.points.insert(self.points.len(), point);
        self
    }

    pub fn build_points_vertex(&mut self) {
        if self.points.len() > 0 && self.points_vertex.is_none() {
            let buf_array = self
                .points
                .iter()
                .map(|p| vec![p.loc.x, p.loc.y, p.loc.z, p.color.r, p.color.g, p.color.b])
                .flatten()
                .collect::<Vec<f32>>();
            self.points_vertex = Some(Vertex::new(
                (buf_array.len() as usize * mem::size_of::<GLfloat>()) as GLsizeiptr,
                buf_array.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
                vec![gl::FLOAT, gl::FLOAT],
                vec![3, 3],
                (6 * mem::size_of::<GLfloat>()) as GLsizei,
                (buf_array.len() / 6) as i32,
            ));
        }
    }
}

#[derive(Clone)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

/// 点情報を保持する
/// locには画像の中心を原点(0, 0)、右上を(1, 1)とした座標系での値を保持する。
struct Point {
    loc: Point3<f32>,
    color: Color,
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) -> Point {
        Point {
            loc: Point3::<f32> { x, y, z },
            color: Color { r, g, b },
        }
    }

    pub fn add_relation(self) -> Point {
        self
    }
}

struct PointRelation {
    locs: Vec<Point3<f32>>,
    ids: Vec<String>,
}

/// Textureに登録した画像を管理する。
/// 画像は左下が原点(pointerの開始地点)になるように、適当にflipする
pub struct ImageManager {
    images: HashMap<String, Image>,
    current_image_key: String,
}

impl ImageManager {
    pub fn new() -> ImageManager {
        let image_manager = ImageManager {
            images: HashMap::new(),
            current_image_key: "".to_string(),
        };
        image_manager
    }

    pub fn load_image(&mut self, path: &Path, vflip: bool, id: &str) -> Result<()> {
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
        let id = id.to_string();
        if self.images.contains_key(&id) {
            warn!(
                "Image key {} already exist in `images`. Skip add image.",
                id
            );
            return;
        }
        if self.current_image_key.len() == 0 {
            self.current_image_key = id.clone();
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
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

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
        self.images
            .insert(id, Image::new(texture, image.width(), image.height()));
    }

    pub fn get_current_texture_id(&self) -> u32 {
        match self.images.get(&self.current_image_key) {
            Some(image) => image.image_texture_id,
            None => 0,
        }
    }

    pub fn get_current_texture_image_size(&self) -> (u32, u32) {
        match self.images.get(&self.current_image_key) {
            Some(image) => (image.width, image.height),
            None => (1u32, 1u32),
        }
    }

    pub fn get_current_points_vertex(&self) -> &Option<Vertex> {
        &self
            .images
            .get(&self.current_image_key)
            .unwrap()
            .points_vertex
    }

    /// add point (`x`, `y`, `z`) to image of `image_id`.
    /// The coordinate system is normalized from -1.0 to 1.0 with image center as (0, 0).
    pub fn add_point(&mut self, image_id: &str, x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) {
        let point = Point::new(x, y, z, r, g, b);
        let image = self.images.remove(image_id).unwrap();
        let image = image.add_point(point);
        self.images.insert(image_id.to_string(), image);
    }

    pub fn add_point_relation(&mut self) {}

    pub fn build_points_vertex(mut self) -> Self {
        self.images
            .iter_mut()
            .for_each(|(_, image)| image.build_points_vertex());
        self
    }
}
