use std::f32::consts::E;

use ndarray::prelude::*;
use ndarray::ViewRepr;

pub fn sigmoid(x: &f32, stretch: f32) -> f32 {
    1.0 / (1.0 + E.powf(-(x / stretch)))
}

pub fn update_with_laplace_operator_1(
    dimx: usize,
    dimy: usize,
    tau: &Array2<f32>,
    u: &Array3<f32>,
) -> Array2<f32> {
    let alphas: ArrayBase<ViewRepr<&f32>, Dim<[usize; 2]>> =
        tau.slice(s![1..(dimx - 1), 1..(dimy - 1)]);

    let laplace_operator: Array2<f32> = -4.0
        * &u.slice(s![1, 1..(dimx - 1), 1..(dimy - 1)])
        + u.slice(s![1, 0..(dimx - 2), 1..(dimy - 1)])
        + u.slice(s![1, 2..dimx, 1..(dimy - 1)])
        + u.slice(s![1, 1..(dimx - 1), 0..(dimy - 2)])
        + u.slice(s![1, 1..(dimx - 1), 2..dimy]);

    let prev: Array2<f32> = 2.0 * &u.slice(s![1, 1..(dimx - 1), 1..(dimy - 1)])
        - u.slice(s![2, 1..(dimx - 1), 1..(dimy - 1)]);

    laplace_operator * alphas + prev
}

pub fn update_with_laplace_operator_4(
    dimx: usize,
    dimy: usize,
    tau: &Array2<f32>,
    u: &Array3<f32>,
) -> Array2<f32> {
    let alphas: ArrayBase<ViewRepr<&f32>, Dim<[usize; 2]>> =
        tau.slice(s![4..dimx - 4, 4..dimy - 4]);

    let laplace_operator: Array2<f32> = -1.0 / 500.0 * &u.slice(s![1, 4..dimx-4, 0..dimy-8])                     // c, r - 4
        + 8.0/315.0 * &u.slice(s![1, 4..dimx-4, 1..dimy-7])         // c, r - 3
        - 1.0/5.0 * &u.slice(s![1, 4..dimx-4, 2..dimy-6])           // c, r - 2
        + 8.0/5.0 * &u.slice(s![1, 4..dimx-4, 3..dimy-5])           // c, r - 1

        - 1.0/560.0 * &u.slice(s![1, 0..dimx-8, 4..dimy-4])         // c - 4, r
        + 8.0/315.0 * &u.slice(s![1, 1..dimx-7, 4..dimy-4])         // c - 3, r
        - 1.0/5.0 * &u.slice(s![1, 2..dimx-6, 4..dimy-4])
        + 8.0/5.0 * &u.slice(s![1, 3..dimx-5, 4..dimy-4])
        - 410.0/72.0 * &u.slice(s![1, 4..dimx-4, 4..dimy-4])        // c, r
        + 8.0/5.0 * &u.slice(s![1, 5..dimx-3, 4..dimy-4])           // c + 1, r
        - 1.0/5.0 * &u.slice(s![1, 6..dimx-2, 4..dimy-4])
        + 8.0/315.0 * &u.slice(s![1, 7..dimx-1, 4..dimy-4])
        - 1.0/560.0 * &u.slice(s![1, 8..dimx, 4..dimy-4])

        + 8.0/5.0 * &u.slice(s![1, 4..dimx-4, 5..dimy-3])           // c, r + 1
        - 1.0/5.0 * &u.slice(s![1, 4..dimx-4, 6..dimy-2])
        + 8.0 / 325.0 * &u.slice(s![1, 4..dimx - 4, 7..dimy - 1])
        - 1.0 / 560.0 * &u.slice(s![1, 4..dimx - 4, 8..dimy]);

    let prev: Array2<f32> = 2.0 * &u.slice(s![1, 4..dimx - 4, 4..dimy - 4])
        - u.slice(s![2, 4..dimx - 4, 4..dimy - 4]);

    laplace_operator * alphas + prev
}

pub fn update_with_absorbing_boundary(
    dimx: usize,
    dimy: usize,
    sz: usize,
    kappa: &Array2<f32>,
    u: &mut Array3<f32>,
) {
    let dimx1 = dimx - 1;
    let dimx2 = dimx - 2;
    let dimy1 = dimy - 1;
    let dimy2 = dimy - 2;

    let dimx_sz1 = dimx - sz - 1;
    let dimx_sz2 = dimx - sz - 2;

    let dimy_sz1 = dimy - sz - 1;
    let dimy_sz2 = dimy - sz - 2;

    let sz_p1 = sz + 1;

    let boundary =
        (kappa.slice(s![dimx_sz1..dimx1, 1..dimy1]).mapv(|k| k - 1.0)
            / kappa.slice(s![dimx_sz1..dimx1, 1..dimy1]).mapv(|k| k + 1.0))
            * (&u.slice(s![0, dimx_sz2..dimx2, 1..dimy1])
                - &u.slice(s![1, dimx_sz1..dimx1, 1..dimy1]))
            + u.slice(s![1, dimx_sz2..dimx2, 1..dimy1]);

    u.slice_mut(s![0, dimx_sz1..dimx1, 1..dimy1])
        .assign(&boundary);

    let boundary = (kappa.slice(s![0..sz, 1..dimy1]).mapv(|k| k - 1.0)
        / kappa.slice(s![0..sz, 1..dimy1]).mapv(|k| k + 1.0))
        * (&u.slice(s![0, 1..sz_p1, 1..dimy1])
            - &u.slice(s![1, 0..sz, 1..dimy1]))
        + u.slice(s![1, 1..sz_p1, 1..dimy1]);

    u.slice_mut(s![0, 0..sz, 1..dimy1]).assign(&boundary);

    let boundary =
        (kappa.slice(s![1..dimx1, dimy_sz1..dimy1]).mapv(|k| k - 1.0)
            / kappa.slice(s![1..dimx1, dimy_sz1..dimy1]).mapv(|k| k + 1.0))
            * (&u.slice(s![0, 1..dimx1, dimy_sz2..dimy2])
                - &u.slice(s![1, 1..dimx1, dimy_sz1..dimy1]))
            + u.slice(s![1, 1..dimx1, dimy_sz2..dimy2]);

    u.slice_mut(s![0, 1..dimx1, dimy_sz1..dimy1])
        .assign(&boundary);

    let boundary = (kappa.slice(s![1..dimx1, 0..sz]).mapv(|k| k - 1.0)
        / kappa.slice(s![1..dimx1, 0..sz]).mapv(|k| k + 1.0))
        * (&u.slice(s![0, 1..dimx1, 0 + 1..sz_p1])
            - &u.slice(s![1, 1..dimx1, 0..sz]))
        + u.slice(s![1, 1..dimx1, 0 + 1..sz_p1]);

    u.slice_mut(s![0, 1..dimx1, 0..sz]).assign(&boundary);
}
