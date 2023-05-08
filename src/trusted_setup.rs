/*
   Runs setup() for KZG multi-open scheme. Computes the powers of tau in G1
   and G2. Serialized into JSON in out/ directory.
*/

use config_file::FromConfigFile;
use halo2_base::halo2_proofs::halo2curves::bn256::{Fr, G1, G2};
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Deserialize)]
#[allow(dead_code)]
struct Config {
    blob_len: u64,
    openings: Vec<u64>,
    pp_f: String,
    blob_data_f: String,
    blob_commit_f: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_camel_case_types)]
struct pp {
    ptau_g1: Vec<G1>,
    ptau_g2: Vec<G2>,
}

// Specify config file
const CONFIG_F: &str = "configs/debug.json";

// Will be toxic waste in practice
const TAU: u64 = 123;

fn main() {
    let cfg = Config::from_config_file(CONFIG_F).unwrap();
    let tau: Fr = Fr::from(TAU);

    // Powers of tau in G1 to commit to polynomials p(X) and q(X)
    let mut ptau_g1: Vec<G1> = vec![G1::generator()];
    for _ in 1..cfg.blob_len {
        ptau_g1.push(ptau_g1.last().unwrap() * tau);
    }

    // Powers of tau in G2 to commit to polynomials z(X) and r(X) 
    let mut ptau_g2: Vec<G2> = vec![G2::generator()];
    for _ in 1..cfg.openings.len() {
        ptau_g2.push(ptau_g2.last().unwrap() * tau);
    }

    // Write pp to file
    let circuit_pp = pp {
        ptau_g1: ptau_g1,
        ptau_g2: ptau_g2,
    };
    let _ = serde_json::to_writer(&File::create(cfg.pp_f).unwrap(), &circuit_pp);
}
