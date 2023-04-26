use config_file::FromConfigFile;
use halo2_base::halo2_proofs::{
    arithmetic::{eval_polynomial, lagrange_interpolate},
    halo2curves::bn256::{Fr, G1Affine, G1, G2},
};
use rand::prelude::*;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use serde::{Deserialize, Serialize};
use std::fs::File;

// Specify config file
const CONFIG_F: &str = "configs/debug.json";

#[derive(Deserialize)]
#[allow(dead_code)]
struct Config {
    blob_len: u64,
    log_blob_len: u64,
    openings: Vec<u64>,
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
    let mut rng = ChaCha8Rng::seed_from_u64(123);

    let cfg = Config::from_config_file(CONFIG_F).unwrap();
    let pp: pp = serde_json::from_reader(File::open(cfg.pp_f).unwrap()).unwrap();

    let dummy_blob: Vec<Fr> = (0..cfg.blob_len)
        .map(|x| Fr::from(rng.gen::<u64>()))
        .collect();

    let blob_idxs: Vec<Fr> = (0..cfg.blob_len).map(|x| Fr::from(x)).collect();
    let p_coeffs: Vec<Fr> = lagrange_interpolate(&blob_idxs, &dummy_blob);

    let open_idxs: Vec<Fr> = cfg.openings.iter().map(|idx| Fr::from(*idx)).collect();
    let open_vals: Vec<Fr> = cfg
        .openings
        .iter()
        .map(|idx| dummy_blob[*idx as usize])
        .collect();
    let r_coeffs: Vec<Fr> = lagrange_interpolate(&open_idxs, &open_vals);

    println!("p: {:?}", p_coeffs);
    println!("r: {:?}", r_coeffs);

    for i in 0..5 {
        println!("EVAL p: {:?}", eval_polynomial(&p_coeffs, Fr::from(i)));
        println!("EVAL r: {:?}", eval_polynomial(&r_coeffs, Fr::from(i)));
    }
}
