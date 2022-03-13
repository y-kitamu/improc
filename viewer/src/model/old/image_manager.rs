use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use image::DynamicImage;
use imgui::im_str;
use log::warn;

use crate::{
    define_gl_primitive,
    shader::{image_shader::ImageShader, relation_line_shader::RelationLineShader},
};

use super::{create_simple_vertex, image::Image, GLPrimitive};

const DEFAULT_POIRNT_RELATION_SHADER: &str = "relation_line";

/// Textureに登録した画像を管理する。
/// 画像は左下が原点(pointerの開始地点)になるように、適当にflipする
/// 外部から`model` moduleにaccessするためのinterface. (`ImageManager`以外はprivateにする)
pub struct ImageManager {
    images: HashMap<String, Image>,
    vao: Option<u32>,
    vbo: Option<u32>,
    vertex_num: i32,
    point_relation_shader: RelationLineShader,
    is_show_point_relation: bool,
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
            point_relation_shader: RelationLineShader::new(DEFAULT_POIRNT_RELATION_SHADER),
            is_show_point_relation: true,
        }
    }

    pub fn build(mut self) -> Self {
        self.images.iter_mut().for_each(|(_key, val)| {
            val.build();
        });
        self
    }

    pub fn draw(&mut self, img_key: &str, screen_width: u32, screen_height: u32) {
        self.images.get_mut(img_key).unwrap().draw_objects(
            self.vao.unwrap(),
            self.vertex_num,
            screen_width,
            screen_height,
        );
    }

    pub fn draw_point_relations(&self, lhs_key: &str, rhs_key: &str) {
        if self.is_show_point_relation {
            let lhs_img = self.images.get(lhs_key).unwrap();
            let rhs_img = self.images.get(rhs_key).unwrap();
            self.point_relation_shader
                .set_uniform_variables(lhs_img.shader(), rhs_img.shader());
            lhs_img.draw_point_relations(rhs_key);
        }
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

    pub fn get_current_image_shader(&self, key: &str) -> &ImageShader {
        self.images.get(key).unwrap().shader()
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

    pub fn add_arrow(&mut self, image_id: &str, x: f32, y: f32, direction: f32, length: f32) {
        let image = self.images.remove(image_id).unwrap();
        let image = image.add_arrow(x, y, direction, length);
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

    pub fn on_mouse_wheel(&mut self, key: &str, x: f32, y: f32, scale: f32) {
        if let Some(img) = self.images.get_mut(key) {
            img.on_mouse_wheel(x, y, scale);
        }
    }

    pub fn on_mouse_button_down(&mut self, key: &str, fx: f32, fy: f32) {
        if let Some(img) = self.images.get_mut(key) {
            img.on_mouse_button_down(fx, fy);
        }
    }

    pub fn on_mouse_button_up(&mut self, key: &str) {
        if let Some(img) = self.images.get_mut(key) {
            img.on_mouse_button_up();
        }
    }

    pub fn on_mouse_motion_event(&mut self, key: &str, dx: f32, dy: f32) {
        if let Some(img) = self.images.get_mut(key) {
            img.on_mouse_motion_event(dx, dy);
        }
    }

    pub fn draw_points_imgui(&mut self, ui: &imgui::Ui, image_key: &str) {
        ui.separator();
        ui.text(im_str!("Point parameter"));

        let mut is_hide = match self.images.get(image_key) {
            Some(img) => !img.points.is_show,
            None => false,
        };
        if ui.checkbox(im_str!("Hide points"), &mut is_hide) {
            self.images.iter_mut().for_each(|(_, val)| {
                val.points.is_show = !is_hide;
            });
        }

        let mut pt_size = match self.images.get(image_key) {
            Some(img) => img.get_point_size(),
            None => 1.0,
        };

        if imgui::Slider::new(im_str!("Point size"))
            .range(1.0..=100.0)
            .build(&ui, &mut pt_size)
        {
            self.images.iter_mut().for_each(|(_, val)| {
                val.set_point_size(pt_size);
            });
        }
    }

    pub fn draw_arrows_imgui(&mut self, ui: &imgui::Ui, image_key: &str) {
        let (mut r, mut g, mut b, mut a, mut scale, mut is_hide) = match self.images.get(image_key)
        {
            Some(img) => {
                let color = &img.arrows.shader.color.value;
                (
                    color.x,
                    color.y,
                    color.z,
                    color.w,
                    img.arrows.shader.scale.value,
                    !img.arrows.is_show,
                )
            }
            None => (1.0, 1.0, 1.0, 1.0, 1.0, false),
        };

        if imgui::Slider::new(im_str!("Line Scale"))
            .range(0.1..=100.0)
            .build(&ui, &mut scale)
        {
            self.images.iter_mut().for_each(|(_, val)| {
                val.arrows.shader.scale.value = scale;
            });
        }

        let mut flag = false;
        flag |= imgui::Slider::new(im_str!("Arrow Color (R)"))
            .range(0.0..=1.0)
            .build(&ui, &mut r);
        flag |= imgui::Slider::new(im_str!("Arrow Color (G)"))
            .range(0.0..=1.0)
            .build(&ui, &mut g);
        flag |= imgui::Slider::new(im_str!("Arrow Color (B)"))
            .range(0.0..=1.0)
            .build(&ui, &mut b);
        flag |= imgui::Slider::new(im_str!("Arrow Alpha"))
            .range(0.0..=1.0)
            .build(&ui, &mut a);
        if flag {
            self.images.iter_mut().for_each(|(_, val)| {
                val.arrows.set_color(r, g, b, a);
            });
        }
    }

    pub fn draw_lines_imgui(&mut self, ui: &imgui::Ui) {
        ui.separator();
        ui.text(im_str!("Line parameter"));

        let mut is_hide = !self.is_show_point_relation;
        if ui.checkbox(im_str!("Hide lines"), &mut is_hide) {
            self.is_show_point_relation = !is_hide;
        }

        imgui::Slider::new(im_str!("Line Color (R)"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.point_relation_shader.color.value.x);
        imgui::Slider::new(im_str!("Line Color (G)"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.point_relation_shader.color.value.y);
        imgui::Slider::new(im_str!("Line Color (B)"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.point_relation_shader.color.value.z);
        imgui::Slider::new(im_str!("Line Alpha"))
            .range(0.0..=1.0)
            .build(&ui, &mut self.point_relation_shader.color.value.w);
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use cgmath::{One, Vector4};

    use crate::{
        shader::{arrow_line_shader::ArrowLineShader, point_shader::PointShader, UniformVariable},
        Matrix4,
    };

    use super::super::{arrow::Arrows, point::Points};

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
            is_show: true,
        }
    }

    fn get_arrows() -> Arrows {
        Arrows {
            vao: Some(2),
            vbo: Some(2),
            vertex_num: 20,
            arrows: Vec::new(),
            shader: ArrowLineShader {
                id: 0,
                color: UniformVariable {
                    name: CString::new("uColor").unwrap(),
                    value: Vector4::<f32>::new(1.0, 0.0, 0.0, 1.0),
                },
                scale: UniformVariable::new("uScale", 1.0),
            },
            is_show: true,
        }
    }

    fn get_image(key: &str) -> Image {
        Image {
            key: key.to_string(),
            texture_id: 1,
            image_shader: get_image_shader(),
            width: 1920,
            height: 1080,
            points: get_points(),
            arrows: get_arrows(),
            point_relations: HashMap::new(),
        }
    }

    #[test]
    fn test_image_manager() {
        let mut manager = ImageManager {
            images: HashMap::new(),
            vao: None,
            vbo: None,
            vertex_num: 0,
            point_relation_shader: RelationLineShader {
                id: 0,
                color: UniformVariable::new("uColor", Vector4::<f32>::new(1.0, 0.0, 0.0, 1.0)),
            },
            is_show_point_relation: true,
        };

        let key: &str = "default";

        assert_eq!(manager.get_texture_id(key), 0);

        assert!(manager.images.is_empty());
        let image = get_image("default");
        manager.images.insert(image.key().to_string(), image);
        let keys: Vec<&String> = manager.get_image_keys().collect();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].clone(), key.to_string());

        assert_eq!(manager.get_texture_id(key), 1);

        let (w, h) = manager.get_texture_image_size(key);
        assert_eq!(w, 1920);
        assert_eq!(h, 1080);

        let imshader = manager.get_current_image_shader(key);
        assert_eq!(imshader.id, 0);

        manager.add_point(key, 960.0, 540.0, -0.5, 0.3, 0.2, 0.5);
        assert_eq!(manager.images[key].points.points.len(), 1);
        assert!(manager.images[key].points.points[0].loc.x < 1e-5);
        assert!(manager.images[key].points.points[0].loc.y < 1e-5);

        let other_key = "other";
        manager
            .images
            .insert(other_key.to_string(), get_image(other_key));
        manager.add_point_relation(key, 960.0, 540.0, other_key, 0.0, 0.0);
        assert!(manager.images[key].point_relations[other_key].lines[0].x < 1e-5);
        assert!(manager.images[key].point_relations[other_key].lines[0].y < 1e-5);
        assert!(
            (manager.images[key].point_relations[other_key].lines[0].other_x + 1.0).abs() < 1e-5
        );
        assert!(
            (manager.images[key].point_relations[other_key].lines[0].other_y - 1.0).abs() < 1e-5,
            "other_y = {}",
            manager.images[key].point_relations[other_key].lines[0].other_y
        );

        manager.on_mouse_wheel(key, 0.2, -0.2, 2.0);
        assert!((manager.images[key].image_shader.model_mat.value[0][0] - 2.0).abs() < 1e-5);
        assert!((manager.images[key].image_shader.model_mat.value[1][1] - 2.0).abs() < 1e-5);
        assert!((manager.images[key].image_shader.model_mat.value[3][0] + 0.2).abs() < 1e-5);
        assert!((manager.images[key].image_shader.model_mat.value[3][1] - 0.2).abs() < 1e-5);

        manager.on_mouse_motion_event(key, 0.0, 0.0);
        assert!((manager.images[key].image_shader.model_mat.value[3][0] + 0.2).abs() < 1e-5);
        assert!((manager.images[key].image_shader.model_mat.value[3][1] - 0.2).abs() < 1e-5);

        manager.on_mouse_button_up(key);
        assert!(!manager.images[key].image_shader.is_dragging);
    }
}
