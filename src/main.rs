use halo2_base::halo2_proofs::{
    arithmetic::Field,
    halo2curves::bn256::{Fr, G1, G2},
};

// Will be toxic waste in practice
const TAU: u64 = 123;

// Number of elements in blob
const N: u64 = 4; // 4096

// Number of openings 
const T: u64 = 2; // 64

fn main() {
    let tau: Fr = Fr::from(TAU);

    // Powers of tau to commit to blob polynomial p(X)
    let mut powers: Vec<G1> = vec![G1::generator() * tau];
    for _ in 1u64..T {
        powers.push(powers.last().unwrap() * tau);
    }
    
    // Lagrange basis to commit to interpolation polynomial r(X)
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

    println!("{:?}", G2::generator() * tau);
    println!("{:?}", powers);
    println!("{:?}", lagrange_basis);
}
