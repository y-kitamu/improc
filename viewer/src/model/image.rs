use std::{collections::HashMap, ffi::c_void, mem};

use gl::types::{GLfloat, GLsizei, GLsizeiptr};
use log::warn;

use crate::vertex::Vertex;

use super::point::Point;

/// 画像の描画に必要な情報、画像上の点の情報を保持するstruct.
/// `points`に保持される点は正規化座標系上の点である。
/// (画像の左下を(-1.0, -1.0)、右上を(1.0, 1.0)で中心を(0, 0)とする座標系)
/// ただし、functionの引数ではimage coordinate(画像のpixel単位の座標)を使用する。
/// `points_vertex`は点をOpenGL描画するためのvao, vboを保持する
/// `point_relation_vertex`は画像間の直線をOptnGLで描画するためのvao, vboを保持する。
pub struct Image {
    key: String,
    image_texture_id: u32,
    width: u32,
    height: u32,
    points: Vec<Point>,
    points_vertex: Option<Vertex>,
    point_relation_vertex: HashMap<String, Vertex>,
}

impl Image {
    pub fn new(key: &str, image_texture_id: u32, image_width: u32, image_height: u32) -> Image {
        Image {
            key: key.to_string(),
            image_texture_id,
            width: image_width,
            height: image_height,
            points: Vec::new(),
            points_vertex: Option::None,
            point_relation_vertex: HashMap::new(),
        }
    }

    pub fn image_key(&self) -> &str {
        &self.key
    }

    pub fn id(&self) -> u32 {
        self.image_texture_id
    }

    pub fn w(&self) -> u32 {
        self.width
    }

    pub fn h(&self) -> u32 {
        self.height
    }

    pub fn get_points_vertex(&self) -> &Option<Vertex> {
        &self.points_vertex
    }

    pub fn get_point_relation_vertex(&self, key: &str) -> Option<&Vertex> {
        self.point_relation_vertex.get(key)
    }

    pub fn convert_to_norm_coord(&self, x: f32, y: f32) -> (f32, f32) {
        let x = x / self.width as f32 * 2.0 - 1.0;
        let y = 1.0 - y / self.height as f32 * 2.0;
        (x, y)
    }

    /// 画像に点を追加する
    /// Argument `x` and `y` are treated as point on the image coordinate system.
    /// A value range of `z` is from -1.0 to 1.0.
    /// Argument `r`, `g` and `b` are pixel values range from 0.0 to 1.0.
    pub fn add_point(mut self, x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) -> Image {
        let (x, y) = self.convert_to_norm_coord(x, y);
        let point = Point::new(x, y, z, r, g, b);
        self.points.insert(self.points.len(), point);
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
        let (other_x, other_y) = other_image.convert_to_norm_coord(other_x, other_y);
        match self.search_point(x, y) {
            Some(pt) => {
                pt.add_relation(other_image.image_key(), other_x, other_y);
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
    /// Argument `x` and `y` are treated as point on the image coordinate system.
    /// 引数の`x`, `y`は正規化座標系上の点。
    fn search_point(&mut self, x: f32, y: f32) -> Option<&mut Point> {
        let (x, y) = self.convert_to_norm_coord(x, y);
        for pt in &mut self.points {
            if pt.is_equal_to(x, y) {
                return Some(pt);
            }
        }
        None
    }

    /// 画像(`Image`)に登録されている点群をOpenGLに登録(`Vertex::new`でvao, vboを作成)する
    pub fn build_points_vertex(&mut self) {
        if self.points.len() > 0 && self.points_vertex.is_none() {
            let buf_array = self
                .points
                .iter()
                .map(|p| vec![p.x(), p.y(), p.z(), p.r(), p.g(), p.b()])
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
            .map(|pt| match pt.get_relation(key) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image() {
        let image = Image::new("default", 0u32, 1280u32, 1080u32);
        assert_eq!(image.id(), 0u32);
        assert_eq!(image.w(), 1280u32);
        assert_eq!(image.h(), 1080u32);

        let mut image = image.add_point(1280.0f32, 1080.0f32, 0.1, 1.0, 1.0, 1.0);
        let pt = image.search_point(1280.0, 1080.0);
        assert!(pt.is_some());
        let pt = pt.unwrap();
        assert!((pt.x() - 1.0).abs() < 1e-5, "pt.x() = {}", pt.x());
        assert!((pt.y() + 1.0).abs() < 1e-5, "pt.y() = {}", pt.y());

        let other_key = "other";
        let other_img = Image::new(other_key, 1u32, 1080u32, 960u32);
        let image = image.add_point_relation(1280.0, 1080.0, &other_img, 540.0, 240.0);
        let image = image.add_point_relation(1200.0, 1080.0, &other_img, 540.0, 240.0);
        assert_eq!(image.points.len(), 1);
        let rel = image.points[0].get_relation(other_key);
        assert!(rel.is_some());
        let rel = rel.unwrap();
        assert!((rel.x - 0.0).abs() < 1e-5, "rel.x = {}", rel.x);
        assert!((rel.y - 0.5).abs() < 1e-5, "rel.y = {}", rel.y);

        let res = image.get_points_vertex();
        assert!(res.is_none());
        let res = image.get_point_relation_vertex("other");
        assert!(res.is_none());
    }
}
