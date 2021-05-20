use dusk_bls12_381::BlsScalar;
use dusk_bytes::Serializable;
use dusk_jubjub::{JubJubAffine, JubJubScalar};
use dusk_plonk::prelude::*;
use rand_core::OsRng;

#[derive(Debug, Default)]
pub struct TestCircuit {
    a: BlsScalar,
    f: JubJubAffine,
}

fn main() -> Result<(), Error> {
    let scalar = JubJubScalar::from_bytes_wide(&[
        182, 44, 247, 214, 94, 14, 151, 208, 130, 16, 200, 204, 147, 32, 104, 166, 0, 59, 52, 1, 1,
        59, 103, 6, 169, 175, 51, 101, 234, 180, 125, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    let a = BlsScalar::from_bytes(&scalar.to_bytes()).unwrap();
    impl Circuit for TestCircuit {
        const CIRCUIT_ID: [u8; 32] = [0xff; 32];
        fn gadget(&mut self, composer: &mut StandardComposer) -> Result<(), Error> {
            let secret_scalar = composer.add_input(self.a);

            let point_scalar =
                composer.fixed_base_scalar_mul(secret_scalar, dusk_jubjub::GENERATOR_EXTENDED);

            composer.assert_equal_public_point(point_scalar, self.f);
            println!("circuit size {:}", composer.circuit_size());
            Ok(())
        }
        fn padded_circuit_size(&self) -> usize {
            1 << 11
        }
    }

    // Now let's use the Circuit we've just implemented!

    let pp = PublicParameters::setup(1 << 12, &mut OsRng)?;
    // Initialize the circuit
    let mut circuit = TestCircuit::default();

    // Compile the circuit
    let (pk, vd) = circuit.compile(&pp)?;
    // Prover POV
    let proof = {
        let mut circuit = TestCircuit {
            a: a,
            f: JubJubAffine::from(dusk_jubjub::GENERATOR_EXTENDED * scalar),
        };
        circuit.gen_proof(&pp, &pk, b"Test")
    }?;

    // Verifier POV
    let public_inputs: Vec<PublicInputValue> =
        vec![JubJubAffine::from(dusk_jubjub::GENERATOR_EXTENDED * scalar).into()];

    circuit::verify_proof(
        &pp,
        &vd.key(),
        &proof,
        &public_inputs,
        &vd.pi_pos(),
        b"Test",
    )
}
