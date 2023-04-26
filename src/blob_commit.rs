use config_file::FromConfigFile;
use halo2_base::halo2_proofs::halo2curves::bn256::{Fr, G1, G1Affine, G2};
use serde::{Deserialize, Serialize};
use std::fs::File;

// Specify config file
const CONFIG_F: &str = "configs/debug.json";

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
    let pp: pp = serde_json::from_reader(File::open(cfg.pp_f).unwrap()).unwrap();

    let dummy_blob: Vec<Fr> = (1..=cfg.n).map(|x| Fr::from(x * 1000)).collect();
    let blob_commit: G1 = pp
        .lagrange_basis
        .iter()
        .enumerate()
        .map(|(i, l)| l * dummy_blob[i])
        .sum();
    println!("{:?}", G1Affine::from(blob_commit));
    println!("{:?}", G1Affine::from(G1::generator() * Fr::from(3000)));
}
