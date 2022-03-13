use std::cell::Cell;

use cgmath::One;

use crate::{shader::UniformVariable, Mat4};

use super::{drawables::Drawable, Model};

pub struct ViewerModel {
    // view, projection matrixはすべてのobjectで共通。model matrixのみobject固有の値
    view_mat: UniformVariable<Mat4>,         // view matrix
    projection_mat: UniformVariable<Mat4>,   // projection matrix
    drawables: Cell<Vec<Box<dyn Drawable>>>, // drawable objects
}

impl ViewerModel {
    pub fn new() -> Box<Self> {
        Box::new(ViewerModel {
            view_mat: UniformVariable::<Mat4>::new("uView", Mat4::one()),
            projection_mat: UniformVariable::<Mat4>::new("uProjection", Mat4::one()),
            drawables: Cell::new(Vec::new()),
        })
    }

    pub fn add_drawables(&mut self) {}
}

impl Model for ViewerModel {
    fn get_view_mat(&self) -> UniformVariable<Mat4> {
        self.view_mat.clone()
    }

    fn get_projection_mat(&self) -> UniformVariable<Mat4> {
        self.projection_mat.clone()
    }

    fn get_mut_drawables(&mut self) -> &mut Vec<Box<dyn Drawable>> {
        self.drawables.get_mut()
    }

    fn add_drawable(&mut self, drawable: Box<dyn Drawable>) {
        self.drawables.get_mut().push(drawable);
    }
}
