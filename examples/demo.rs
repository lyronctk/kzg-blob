/*
 * Showcases Blob usage.
 */
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use rand::{
    distributions::{Alphanumeric, DistString},
    prelude::*,
};
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use std::fs::File;

use kzgblob::blob::{Blob, CircuitInputs};

// Debugging parameters
const BLOB_LEN: u64 = 32;
const OPENINGS: [u64; 2] = [2, 3];
const BYTES_IN_FR: usize = 32;
const ADDR_LEN: usize = 40;
const N_TX: u64 = BLOB_LEN / 2; // div by 2 since one tx == 2 Fr elements

// Output files
const OUT_PP: &str = "out/pp.json";
const OUT_CIRCUIT: &str = "out/circuit_inputs.json";

// Will be toxic waste in practice
const TAU: u64 = 321;

#[derive(Debug, Clone)]
struct DemoTx {
    from: String,
    to: String,
    gasLimit: u64,
    maxFeePerGas: u64,
    maxPriorityFeePerGas: u64,
    nonce: u64,
    value: u64,
}

/*
 * Convert struct into an array of bytes.
 * From https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
 */
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

/*
 * Converts array of bytes to a vector of Fr field elements. Truncates when the
 * array is not a multiple of BYTES_IN_FR. Truncating for demo purposes only.
 */
fn bytes_to_fr(bytes: &[u8]) -> Vec<Fr> {
    bytes
        .to_vec()
        .chunks_exact(BYTES_IN_FR)
        .map(|chunk| Fr::from_bytes(chunk.try_into().unwrap()).unwrap())
        .collect()
}

/*
 * Generate a random DemoTx.
 */
fn random_tx(rng: &mut ThreadRng) -> DemoTx {
    DemoTx {
        from: format!("0x{}", Alphanumeric.sample_string(rng, ADDR_LEN)),
        to: format!("0x{}", Alphanumeric.sample_string(rng, ADDR_LEN)),
        gasLimit: rng.gen_range(10000..30000),
        maxFeePerGas: rng.gen_range(100..500),
        maxPriorityFeePerGas: rng.gen_range(5..15),
        nonce: rng.gen_range(0..30),
        value: rng.gen_range(10000000000..1000000000000),
    }
}

fn main() {
    let mut rng = rand::thread_rng();

    println!(
        "== Generating {} random transactions to put into a blob",
        N_TX
    );
    let blob_txs: Vec<DemoTx> = (0..N_TX).map(|_| random_tx(&mut rng)).collect();
    println!("- {:?}", blob_txs);
    println!("==");

    println!("== Representing the blob as field elements in Fr");
    let blob: Vec<Fr> = blob_txs
        .into_iter()
        .map(|tx| bytes_to_fr(unsafe { any_as_u8_slice(&tx) }))
        .flatten()
        .collect();
    println!("- {:?}", blob);
    println!("==");

    // // Run mock trusted setup
    // let pp = Blob::mock_trusted_setup(TAU, BLOB_LEN, OPENINGS.len() as u64);

    // // Commit to blob data
    // let blob: Blob = Blob::new(&dummy_data, pp.clone());
    // let p_bar = blob.commit();

    // // Compute opening proof
    // let (q_bar, z_coeffs, r_coeffs) = blob.open_prf(OPENINGS.to_vec());

    // // Write public parameters & circuit inputs to json
    // let circuit_inputs = CircuitInputs {
    //     p_bar: p_bar,
    //     open_idxs: OPENINGS.iter().map(|idx| Fr::from(*idx)).collect(),
    //     open_vals: OPENINGS
    //         .iter()
    //         .map(|idx| dummy_data[*idx as usize])
    //         .collect(),
    //     q_bar: q_bar,
    //     z_coeffs: z_coeffs,
    //     r_coeffs: r_coeffs,
    // };
    // let _ = serde_json::to_writer(&File::create(OUT_PP).unwrap(), &pp);
    // let _ = serde_json::to_writer(&File::create(OUT_CIRCUIT).unwrap(), &circuit_inputs);
}
