/*
 * For doing basic operations {Add, Sub, Mul, Div, Eval} for polynomials
 * defined over finite fields. Used Euclidean division.
 *
 * Inspired by https://applied-math-coding.medium.com/implementing-polynomial-division-rust-ca2a59370003
 */

use halo2_base::halo2_proofs::{
    arithmetic::{eval_polynomial, lagrange_interpolate, CurveExt},
    halo2curves::FieldExt,
};
use std::ops::{Add, AddAssign, Mul, Neg, Sub};

#[derive(Clone, Debug)]
pub struct Polynomial<F: FieldExt>(Vec<F>);

impl<F: FieldExt> Add for Polynomial<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut a = vec![];
        for i in 0..usize::max(self.deg(), rhs.deg()) + 1 {
            a.push(*self.0.get(i).unwrap_or(&F::zero()) + rhs.0.get(i).unwrap_or(&F::zero()));
        }
        Polynomial(a)
    }
}

impl<F: FieldExt> Sub for Polynomial<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut a = vec![];
        for i in 0..usize::max(self.deg(), rhs.deg()) + 1 {
            a.push(*self.0.get(i).unwrap_or(&F::zero()) - rhs.0.get(i).unwrap_or(&F::zero()));
        }
        Polynomial(a)
    }
}

impl<F: FieldExt> Neg for Polynomial<F> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Polynomial(self.0.iter().map(|a| a.neg()).collect())
    }
}

impl<F: FieldExt> Mul for Polynomial<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let [n, m] = [self.deg(), rhs.deg()];
        let mut a = vec![F::zero(); n + m + 1];
        for i in 0..n + 1 {
            for j in 0..m + 1 {
                a[i + j] = a[i + j] + self.0[i] * rhs.0[j];
            }
        }
        Polynomial(a)
    }
}

impl<F: FieldExt> Polynomial<F> {
    pub fn new(coeffs: Vec<F>) -> Self {
        return Self(coeffs);
    }

    /*
     * Uses lagrange interpolation to find the lowest degree polynomial that
     * passes through (points, evals). Coefficients are stored in the monomial
     * basis w/ increasing degree. Eg Coefficients of f(x) = 2 + x + 3x^2 are
     * stored as [2, 1, 3].
     */
    pub fn from_points(points: &[F], evals: &[F]) -> Self {
        Self::new(lagrange_interpolate(points, evals))
    }

    pub fn vanishing(openings: Vec<u64>) -> Self {
        let mut z: Polynomial<F> = Self::new(vec![F::one()]);
        for open_idx in openings {
            // Mult by (X - z_i), coefficients of which are [-z_i, 1]
            z = z * Self::new(vec![F::from(open_idx).neg(), F::one()]);
        }
        z
    }

    fn deg(&self) -> usize {
        self.0
            .iter()
            .enumerate()
            .rev()
            .find(|(_, a)| !bool::from(a.is_zero()))
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    pub fn eval_ptau<G: CurveExt + Mul<F, Output = G> + AddAssign>(&self, ptau: &[G]) -> G {
        if self.0.is_empty() {
            panic!("Cannot evaluate polynomial with no coefficients.");
        }
        let mut acc = G::generator() * self.0[0];
        for (i, coeff) in self.0.iter().skip(1).enumerate() {
            acc += ptau[i] * coeff.clone();
        }
        acc
    }

    pub fn eval(&self, pt: F) -> F {
        return eval_polynomial(&self.0, pt);
    }

    pub fn is_zero(&self) -> bool {
        for coeff in &self.0 {
            if !bool::from(coeff.is_zero()) {
                return false;
            }
        }
        true
    }

    fn zero() -> Self {
        Polynomial(vec![F::zero()]) // represents the polynomial f(x) = 0
    }

    pub fn div_euclid(f: &Self, g: &Self) -> (Self, Self) {
        let [n, m] = [f.deg(), g.deg()];
        if n < m {
            (Self::zero(), f.clone())
        } else if n == 0 {
            if *g.0.get(0).unwrap_or(&F::zero()) == F::zero() {
                panic!("Cannot divide by 0!");
            }
            (
                Polynomial(vec![f.0[0] * g.0[0].invert().unwrap()]),
                Self::zero(),
            )
        } else {
            let [a_n, b_m] = [f.0[n], g.0[m]];
            let mut q_1 = Polynomial(vec![F::zero(); n - m + 1]);
            q_1.0[n - m] = a_n * b_m.invert().unwrap();
            let h_2 = f.clone() - q_1.clone() * g.clone();
            let (q_2, r) = Self::div_euclid(&h_2, g);
            (q_1 + q_2, r)
        }
    }
}
