use std::{collections::HashMap, os::raw::c_void};
use std::{mem, path::Path};

use anyhow::Result;
use cgmath::Point3;
use gl::types::{GLfloat, GLsizei, GLsizeiptr};
use image::{DynamicImage, GenericImageView};
use log::warn;

use crate::vertex::Vertex;

/// 画像の描画に必要な情報、画像上の点の情報を保持するstruct.
/// `points`に保持される点は正規化座標系上の点である。
/// (画像の左下を(-1.0, -1.0)、右上を(1.0, 1.0)で中心を(0, 0)とする座標系)
pub struct Image {
    image_texture_id: u32,
    width: u32,
    height: u32,
    points: Vec<Point>,
    points_vertex: Option<Vertex>,
    point_relation_vertex: HashMap<String, Vertex>,
}

impl Image {
    pub fn new(image_texture_id: u32, image_width: u32, image_height: u32) -> Image {
        Image {
            image_texture_id,
            width: image_width,
            height: image_height,
            points: Vec::new(),
            points_vertex: Option::None,
            point_relation_vertex: HashMap::new(),
        }
    }

    /// 画像に点を追加する
    pub fn add_point(mut self, point: Point) -> Image {
        self.points.insert(self.points.len(), point);
        self
    }

    /// 画像に他の画像の点との関係(`relation`)を追加する
    /// 引数の`x`, `y`, `other_x`, `other_y`は正規化座標系上の点。
    /// (画像の左下を(-1.0, -1.0)、右上を(1.0, 1.0)で中心を(0, 0)とする座標系)
    pub fn add_point_relation(
        mut self,
        x: f32,
        y: f32,
        other_key: &str,
        other_x: f32,
        other_y: f32,
    ) -> Image {
        match self.search_point(x, y) {
            Some(pt) => {
                pt.add_relation(other_key, other_x, other_y);
            }
            None => {
                warn!(
                    "No point is found in image id = `{}` at (x, y) = ({}, {}). Skip adding relation.",
                    self.image_texture_id, x, y
                );
            }
        };
        self
    }

    /// 指定した座標の`Point` objectを取得する。存在しない場合はNoneを返す
    /// 引数の`x`, `y`は正規化座標系上の点。
    /// (画像の左下を(-1.0, -1.0)、右上を(1.0, 1.0)で中心を(0, 0)とする座標系)
    fn search_point(&mut self, x: f32, y: f32) -> Option<&mut Point> {
        for pt in &mut self.points {
            if pt.is_equal_to(x, y) {
                return Some(pt);
            }
        }
        None
    }

    /// 画像(`Image`)に登録されている点群をOpenGlに登録(`Vertex::new`でvao, vboを作成)する
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

    pub fn build_point_relation(&mut self, key: &str) {
        let buf_array: Vec<f32> = self
            .points
            .iter()
            .map(|pt| match pt.relations.get(key) {
                Some(rel) => vec![pt.x(), pt.y(), pt.z(), 0.0f32, rel.x, rel.y, rel.z, 1.0f32],
                None => vec![],
            })
            .flatten()
            .collect();
        let block_size: usize = 4;
        self.point_relation_vertex.insert(
            key.to_string(),
            Vertex::new(
                (buf_array.len() as usize * mem::size_of::<GLfloat>()) as GLsizeiptr,
                buf_array.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
                vec![gl::FLOAT, gl::FLOAT],
                vec![3, 1],
                (block_size * mem::size_of::<GLfloat>()) as GLsizei,
                (buf_array.len() / block_size) as i32,
            ),
        );
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
pub struct Point {
    loc: Point3<f32>,
    color: Color,
    relations: HashMap<String, Point3<f32>>,
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) -> Point {
        Point {
            loc: Point3::<f32> { x, y, z },
            color: Color { r, g, b },
            relations: HashMap::new(),
        }
    }

    pub fn add_relation(&mut self, key: &str, x: f32, y: f32) {
        let pt = Point3::new(x, y, 1.0);
        self.relations.insert(key.to_string(), pt);
    }

    pub fn is_equal_to(&self, x: f32, y: f32) -> bool {
        (self.x() - x) < 1e-5 && (self.y() - y) < 1e-5
    }

    pub fn x(&self) -> f32 {
        self.loc.x
    }

    pub fn y(&self) -> f32 {
        self.loc.y
    }

    pub fn z(&self) -> f32 {
        self.loc.z
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        (other.x() - self.x()) < 1e-5 && (other.y() - self.y()) < 1e-5
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

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

    pub fn build(mut self) -> Self {
        self.build_points_vertex().build_point_relation()
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
        self.images
            .insert(id, Image::new(texture, image.width(), image.height()));
    }

    /// `ImageManager`に登録済みの画像のkeyの一覧を取得する
    pub fn get_image_keys(&self) -> std::collections::hash_map::Keys<String, Image> {
        self.images.keys()
    }

    /// `key`で指定した画像のtexture id(OpenGLの`gl::BindTexture`で指定するid)を取得する
    pub fn get_texture_id(&self, key: &str) -> u32 {
        match self.images.get(key) {
            Some(image) => image.image_texture_id,
            None => 0,
        }
    }

    /// `key`で指定した画像のtexture size(画像サイズ)を取得する
    pub fn get_texture_image_size(&self, key: &str) -> (u32, u32) {
        match self.images.get(key) {
            Some(image) => (image.width, image.height),
            None => (1u32, 1u32),
        }
    }

    /// `key`で指定した画像の頂点情報(`Vertex`)を取得する
    pub fn get_points_vertex(&self, key: &str) -> &Option<Vertex> {
        if !self.is_build {
            warn!("`ImageManager` has not been built. `build_points_vertex` should be called.")
        }
        &self.images.get(key).unwrap().points_vertex
    }

    /// `lhs_key`, `rhs_key`で指定した画像間のpoint relationのVertexを取得する
    /// `lhs_key`, `rhs_key`の順番を逆にすると正しく表示されなくなるので注意する。
    pub fn get_point_relation(&self, lhs_key: &str, rhs_key: &str) -> Option<&Vertex> {
        self.images
            .get(lhs_key)
            .unwrap()
            .point_relation_vertex
            .get(rhs_key)
    }

    /// add point (`x`, `y`, `z`) to image of `image_id`.
    /// Arguments `x`, `y` and `z` are treated as point on the normalized coordinate system
    /// in which value range is from -1.0 to 1.0 with image center as (0, 0).
    pub fn add_point(&mut self, image_id: &str, x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) {
        let point = Point::new(x, y, z, r, g, b);
        let image = self.images.remove(image_id).unwrap();
        let image = image.add_point(point);
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
        let image = self
            .images
            .remove(lhs_key)
            .unwrap()
            .add_point_relation(lx, ly, rhs_key, rx, ry);
        self.images.insert(lhs_key.to_string(), image);
        let image = self
            .images
            .remove(rhs_key)
            .unwrap()
            .add_point_relation(rx, ry, lhs_key, lx, ly);
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
