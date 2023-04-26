/*
   Executes setup() for KZG multi-open scheme. Computes the powers of tau in G1,
   lagrange basis in G1, and tau in G2. Serialized into JSON in out/ directory.
*/

use config_file::FromConfigFile;
use halo2_base::halo2_proofs::{
    arithmetic::Field,
    halo2curves::bn256::{Fr, G1, G2},
};
use serde::{Deserialize, Serialize};
use std::fs::File;

// Specify config file
const CONFIG_F: &str = "configs/debug.json";

// Will be toxic waste in practice
const TAU: u64 = 123;

#[derive(Deserialize)]
#[allow(dead_code)]
struct Config {
    n: u64,
    t: u64,
    pp_f: String,
    blob_data_f: String,
    blob_commit_f: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_camel_case_types)]
struct pp {
    ptau: Vec<G1>,
    lagrange_basis: Vec<G1>,
    tau_g2: G2,
}

fn main() {
    let cfg = Config::from_config_file(CONFIG_F).unwrap();
    let tau: Fr = Fr::from(TAU);

    // Powers of tau to commit to blob polynomial p(X)
    let mut ptau: Vec<G1> = vec![G1::generator() * tau];
    for _ in 1u64..cfg.n {
        ptau.push(ptau.last().unwrap() * tau);
    }

    // Lagrange basis to commit to interpolation polynomial r(X)
    let mut lagrange_basis: Vec<G1> = Vec::new();
    for i in 0u64..cfg.n {
        let i_fr: Fr = Fr::from(i);
        let mut lambda: Fr = Fr::from(1);
        for j in 0u64..cfg.n {
            if i == j {
                continue;
            };
            let j_fr: Fr = Fr::from(j);
            lambda *= (tau - j_fr) * (i_fr - j_fr).invert().unwrap();
        }
        println!("lambda {}: {:?}", i, lambda);
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
    let _ = serde_json::to_writer(&File::create(cfg.pp_f).unwrap(), &circuit_pp);
}
