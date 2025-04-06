use ark_bls12_381::{Bls12_381, Fr};
use ark_crypto_primitives::snark::SNARK;
use ark_ff::{PrimeField, UniformRand};
use ark_groth16::Groth16;
use ark_relations::{
    lc,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError},
};
use instant::Instant;
use wasm_bindgen::prelude::*;

const NUM_PROVE_REPETITIONS: usize = 1;
const NUM_CONSTRAINTS: usize = 1 << 10;
const NUM_VARIABLES: usize = 1 << 10;

#[derive(Copy, Clone)]
struct DummyCircuit<F: PrimeField> {
    pub a: Option<F>,
    pub b: Option<F>,
    pub num_variables: usize,
    pub num_constraints: usize,
}

impl<F: PrimeField> ConstraintSynthesizer<F> for DummyCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let a = cs.new_witness_variable(|| self.a.ok_or(SynthesisError::AssignmentMissing))?;
        let b = cs.new_witness_variable(|| self.b.ok_or(SynthesisError::AssignmentMissing))?;
        let c = cs.new_input_variable(|| {
            let a = self.a.ok_or(SynthesisError::AssignmentMissing)?;
            let b = self.b.ok_or(SynthesisError::AssignmentMissing)?;

            Ok(a * b)
        })?;

        for _ in 0..(self.num_variables - 3) {
            let _ = cs.new_witness_variable(|| self.a.ok_or(SynthesisError::AssignmentMissing))?;
        }

        for _ in 0..self.num_constraints - 1 {
            cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)?;
        }

        cs.enforce_constraint(lc!(), lc!(), lc!())?;

        Ok(())
    }
}

/// This function measures the performance of a proof generation process. The function is designed to be used as a benchmark to decide whether the proof generation should be done on the user's device or delegated to a server.
///
/// # Returns
/// The average time taken (in seconds) for proof generation.
///
/// # Example
///
/// ```no_run
/// // JavaScript example code
///
/// // 모든 circuit을 지원하는 minimum 설정
/// const INITIALIZE = 19;
/// const MAXIMUM = 6144;
/// const basedTime = 1.0;
///
/// let wasmPkg = await import('./pkg-with-threads/zkvoting.js');
///
/// await wasmPkg.default(undefined, new WebAssembly.Memory({
///     initial: INITIALIZE,
///     maximum: MAXIMUM,
///     shared: true
/// }));
///
/// await wasmPkg.initThreadPool(navigator.hardwareConcurrency);
///
/// const benchTime = await wasmPkg.bench();
///
/// if (benchTime > basedTime) {
///     // Process: 서버 위임 투표
/// } else {
///     // Process: 유권자 기기에서 투표
///
/// }
/// ```
///
#[wasm_bindgen(js_name = "bench")]
pub fn bench_prove() -> f64 {
    use ark_std::rand::SeedableRng;
    let rng = &mut ark_std::rand::rngs::StdRng::seed_from_u64(0u64);
    let c = DummyCircuit::<Fr> {
        a: Some(Fr::rand(rng)),
        b: Some(Fr::rand(rng)),
        num_variables: NUM_VARIABLES,
        num_constraints: NUM_CONSTRAINTS,
    };

    let (pk, _) = Groth16::<Bls12_381>::circuit_specific_setup(c, rng).unwrap();

    let start = Instant::now();

    for _ in 0..NUM_PROVE_REPETITIONS {
        let _ = Groth16::<Bls12_381>::prove(&pk, c, rng).unwrap();
    }

    start.elapsed().as_secs_f64() / NUM_PROVE_REPETITIONS as f64
}
