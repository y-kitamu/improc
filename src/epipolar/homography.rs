//! Homography matrix
use nalgebra as na;

use crate::{linalg::matrix::pseudo_inverse, optimizer::ObservedData};

/// Struct for computing homography matrix from observed points in two images.
/// - `data` is observed points on the two images. [image0_pt0, image1_pt0, image0_pt1, ....].
/// - `scale` is scale factor for minimizing the impact of floating point error.
/// - `delta` is offset for an optimal point. It is used in geometric error minimization.
pub struct HomographyData<'a> {
    data: &'a [na::Point2<f64>],
    scale: f64,
    delta: Vec<na::Point2<f64>>,
}

impl<'a> HomographyData<'a> {
    fn get_t(&self, index: usize) -> na::DMatrix<f64> {
        let idx = index / self.num_equation();
        let offset = index % self.num_equation();

        let f = self.scale;
        let p0 = self.data[idx * 2];
        let p1 = self.data[idx * 2 + 1];
        let (x, y) = (p0[0], p0[1]);
        let (xh, yh) = (p1[0], p1[1]);

        match offset {
            0 => {
                #[rustfmt::skip]
                let t = na::DMatrix::from_row_slice(9, 4, &[
                    0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0,
                    -f,  0.0, 0.0, 0.0,
                    0.0, -f,  0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0,
                    yh,  0.0, 0.0, x,
                    0.0, yh,  0.0, y,
                    0.0, 0.0, 0.0, f,
                ]);
                t
            }
            1 => {
                #[rustfmt::skip]
                let t = na::DMatrix::from_row_slice(9, 4, &[
                    f,   0.0, 0.0, 0.0,
                    0.0, f,   0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0,
                    -xh, 0.0, -x,  0.0,
                    0.0, -xh, -y,  0.0,
                    0.0, 0.0, -f,  0.0,
                ]);
                t
            }
            2 => {
                #[rustfmt::skip]
                let t = na::DMatrix::from_row_slice(9, 4, &[
                    -yh, 0.0, 0.0, -x,
                    0.0, -yh, 0.0, -y,
                    0.0, 0.0, 0.0, -f,
                    xh,  0.0, x,   0.0,
                    0.0, xh,  y,   0.0,
                    0.0, 0.0, f,   0.0,
                    0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0,
                ]);
                t
            }
            _ => {
                panic!("Something wrong in `HomographyData`")
            }
        }
    }
}

impl<'a> ObservedData<'a> for HomographyData<'a> {
    fn new(data: &'a [na::Point2<f64>]) -> Self {
        HomographyData {
            data,
            scale: 1.0,
            delta: vec![na::Point2::new(0.0, 0.0); data.len()],
        }
    }

    fn len(&self) -> usize {
        self.data.len() / 2
    }

    fn vector(&self, data_index: usize) -> na::DVector<f64> {
        let idx = data_index / self.num_equation();
        let offset = data_index % self.num_equation();

        let p0 = self.data[idx * 2];
        let p1 = self.data[idx * 2 + 1];
        let f = self.scale;
        let (x, y) = (p0[0], p0[1]);
        let (xh, yh) = (p1[0], p1[1]);

        if offset == 0 {
            na::DVector::from_vec(vec![
                0.0,
                0.0,
                0.0,
                -f * x,
                -f * y,
                -f * f,
                x * yh,
                y * yh,
                f * yh,
            ])
        } else if offset == 1 {
            na::DVector::from_vec(vec![
                f * x,
                f * y,
                f * f,
                0.0,
                0.0,
                0.0,
                -x * xh,
                -y * xh,
                -f * xh,
            ])
        } else if offset == 2 {
            na::DVector::from_vec(vec![
                -x * yh,
                -y * yh,
                -f * yh,
                x * xh,
                y * xh,
                f * xh,
                0.0,
                0.0,
                0.0,
            ])
        } else {
            panic!("Something wrong in `HomographyData`.")
        }
    }

    fn matrix(&self, weight_vector: &[f64]) -> na::DMatrix<f64> {
        let weight_vector: Vec<f64> = if weight_vector.iter().all(|val| (val - 1.0).abs() < 1e-5) {
            let vec_size = self.vec_size();
            (0..weight_vector.len())
                .map(|idx| {
                    let val = idx % vec_size;
                    if val % 3 == val / 3 {
                        1.0
                    } else {
                        0.0
                    }
                })
                .collect()
        } else {
            weight_vector.to_vec()
        };
        let n_eqs = self.num_equation();
        let n_eqs_square = n_eqs * n_eqs;
        (0..self.len())
            .map(|idx| {
                (0..n_eqs)
                    .map(|i| {
                        let xi_i = self.vector(idx * n_eqs + i);
                        (0..n_eqs)
                            .map(|j| {
                                let xi_j = self.vector(idx * n_eqs + j);
                                let w = weight_vector[idx * n_eqs_square + i * n_eqs + j];
                                w * &xi_i * xi_j.transpose()
                            })
                            .sum::<na::DMatrix<f64>>()
                    })
                    .sum::<na::DMatrix<f64>>()
            })
            .sum::<na::DMatrix<f64>>()
            / self.len() as f64
    }

    fn variance(&self, data_index: usize) -> na::DMatrix<f64> {
        let n_eqs = self.num_equation();
        let n_eqs_square = n_eqs * n_eqs;
        let idx = data_index / n_eqs_square;
        let k = (data_index % n_eqs_square) / n_eqs;
        let l = (data_index % n_eqs_square) % n_eqs;
        let kt = self.get_t(idx * n_eqs + k);
        let lt = self.get_t(idx * n_eqs + l);
        kt * lt.transpose()
    }

    fn weights(&self, params: &na::DVector<f64>) -> Vec<f64> {
        let n_eqs = self.num_equation();
        let n_eqs_square = n_eqs * n_eqs;
        if params.iter().all(|val| val.abs() < 1e-5) {
            return vec![1.0; self.len() * n_eqs_square];
        }

        (0..self.len())
            .map(|idx| {
                let vars_mat = na::DMatrix::from_fn(n_eqs, n_eqs, |r, c| {
                    let var_mat = self.variance(idx * n_eqs_square + r * n_eqs + c);
                    params.dot(&(var_mat * params))
                });
                let inv = pseudo_inverse(&vars_mat).unwrap();
                (0..n_eqs_square)
                    .map(|idx| inv[(idx / n_eqs, idx % n_eqs)])
                    .collect::<Vec<f64>>()
            })
            .flatten()
            .collect()
    }

    fn update_delta(&mut self, params: &na::DVector<f64>) -> f64 {
        let weights = self.weights(params);
        let n_eqs = self.num_equation();
        let n_eqs_square = n_eqs * n_eqs;

        (0..self.len())
            .map(|idx| {
                let delta = (0..n_eqs)
                    .map(|i| {
                        let xi = self.vector(n_eqs * idx + i);
                        let dot = xi.dot(params);
                        (0..n_eqs)
                            .map(|j| {
                                let w = weights[n_eqs_square * idx + n_eqs * i + j];
                                let t = self.get_t(n_eqs * idx + j);
                                w * dot * t.transpose() * params
                            })
                            .sum::<na::DVector<f64>>()
                    })
                    .sum::<na::DVector<f64>>();
                self.delta[idx * 2][0] -= delta[0];
                self.delta[idx * 2][1] -= delta[1];
                self.delta[idx * 2 + 1][0] -= delta[2];
                self.delta[idx * 2 + 1][1] -= delta[3];
                delta.norm_squared()
            })
            .sum::<f64>()
    }

    fn get_data(&self) -> Vec<na::Point2<f64>> {
        self.data
            .iter()
            .zip(self.delta.iter())
            .map(|(x, d)| na::Point2::new(x[0] + d[0], x[1] + d[1]))
            .collect()
    }

    fn num_equation(&self) -> usize {
        3
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        optimizer::{
            fns::fns,
            geometric::minimize_geometric_distance,
            least_square::{iterative_reweight, least_square_fitting},
            taubin::{renormalization, taubin},
        },
        PrintDebug,
    };

    use super::*;
    use anyhow::Result;

    use rand::Rng;

    const LOOP_NUM: usize = 100;

    fn create_random_homography() -> na::DMatrix<f64> {
        let mut rng = rand::thread_rng();
        loop {
            let matrix = na::DMatrix::from_fn(3, 3, |_, _| rng.gen::<f64>());
            let det = matrix.determinant().abs();
            if 0.9 < det && det < 1.1 {
                return matrix;
            }
        }
    }

    fn create_random_points(homo: &na::DMatrix<f64>) -> Vec<na::Point2<f64>> {
        create_random_points_impl(homo, 0.0)
    }

    fn create_random_points_impl(
        homo: &na::DMatrix<f64>,
        noise_scale: f64,
    ) -> Vec<na::Point2<f64>> {
        let mut rng = rand::thread_rng();

        (0..100)
            .map(|_| {
                let vec0 = na::DVector::from_vec(vec![rng.gen::<f64>(), rng.gen::<f64>(), 1.0]);
                let vec1 = homo * &vec0;
                let vec1 = &vec1 / vec1[2];
                let dx0 = (rng.gen::<f64>() - 0.5) * noise_scale;
                let dy0 = (rng.gen::<f64>() - 0.5) * noise_scale;
                let dx1 = (rng.gen::<f64>() - 0.5) * noise_scale;
                let dy1 = (rng.gen::<f64>() - 0.5) * noise_scale;
                vec![
                    na::Point2::new(vec0[0] + dx0, vec0[1] + dy0),
                    na::Point2::new(vec1[0] + dx1, vec1[1] + dy1),
                ]
            })
            .flatten()
            .collect()
    }

    fn test_template<F>(func: F) -> f64
    where
        F: Fn(&[na::Point2<f64>]) -> Result<na::DVector<f64>>,
    {
        let homo = create_random_homography().normalize();
        let pts = create_random_points_impl(&homo, 0.005);

        let res = func(&pts).unwrap().normalize();
        let mut res = na::DMatrix::from_row_slice(3, 3, res.as_slice());
        if res[(2, 2)] < 0.0 {
            res *= -1.0;
        }

        println!("GT : ");
        homo.print();
        println!("Pred : ");
        res.print();
        // assert!(
        //     (&homo - &res).norm_squared() < 1e-2,
        //     "res = {}",
        //     (&homo - &res).norm_squared()
        // );
        println!("Diff : {}", (&homo - &res).norm_squared());
        (&homo - &res).norm_squared()
    }

    #[test]
    fn test_simple() {
        let homo = na::DMatrix::<f64>::from_diagonal(&na::DVector::from_vec(vec![1.0, 1.0, 1.0]))
            .normalize();
        let pts = create_random_points(&homo);
        let res = least_square_fitting::<HomographyData>(&pts)
            .unwrap()
            .normalize();
        let mut res = na::DMatrix::from_row_slice(3, 3, res.as_slice());
        if res[(2, 2)] < 0.0 {
            res *= -1.0;
        }
        assert!(
            (&homo - &res).norm_squared() < 1e-5,
            "res = {}",
            (&homo - &res).norm_squared()
        );
    }

    #[test]
    fn test_least_square() {
        let homo = create_random_homography().normalize();
        let pts = create_random_points(&homo);

        let res = least_square_fitting::<HomographyData>(&pts)
            .unwrap()
            .normalize();
        let mut res = na::DMatrix::from_row_slice(3, 3, res.as_slice());
        if res[(2, 2)] < 0.0 {
            res *= -1.0;
        }

        println!("GT : ");
        homo.print();
        println!("Pred : ");
        res.print();
        assert!(
            (&homo - &res).norm_squared() < 1e-5,
            "res = {}",
            (&homo - &res).norm_squared()
        );
    }

    #[test]
    fn test_least_square_with_noise() {
        let res: usize = (0..LOOP_NUM)
            .map(|_| test_template(|pts| least_square_fitting::<HomographyData>(pts)))
            .map(|val| if val < 1e-4 { 1 } else { 0 })
            .sum();
        assert!(
            res as f64 > LOOP_NUM as f64 * 0.9,
            "success : {} / {}",
            res,
            LOOP_NUM
        );
    }

    #[test]
    fn test_iterative_reweight() {
        let res: usize = (0..LOOP_NUM)
            .map(|_| test_template(|pts| iterative_reweight::<HomographyData>(pts)))
            .map(|val| if val < 1e-4 { 1 } else { 0 })
            .sum();
        assert!(
            res as f64 > LOOP_NUM as f64 * 0.9,
            "success : {} / {}",
            res,
            LOOP_NUM
        );
    }

    #[test]
    fn test_taubin() {
        let res: usize = (0..LOOP_NUM)
            .map(|_| test_template(|pts| taubin::<HomographyData>(pts)))
            .map(|val| if val < 1e-4 { 1 } else { 0 })
            .sum();
        println!("success : {} / {}", res, LOOP_NUM);
        assert!(
            res as f64 > LOOP_NUM as f64 * 0.9,
            "success : {} / {}",
            res,
            LOOP_NUM
        );
    }

    #[test]
    fn test_renormalization() {
        let res: usize = (0..LOOP_NUM)
            .map(|_| test_template(|pts| renormalization::<HomographyData>(pts)))
            .map(|val| if val < 1e-4 { 1 } else { 0 })
            .sum();
        println!("success : {} / {}", res, LOOP_NUM);
        assert!(
            res as f64 > LOOP_NUM as f64 * 0.9,
            "success : {} / {}",
            res,
            LOOP_NUM
        );
    }

    #[test]
    fn test_fns() {
        let res: usize = (0..LOOP_NUM)
            .map(|_| test_template(|pts| fns::<HomographyData>(pts)))
            .map(|val| if val < 1e-4 { 1 } else { 0 })
            .sum();
        println!("success : {} / {}", res, LOOP_NUM);
        assert!(
            res as f64 > LOOP_NUM as f64 * 0.9,
            "success : {} / {}",
            res,
            LOOP_NUM
        );
    }

    #[test]
    fn test_geometric() {
        let res: usize = (0..LOOP_NUM)
            .map(|_| test_template(|pts| minimize_geometric_distance::<HomographyData>(pts)))
            .map(|val| if val < 1e-4 { 1 } else { 0 })
            .sum();
        println!("success : {} / {}", res, LOOP_NUM);
        assert!(
            res as f64 > LOOP_NUM as f64 * 0.9,
            "success : {} / {}",
            res,
            LOOP_NUM
        );
    }
}
