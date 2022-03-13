use sdl2::sys::SDL_GetMouseState;

use crate::Mat4;

/// mouse positionを取得する
/// `normalized` == trueの場合、normalized texture coordinate (-1.0 ~ 1.0)
pub fn get_mouse_pos() -> (i32, i32) {
    let mut x = 0;
    let mut y = 0;
    let _: u32 = unsafe { SDL_GetMouseState(&mut x, &mut y) };
    (x, y)
}

pub fn scale_matrix(model_mat: &Mat4, cx: f32, cy: f32, scale: f32) -> Mat4 {
    let mut dst = model_mat.clone();
    let mut x = (cx - model_mat[3][0]) / (model_mat[0][0] + 1e-9);
    let mut y = (cy - model_mat[3][1]) / (model_mat[1][1] + 1e-9);
    if x.abs() > 1.0 || y.abs() > 1.0 {
        x = 0.0;
        y = 0.0;
    }
    let dx = (1.0 - scale) * x;
    let dy = (1.0 - scale) * y;
    dst[3][0] += dst[0][0] * dx;
    dst[3][1] += dst[1][1] * dy;
    dst[0][0] *= scale;
    dst[1][1] *= scale;
    dst
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::One;

    #[test]
    fn test_scale_matrix() {
        let mat = Mat4::one();
        let cx = 10.0;
        let cy = -10.0;
        let scale = 1.5;

        let dst = scale_matrix(&mat, cx, cy, scale);
        assert!((dst[0][0] - 1.5).abs() < 1e-5, "dst[0][1] = {}", dst[0][0]);
        assert!((dst[1][1] - 1.5).abs() < 1e-5, "dst[1][1] = {}", dst[1][1]);
        assert!((dst[3][0] - 0.0).abs() < 1e-5, "dst[3][0] = {}", dst[3][0]);
        assert!((dst[3][1] - 0.0).abs() < 1e-5, "dst[3][1] = {}", dst[3][1]);

        let dst = scale_matrix(&dst, 0.75, -0.3, 2.0);
        assert!((dst[0][0] - 3.0).abs() < 1e-5, "dst[0][1] = {}", dst[0][0]);
        assert!((dst[1][1] - 3.0).abs() < 1e-5, "dst[1][1] = {}", dst[1][1]);
        assert!((dst[3][0] + 0.75).abs() < 1e-5, "dst[3][0] = {}", dst[3][0]);
        assert!((dst[3][1] - 0.30).abs() < 1e-5, "dst[3][1] = {}", dst[3][1]);
    }
}
