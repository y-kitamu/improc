//! Homography matrix
use nalgebra as na;

use crate::{linalg::matrix::pseudo_inverse, optimizer::ObservedData, PrintDebug};

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
            (0..weight_vector.len())
                .map(|idx| if idx % 3 == 0 { 1.0 } else { 0.0 })
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
                                println!("i = {}, j = {}", i, j);
                                (&xi_i * xi_j.transpose()).print();
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
        todo!()
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
        linalg::vector_cross_matrix, optimizer::least_square::least_square_fitting, PrintDebug,
    };

    use super::*;

    use rand::Rng;

    fn create_random_homography() -> na::DMatrix<f64> {
        let mut rng = rand::thread_rng();
        loop {
            let matrix = na::DMatrix::from_fn(3, 3, |_, _| rng.gen::<f64>());
            if matrix.determinant().abs() > 1e-2 {
                return matrix;
            }
        }
    }

    fn create_random_points(homo: &na::DMatrix<f64>) -> Vec<na::Point2<f64>> {
        let mut rng = rand::thread_rng();

        (0..20)
            .map(|_| {
                let vec0 = na::DVector::from_vec(vec![rng.gen::<f64>(), rng.gen::<f64>(), 1.0]);
                let vec1 = homo * &vec0;
                let vec1 = &vec1 / vec1[2];
                vec![
                    na::Point2::new(vec0[0], vec0[1]),
                    na::Point2::new(vec1[0], vec1[1]),
                ]
            })
            .flatten()
            .collect()
    }

    // #[test]
    fn test_leaset_square() {
        let homo = create_random_homography().normalize();
        let pts = create_random_points(&homo);

        (0..20).for_each(|idx| {
            let p0 = pts[idx * 2];
            let p1 = pts[idx * 2 + 1];
            let v0 = na::DVector::from_vec(vec![p0[0], p0[1], 1.0]);
            let v1 = na::DVector::from_vec(vec![p1[0], p1[1], 1.0]);
            let res = vector_cross_matrix(&v1) * &homo * &v0;
            assert!(res.norm() < 1e-5);
        });

        let res = least_square_fitting::<HomographyData>(&pts)
            .unwrap()
            .normalize();
        let res = na::DMatrix::from_row_slice(3, 3, res.as_slice());

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
}
