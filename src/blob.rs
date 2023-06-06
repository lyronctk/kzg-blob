/*
 * Client implementation for reading the proto-danksharding blobs. Covers
 *   1) what happens in the Ethereum nodes when first committing to the data and
 *   2) logic that must happen in the prover at a later time to prove openings
 *      to the blob commitment
 */
use halo2_base::halo2_proofs::{halo2curves::{bn256::{Fr, G1Affine, G1, G2}, group::ff::PrimeField, FieldExt}, arithmetic::{best_fft, lagrange_interpolate, eval_polynomial, Field}};
use serde::{Deserialize, Serialize};
use halo2_base::halo2_proofs::halo2curves::group::Group;

use crate::poly::Polynomial;

#[derive(Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct pp {
    pub K: u32,
    pub ptau_g1: Vec<G1>,
    pub ptau_g2: Vec<G2>,
    // Commitment to the lagrange bases
    pub ptau_Lis: Vec<G1>,
}

pub struct Blob {
    pp: pp,
    data: Vec<Fr>,
    p: Polynomial<Fr>,
}

#[derive(Serialize, Deserialize)]
pub struct CircuitInputs {
    pub p_bar: G1Affine,
    pub open_idxs: Vec<Fr>,
    pub open_vals: Vec<Fr>,
    pub q_bar: G1Affine,
    pub z_coeffs: Vec<Fr>,
    pub r_coeffs: Vec<Fr>
}

impl Blob {
    pub fn root_of_unity(K: u32) -> Fr {
        Fr::root_of_unity().pow(&[2u64.pow(Fr::S - K) as u64, 0, 0, 0])
    }

    /*
     * Instantiates Blob struct w/ public parameters, blob data, and
     * polynomial p(X) that interpolates the blob data.
     */
    pub fn new(d: &Vec<Fr>, pp: pp) -> Self {
        let w = Self::root_of_unity(pp.K);
        let mut idxs = vec![w];
        for _ in 1..2u32.pow(pp.K) as usize {
            idxs.push(idxs.last().unwrap() * w);
        }
        Blob {
            pp: pp,
            data: d.clone(),
            p: Polynomial::from_points(&idxs, &d),
        }
    }

    /*
     * Computes commitment p(τ) to the polynomial p(X).
     */
    pub fn commit(&self) -> G1Affine {
        G1Affine::from(self.p.eval_ptau(&self.pp.ptau_g1))
    }

    /*
     * Computes proof for the openings idxs of the blob commitment. Done by
     * computing a quotient polynomial q(X) = [p(X) - r(X)]/z(X). Opening proof
     * is q(τ). Also saves the coefficients of z(X) and r(X) to avoid having to
     * recompute within the circuit.
     */
    pub fn open_prf(&self, idxs: Vec<u64>) -> (G1Affine, Vec<Fr>, Vec<Fr>) {
        let idxs_fr: Vec<Fr> = idxs.iter().map(|idx| Fr::from(*idx)).collect();
        let vals: Vec<Fr> = idxs.iter().map(|idx| self.data[*idx as usize]).collect();
        let r: Polynomial<Fr> = Polynomial::from_points(&idxs_fr, &vals);

        let z: Polynomial<Fr> = Polynomial::vanishing(idxs);

        let (q, rem) = Polynomial::div_euclid(&(self.p.clone() - r.clone()), &z);
        if !rem.is_zero() {
            panic!("p(X) - r(X) is not divisible by z(X). Cannot compute q(X)");
        }

        let q_bar: G1Affine = G1Affine::from(q.eval_ptau(&self.pp.ptau_g1));
        (q_bar, z.get_coeffs(), r.get_coeffs())
    }

    /*
     * Evaluate a polynomial with group elements as the coefficients
     */
    pub fn evaluate_group_polynomial(coeffs: Vec<G1>, x: Fr) -> G1 {
        let mut out = G1::identity();
        let mut x_ptr = Fr::one();
        for c in coeffs {
            out += c * x_ptr;
            x_ptr *= x;
        }
        return out;
    }

    /*
     * Convenience function for running a mock setup() for the commitment
     * scheme. This is not secure.
     */
    pub fn mock_trusted_setup(tau: u64, K: u32, n_openings: u64) -> pp {
        let blob_len = 2u32.pow(K);
        let tau_fr: Fr = Fr::from(tau);

        // Powers of tau in G1 to commit to polynomials p(X) and q(X)
        let mut ptau_g1: Vec<G1> = vec![G1::generator()];
        for _ in 1..blob_len {
            ptau_g1.push(ptau_g1.last().unwrap() * tau_fr);
        }

        // Powers of tau in G2 to commit to polynomials z(X) and r(X)
        let mut ptau_g2: Vec<G2> = vec![G2::generator()];
        for _ in 1..=n_openings {
            ptau_g2.push(ptau_g2.last().unwrap() * tau_fr);
        }

        // Compute the 
        let selected_root = Fr::root_of_unity().pow(&[2u64.pow(Fr::S - K) as u64, 0, 0, 0]);

        let mut committed_lagrange_bases: Vec<G1> = vec![];
        for i in 0..blob_len {
            let mut evals = vec![Fr::zero(); blob_len as usize];
            evals[i as usize] = Fr::one();
            let mut points = vec![Fr::one()];
            for _ in 1..blob_len {
                points.push(points.last().unwrap() * selected_root)
            }
            let coeffs = lagrange_interpolate(&points, &evals);
            committed_lagrange_bases.push(G1::generator() * eval_polynomial(&coeffs, tau_fr));
        }

        pp {
            ptau_g1,
            ptau_g2,
            ptau_Lis: committed_lagrange_bases,
            K,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_base::halo2_proofs::halo2curves::bn256::{pairing, G2Affine, Gt};
    use rand::prelude::*;
    use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

    /*
     * Runs through a smoke test example blob and verifies that the pairing
     * check passes in the end.
     */
    #[test]
    fn verify_sample_proof() {
        // Test parameters
        let tau: u64 = 123;
        let K: u32 = 2;
        let blob_len: u64 = 2u64.pow(K);
        let openings: Vec<u64> = vec![2, 3];

        // Dummy data
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let dummy_data: Vec<Fr> = (0..blob_len).map(|_| Fr::from(rng.gen::<u64>())).collect();

        // Run mock trusted setup
        let pp = Blob::mock_trusted_setup(tau, K, openings.len() as u64);

        // Commit to the blob data
        let blob: Blob = Blob::new(&dummy_data, pp.clone());
        let p_bar = blob.commit();

        // Compute the opening proof
        let (q_bar, z_coeffs, r_coeffs) = blob.open_prf(openings);
        let z: Polynomial<Fr> = Polynomial::new(z_coeffs);
        let r: Polynomial<Fr> = Polynomial::new(r_coeffs);

        // Confirm that the pairing check passes. Will be carried out in the 
        // circuit
        let z_bar: G2Affine = G2Affine::from(z.eval_ptau(&pp.ptau_g2));
        let r_bar: G1Affine = G1Affine::from(r.eval_ptau(&pp.ptau_g1));
        let lhs: Gt = pairing(&q_bar, &z_bar);
        let rhs: Gt = pairing(
            &G1Affine::from(p_bar - r_bar),
            &G2Affine::from(pp.ptau_g2[0]),
        );
        assert!(lhs == rhs);
    }
}
