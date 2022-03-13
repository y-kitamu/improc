use imgui::im_str;
use sdl2::{event::Event, mouse::MouseWheelDirection};

use crate::{model::image_manager::ImageManager, utility::get_mouse_pos};

use super::PresenterMode;

pub struct DualImagePresenter {
    current_image_keys: (String, String),
}

impl DualImagePresenter {
    const MODE_NAME: &'static str = "dual";

    pub fn new() -> Self {
        let current_image_keys = ("".to_string(), "".to_string());
        DualImagePresenter { current_image_keys }
    }

    // TODO: refactor. default.rsの`draw`とほぼおなじコード
    fn draw_half(
        &self,
        image_key: &str,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
        mut image_manager: ImageManager,
    ) -> ImageManager {
        unsafe {
            gl::Viewport(left as i32, top as i32, width as i32, height as i32);
        }
        image_manager.draw(image_key, width, height);
        image_manager
    }

    fn get_current_image_key(&self, fbo_width: u32) -> &str {
        let (x, _y) = get_mouse_pos();
        if (x as u32) < fbo_width / 2 {
            &self.current_image_keys.0
        } else {
            &self.current_image_keys.1
        }
    }
}

impl PresenterMode for DualImagePresenter {
    fn get_mode_name(&self) -> &str {
        Self::MODE_NAME
    }

    fn process_event(
        &self,
        event: &Event,
        fbo_width: u32,
        fbo_height: u32,
        mut image_manager: ImageManager,
    ) -> (ImageManager, bool) {
        let key = self.get_current_image_key(fbo_width);
        let processed = match event {
            Event::MouseWheel { y, direction, .. } => {
                let (mx, my) = get_mouse_pos();
                let half = fbo_width as f32 / 2.0;
                let cy = (fbo_height as f32 - my as f32) / fbo_height as f32 * 2.0 - 1.0;
                let cx = if (mx as u32) < fbo_width / 2 {
                    (mx as f32) / half * 2.0 - 1.0
                } else {
                    (mx as f32 - half) / half * 2.0 - 1.0
                };
                let mut scale = 1.0f32 + *y as f32 / 10.0f32;
                if *direction == MouseWheelDirection::Flipped {
                    scale = 1.0f32 / scale;
                }
                image_manager.on_mouse_wheel(&key, cx, cy, scale);
                true
            }
            Event::MouseButtonDown { x, y, .. } => {
                // 左上(0, 0), 右下(width, height)の座標系を
                // 中心(0, 0), 左上(-1.0, 1.0), 右下(1.0, -1.0)の座標系に変換する
                let fx = if (*x as u32) < fbo_width / 2 {
                    *x as f32 / fbo_width as f32 * 4.0f32 - 1.0f32
                } else {
                    (*x as f32 / fbo_width as f32 - 0.5) * 4.0f32 - 1.0f32
                };
                let fy = 1.0f32 - *y as f32 / fbo_height as f32 * 2.0f32;
                image_manager.on_mouse_button_down(key, fx, fy);
                true
            }
            Event::MouseButtonUp { .. } => {
                image_manager.on_mouse_button_up(key);
                true
            }
            Event::MouseMotion { xrel, yrel, .. } => {
                let dx = *xrel as f32 / fbo_width as f32 * 4.0f32;
                let dy = -*yrel as f32 / fbo_height as f32 * 2.0f32;
                image_manager.on_mouse_motion_event(key, dx, dy);
                true
            }
            _ => false,
        };
        (image_manager, processed)
    }

    fn draw(&mut self, width: u32, height: u32, mut image_manager: ImageManager) -> ImageManager {
        if self.current_image_keys.0.len() == 0 || self.current_image_keys.1.len() == 0 {
            return image_manager;
        }

        let lhs_key = self.current_image_keys.0.clone();
        image_manager = self.draw_half(&lhs_key, 0, 0, width / 2, height, image_manager);
        let rhs_key = self.current_image_keys.1.clone();
        image_manager = self.draw_half(&rhs_key, width / 2, 0, width / 2, height, image_manager);

        // draw line
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
        image_manager.draw_point_relations(&lhs_key, &rhs_key);
        image_manager
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui, mut image_manager: ImageManager) -> ImageManager {
        imgui::Window::new(im_str!("Parameters"))
            .size([200.0, 250.0], imgui::Condition::FirstUseEver)
            .position([700.0, 10.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Image parameter"));
                ui.separator();

                for key in image_manager.get_image_keys() {
                    let mut flag = self.current_image_keys.0 == *key;
                    if ui.radio_button(&im_str!("{}0", key), &mut flag, true) {
                        self.current_image_keys.0 = key.clone();
                    }
                    ui.same_line(100.0);
                    let mut flag = self.current_image_keys.1 == *key;
                    if ui.radio_button(&im_str!("{}1", key), &mut flag, true) {
                        self.current_image_keys.1 = key.clone();
                    }
                }

                image_manager.draw_points_imgui(ui, &self.current_image_keys.0);
                image_manager.draw_arrows_imgui(ui, &self.current_image_keys.0);
                image_manager.draw_lines_imgui(ui);
                ui.separator();
            });
        image_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual() {
        let mode = DualImagePresenter::new();
        assert_eq!(mode.get_mode_name(), "dual");
    }
}
