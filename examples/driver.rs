/*
 * Showcases Blob usage.
 */
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use rand::prelude::*;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use std::fs::File;

use kzgblob::blob::{Blob, CircuitInputs};

// Debugging parameters
const BLOB_LEN: u64 = 4;
const OPENINGS: [u64; 2] = [2, 3];

// Output files
const OUT_PP: &str = "out/pp.json";
const OUT_CIRCUIT: &str = "out/circuit_inputs.json";

// Will be toxic waste in practice
const TAU: u64 = 321;

fn main() {
    // Dummy data
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let dummy_data: Vec<Fr> = (0..BLOB_LEN).map(|_| Fr::from(rng.gen::<u64>())).collect();
    
    // Run mock trusted setup
    let pp = Blob::mock_trusted_setup(TAU, BLOB_LEN, OPENINGS.len() as u64);

    // Commit to blob data
    let blob: Blob = Blob::new(&dummy_data, pp.clone());
    let p_bar = blob.commit();

    // Compute opening proof
    let (q_bar, z_coeffs, r_coeffs) = blob.open_prf(OPENINGS.to_vec());

    // Write public parameters & circuit inputs tojson 
    let circuit_inputs = CircuitInputs {
        p_bar: p_bar,
        open_idxs: OPENINGS.iter().map(|idx| Fr::from(*idx)).collect(),
        open_vals: OPENINGS
            .iter()
            .map(|idx| dummy_data[*idx as usize])
            .collect(),
        q_bar: q_bar,
        z_coeffs: z_coeffs,
        r_coeffs: r_coeffs,
    };
    let _ = serde_json::to_writer(&File::create(OUT_PP).unwrap(), &pp);
    let _ = serde_json::to_writer(&File::create(OUT_CIRCUIT).unwrap(), &circuit_inputs);
}
