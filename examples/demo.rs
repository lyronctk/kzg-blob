/*
 * Showcases Blob usage.
 */
use halo2_base::halo2_proofs::{halo2curves::{bn256::{Fr, G1}, group::Group}, poly::kzg::msm};
use rand::{
    distributions::{Alphanumeric, DistString},
    prelude::*,
};

// Demo transactions
const BYTES_IN_FR: usize = 32; // element in FR is 32 bytes wide
const FR_PER_TX: usize = 2; // a DemoTx can be represented by two FR elements 
const ADDR_LEN: usize = 40; // demo addresses are 40 characters 

// Blob parameters
const K: u32 = 5;
const BLOB_LEN: u64 = 2u64.pow(K);
const N_TX: u64 = BLOB_LEN / FR_PER_TX as u64;
const OPEN_TX_IDX: usize = 5;

// Will be toxic waste in practice
const TAU: u64 = 321;

#[derive(Debug, Clone)]
struct DemoTx {
    from: String,
    to: String,
    gas_limit: u64,
    max_fee_per_gas: u64,
    max_priority_fee_per_gas: u64,
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
        gas_limit: rng.gen_range(10000..30000),
        max_fee_per_gas: rng.gen_range(100..500),
        max_priority_fee_per_gas: rng.gen_range(5..15),
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
        .iter()
        .map(|tx| bytes_to_fr(unsafe { any_as_u8_slice(&tx.clone()) }))
        .flatten()
        .collect();
    println!("- {:?}", blob);
    println!("==");

    println!("== Running mock trusted setup");
    let pp = Blob::mock_trusted_setup(TAU, K, FR_PER_TX as u64);
    println!("- Done");
    println!("==");

    println!("== Committing to the blob data");
    let blob: Blob = Blob::new(&blob, pp.clone());
    let p_bar = blob.commit();
    println!("- {:?}", p_bar);
    println!("==");

    println!("== Computing the opening proof for this tx");
    let open_idxs: Vec<u64> = (OPEN_TX_IDX * FR_PER_TX..(OPEN_TX_IDX + 1) * FR_PER_TX)
        .collect::<Vec<usize>>()
        .into_iter()
        .map(|x| x as u64)
        .collect();
    let (q_bar, z_coeffs, r_coeffs) = blob.open_prf(&open_idxs);
    println!("- {:?}", blob_txs[OPEN_TX_IDX]);
    println!("- {:?}", q_bar);
    println!("==");

    // TODO: Move this to a test
    // Perform a check to ensure that r_coeffs and the committed lagrange bases evaluate to the same thing
    let mut r_eval_left = G1::identity();
    for (ix, c) in r_coeffs.iter().enumerate() {
        r_eval_left += pp.ptau_g1[ix] * c;
    }

    let r_points: Vec<_> = open_idxs.iter().map(|idx| blob.data[*idx as usize]).collect();
    let mut r_eval_right = G1::identity();
    for (ix, r) in open_idxs.iter().zip(r_points) {
        r_eval_right += pp.ptau_lis[*ix as usize] * r;
    }
    assert_eq!(r_eval_left, r_eval_right);
    
    // Write public parameters & circuit inputs to json
    let circuit_inputs = CircuitInputs {
        p_bar: p_bar,
        open_idxs: open_idxs.iter().map(|idx| Fr::from(*idx)).collect(),
        open_vals: open_idxs
            .iter()
            .map(|idx| blob.data[*idx as usize])
            .collect(),
        q_bar: q_bar,
        z_coeffs: z_coeffs,
        r_coeffs: r_coeffs,
    };
    let _ = serde_json::to_writer(&File::create(OUT_PP).unwrap(), &pp);
    let _ = serde_json::to_writer(&File::create(OUT_CIRCUIT).unwrap(), &circuit_inputs);
}
