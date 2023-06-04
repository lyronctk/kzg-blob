/*
 * Showcases Blob usage.
 */
use concat_arrays::concat_arrays;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use rand::prelude::*;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use std::fs::File;

use kzgblob::blob::{Blob, CircuitInputs};

// Debugging parameters
const BLOB_LEN: u64 = 4;
const OPENINGS: [u64; 2] = [2, 3];
const BLOB_STR: &str = "{from: 0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8, 0xac03bb73b6a9e108530aff4df5077c2b3d481e5a}";
const TX_FR_REPR_LENGTH: usize = 96;
const BYTES_IN_FR: usize = 32;

// Output files
const OUT_PP: &str = "out/pp.json";
const OUT_CIRCUIT: &str = "out/circuit_inputs.json";

// Will be toxic waste in practice
const TAU: u64 = 321;

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
 * Convert a struct into an array of bytes.
 * From https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
 */
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

/*
 * Converts array of bytes to a vector of Fr field elements. Truncates when the
 * array is not a multiple of BYTES_IN_FR 
 */
fn bytes_to_fr(bytes: &[u8]) -> Vec<Fr> {
    bytes
        .to_vec()
        .chunks_exact(BYTES_IN_FR)
        .map(|chunk| Fr::from_bytes(chunk.try_into().unwrap()).unwrap())
        .collect()
}

fn main() {
    let tx: DemoTx = DemoTx {
        from: "0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8".to_string(),
        to: "0xac03bb73b6a9e108530aff4df5077c2b3d481e5a".to_string(),
        gasLimit: 21000,
        maxFeePerGas: 300,
        maxPriorityFeePerGas: 10,
        nonce: 0,
        value: 10000000000,
    };
    let bytes: &[u8] = unsafe { any_as_u8_slice(&tx) };
    println!("fr: {:?}", bytes_to_fr(bytes));

    return;

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

    // Write public parameters & circuit inputs to json
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
