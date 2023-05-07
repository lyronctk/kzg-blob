use config_file::FromConfigFile;
use halo2_base::halo2_proofs::{
    halo2curves::bn256::{Fr, G1, G2},
};
use rand::prelude::*;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use serde::{Deserialize, Serialize};
use std::fs::File;

mod util;
use util::poly::Polynomial;

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
    let cfg = Config::from_config_file(CONFIG_F).unwrap();
    let pp: pp = serde_json::from_reader(File::open(cfg.pp_f).unwrap()).unwrap();

    let mut rng = ChaCha8Rng::seed_from_u64(123);
    let dummy_blob: Vec<Fr> = (0..cfg.blob_len)
        .map(|_| Fr::from(rng.gen::<u64>()))
        .collect();

    let blob_idxs: Vec<Fr> = (0..cfg.blob_len).map(|x| Fr::from(x)).collect();
    let p: Polynomial<Fr> = Polynomial::from_points(&blob_idxs, &dummy_blob);

    let open_idxs: Vec<Fr> = cfg.openings.iter().map(|idx| Fr::from(*idx)).collect();
    let open_vals: Vec<Fr> = cfg
        .openings
        .iter()
        .map(|idx| dummy_blob[*idx as usize])
        .collect();
    let r: Polynomial<Fr> = Polynomial::from_points(&open_idxs, &open_vals);

    let mut z: Polynomial<Fr> = Polynomial::(cfg.openings);

    let mut z_poly: Polynomial<Fr> = Polynomial::new(vec![Fr::from(1)]);
    for open_idx in cfg.openings {
        // Mult by (X - z_i)
        z_poly = z_poly * Polynomial::new(vec![Fr::from(open_idx).neg(), Fr::from(1)]);
    }

    let q_poly: (Polynomial<Fr>, Polynomial<Fr>) = Polynomial::div_euclid(&(p - r), z_poly);
    println!("q: {:?}", q_poly);

    for i in 0..6 {
        println!("== {}", i);
        println!("EVAL p: {:?}", p.eval(Fr::from(i)));
        println!("EVAL r: {:?}", r.eval(Fr::from(i)));
        println!("EVAL z: {:?}", z.eval(Fr::from(i)));
        println!("==");
    }
}
