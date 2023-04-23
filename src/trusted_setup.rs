/*
    Executes setup() for KZG multi-open scheme. Computes the powers of tau in G1,
    lagrange basis in G1, and tau in G2. Serialized into JSON in out/ directory. 
 */

use halo2_base::halo2_proofs::{
    arithmetic::Field,
    halo2curves::bn256::{Fr, G1, G2},
};
use std::fs::File;
use serde::{Deserialize, Serialize};

// Will be toxic waste in practice
const TAU: u64 = 123;

// Number of elements in blob
const N: u64 = 4; // 4096

// Number of openings
const T: u64 = 2; // 16

// Write pp here
const OUT: &str = "out/pp_debug.json";

#[derive(Serialize, Deserialize)]
struct pp {
    ptau: Vec<G1>,
    lagrange_basis: Vec<G1>,
    tau_g2: G2,
}

fn main() {
    let tau: Fr = Fr::from(TAU);

    // Powers of tau to commit to blob polynomial p(X)
    let mut ptau: Vec<G1> = vec![G1::generator() * tau];
    for _ in 1u64..N {
        ptau.push(ptau.last().unwrap() * tau);
    }

    // Lagrange basis to commit to interpolation polynomial r(X)
    let mut lagrange_basis: Vec<G1> = Vec::new();
    for i in 0u64..T {
        let i_fr: Fr = Fr::from(i);
        let mut lambda: Fr = Fr::from(0);
        for j in 0u64..T {
            if i == j {
                continue;
            };
            let j_fr: Fr = Fr::from(j);
            lambda += (tau - i_fr) * (i_fr - j_fr).invert().unwrap();
        }
        lagrange_basis.push(G1::generator() * lambda);
    }

    // Compute tau in G2 for the pairing check
    let tau_g2 = G2::generator() * tau;

    // Write pp to file 
    let circuit_pp = pp {
        ptau: ptau,
        lagrange_basis: lagrange_basis,
        tau_g2: tau_g2,
    };
    serde_json::to_writer(&File::create(OUT).unwrap(), &circuit_pp);
}
