use std::path::Path;
use std::{collections::HashMap, os::raw::c_void};

use anyhow::Result;
use cgmath::Point3;
use image::{DynamicImage, GenericImageView};
use log::warn;

struct Image {
    image_texture_id: u32,
    points: Vec<Point>,
}

impl Image {
    pub fn new(image_texture_id: u32) -> Image {
        Image {
            image_texture_id,
            points: Vec::new(),
        }
    }

    pub fn add_point(mut self, point: Point) -> Image {
        self.points.insert(self.points.len(), point);
        self
    }
}

#[derive(Clone)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

struct Point {
    loc: Point3<f32>,
    color: Color,
}

impl Point {
    pub fn new(loc: &Point3<f32>, color: &Color) -> Point {
        Point {
            loc: loc.clone(),
            color: color.clone(),
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

/// Hold images.
/// `image_map` holds
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
        self.images.insert(id, Image::new(texture));
    }

    pub fn get_current_texture_id(&self) -> u32 {
        let texture_id = match self.images.get(&self.current_image_key) {
            Some(image) => image.image_texture_id,
            None => 0,
        };
        texture_id
    }

    pub fn add_point(&mut self, point: &Point3<f32>, image_id: &str, color: &Color) {
        let point = Point::new(point, color);
        let image = self.images.remove(image_id).unwrap();
        let image = image.add_point(point);
        self.images.insert(image_id.to_string(), image);
    }

    pub fn add_point_relation(&mut self) {}
}
