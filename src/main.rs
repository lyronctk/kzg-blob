use halo2_base::halo2_proofs::{
    arithmetic::Field,
    halo2curves::bn256::{Fr, G1, G1Affine},
};

// Will be toxic waste in practice
const TAU: u64 = 123;

// Number of elements in blob
const N: u64 = 4; // 4096

// Number of openings 
const T: u64 = 2; // 64

fn main() {
    let tau: Fr = Fr::from(TAU);
    let mut lagrange_basis: Vec<G1> = Vec::new();
    for i in 0u64..T {
        let i_fr: Fr = Fr::from(i);
        let mut lambda: Fr = Fr::from(0);
        for j in 0u64..T {
            if i == j {
                continue;
            };
            let j_fr: Fr = Fr::from(j);
            lambda += (tau - i_fr) * (i_fr - j_fr).invert().unwrap();
        } 
        lagrange_basis.push(G1::generator() * lambda);
    }
    println!("{:?}", lagrange_basis);

    // let mut rng = OsRng;
    // let tau = G1::generator() * Fr::from(123);
    // println!("{:?}", tau);

    // let P = Some(G1Affine::random(&mut rng)).unwrap();
    // let Q = Some(G2Affine::random(&mut rng)).unwrap();
    // println!("{:?}", P);
    // println!("{:?}", Q);
}
