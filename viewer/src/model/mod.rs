use sdl2::mouse::MouseWheelDirection;

use crate::{shader::UniformVariable, Mat4};

use self::drawables::Drawable;

pub mod drawables;
pub mod viewer_model;

macro_rules! callback_method {
    ($func:ident) => {
        fn $func(&mut self) {
            for drawable in self.get_mut_drawables() {
                drawable.get_mut_shader().$func();
            }
        }
    };
    ($func:ident, $( $arg:ident:$type:ty ), *) => {
        fn $func(&mut self, $( $arg: $type ),*) {
            for drawable in self.get_mut_drawables() {
                drawable.get_mut_shader().$func($( $arg ),*);
            }
        }
    };
}

pub trait Model {
    fn get_view_mat(&self) -> UniformVariable<Mat4>;
    fn get_projection_mat(&self) -> UniformVariable<Mat4>;
    fn get_mut_drawables(&mut self) -> &mut Vec<Box<dyn Drawable>>;
    fn add_drawable(&mut self, drawable: Box<dyn Drawable>);

    fn build(&mut self) {
        for obj in self.get_mut_drawables() {
            obj.build();
        }
    }

    /// Draw `Drawable`s.
    fn draw(&mut self) {
        let view_mat = self.get_view_mat();
        let proj_mat = self.get_projection_mat();
        for obj in self.get_mut_drawables() {
            if obj.is_draw() {
                obj.draw(&view_mat, &proj_mat);
            }
        }
    }

    callback_method!(on_mouse_button_down, fx: f32, fy: f32);
    callback_method!(
        on_mouse_wheel,
        cx: f32,
        cy: f32,
        y: &i32,
        direction: &MouseWheelDirection
    );
    callback_method!(on_mouse_motion_event, xrefl: f32, yrel: f32);
    callback_method!(on_mouse_button_up);
}
