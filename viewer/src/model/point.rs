use std::collections::HashMap;

use cgmath::Point3;

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
    /// Retrun a point object.
    /// Arguments `x`, `y` and `z` are treated as point on the normalized coordinate system
    /// in which value range is from -1.0 to 1.0 with image center as (0, 0).
    /// Argument `r`, `g` and `b` are pixel values range from 0.0 to 1.0.
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
        (self.x() - x).abs() < 1e-5 && (self.y() - y).abs() < 1e-5
    }

    pub fn get_relation(&self, target_image_id: &str) -> Option<&Point3<f32>> {
        self.relations.get(target_image_id)
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

    pub fn r(&self) -> f32 {
        self.color.r
    }

    pub fn g(&self) -> f32 {
        self.color.g
    }

    pub fn b(&self) -> f32 {
        self.color.b
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_point() {
        let mut pt = Point::new(1.0, 0.5, -1.0, 0.0, 1.0, 0.8);
        assert!((pt.x() - 1.0).abs() < 1e-5);
        assert!((pt.y() - 0.5).abs() < 1e-5);
        assert!((pt.z() + 1.0).abs() < 1e-5);
        assert!((pt.r() - 0.0).abs() < 1e-5);
        assert!((pt.g() - 1.0).abs() < 1e-5);
        assert!((pt.b() - 0.8).abs() < 1e-5);

        assert!(pt.is_equal_to(1.0, 0.5));
        assert!(!pt.is_equal_to(1.0, 0.55));

        let key = "sample";
        pt.add_relation(key, 0.5, -0.7);
        let rel = pt.get_relation(key);
        assert!(rel.is_some());
        let rel = rel.unwrap();
        assert!((rel.x - 0.5).abs() < 1e-5);
        assert!((rel.y + 0.7).abs() < 1e-5);
    }
}
