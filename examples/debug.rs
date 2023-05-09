/*
   Runs setup() for KZG multi-open scheme. Computes the powers of tau in G1
   and G2. Serialized into JSON in out/ directory.
*/

use config_file::FromConfigFile;
use halo2_base::halo2_proofs::halo2curves::bn256::{Fr, G1, G2};
use rand::prelude::*;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use serde::{Deserialize, Serialize};
use std::fs::File;

use kzgblob::blob::{Blob, pp};

#[derive(Deserialize)]
#[allow(dead_code)]
struct Config {
    blob_len: u64,
    openings: Vec<u64>,
    pp_f: String,
    blob_data_f: String,
    blob_commit_f: String,
}

// Specify config file
const CONFIG_F: &str = "configs/debug.json";

// Will be toxic waste in practice
const TAU: u64 = 321;

fn main() {
    let cfg = Config::from_config_file(CONFIG_F).unwrap();
    let pp = trusted_setup(TAU, cfg.blob_len, cfg.openings.len() as u64);

    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let dummy_data: Vec<Fr> = (0..cfg.blob_len)
        .map(|_| Fr::from(rng.gen::<u64>()))
        .collect();
    let blob: Blob = Blob::new(&dummy_data, pp);
    let p_bar = blob.commit();
    let (q_bar, z_coeffs, r_coeffs) = blob.open_prf(cfg.openings);

    // let z_bar: G2Affine = G2Affine::from(z.eval_ptau(&pp.ptau_g2));
    // let r_bar: G1Affine = G1Affine::from(r.eval_ptau(&pp.ptau_g1));
    // let lhs: Gt = pairing(&q_bar, &z_bar);
    // let rhs: Gt = pairing(
    //     &G1Affine::from(p_bar - r_bar),
    //     &G2Affine::from(pp.ptau_g2[0]),
    // );
    // println!("pairing check: {:?}", lhs == rhs);
}
