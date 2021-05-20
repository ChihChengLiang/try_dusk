use dusk_jubjub::{JubJubAffine, JubJubScalar};
use dusk_plonk::prelude::*;
use rand_core::OsRng;

// Implement a circuit that checks:
// 1) a + b = c where C is a PI
// 2) a <= 2^6
// 3) b <= 2^5
// 4) a * b = d where D is a PI
// 5) JubJub::GENERATOR * e(JubJubScalar) = f where F is a Public Input
#[derive(Debug, Default)]
pub struct TestCircuit {
    a: BlsScalar,
    b: BlsScalar,
    c: BlsScalar,
    d: BlsScalar,
    e: JubJubScalar,
    f: JubJubAffine,
}

fn main() -> Result<(), Error> {
    impl Circuit for TestCircuit {
        const CIRCUIT_ID: [u8; 32] = [0xff; 32];
        fn gadget(&mut self, composer: &mut StandardComposer) -> Result<(), Error> {
            let a = composer.add_input(self.a);
            let b = composer.add_input(self.b);
            let zero_scalar = BlsScalar::zero();
            let zero_var = composer.add_input(zero_scalar);
            // Make first constraint a + b = c
            composer.poly_gate(
                a,
                b,
                zero_var,
                BlsScalar::zero(),
                BlsScalar::one(),
                BlsScalar::one(),
                BlsScalar::zero(),
                BlsScalar::zero(),
                Some(-self.c),
            );
            // Check that a and b are in range
            composer.range_gate(a, 1 << 6);
            composer.range_gate(b, 1 << 5);
            // Make second constraint a * b = d
            composer.poly_gate(
                a,
                b,
                zero_var,
                BlsScalar::one(),
                BlsScalar::zero(),
                BlsScalar::zero(),
                BlsScalar::one(),
                BlsScalar::zero(),
                Some(-self.d),
            );

            let e = composer.add_input(self.e.into());
            let scalar_mul_result =
                composer.fixed_base_scalar_mul(e, dusk_jubjub::GENERATOR_EXTENDED);
            // Apply the constrain
            composer.assert_equal_public_point(scalar_mul_result, self.f);
            // composer.check_circuit_satisfied();
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
            a: BlsScalar::from(20u64),
            b: BlsScalar::from(5u64),
            c: BlsScalar::from(25u64),
            d: BlsScalar::from(100u64),
            e: JubJubScalar::from(2u64),
            f: JubJubAffine::from(dusk_jubjub::GENERATOR_EXTENDED * JubJubScalar::from(2u64)),
        };
        circuit.gen_proof(&pp, &pk, b"Test")
    }?;

    
    // Verifier POV
    let public_inputs: Vec<PublicInputValue> = vec![
        BlsScalar::from(25u64).into(),
        BlsScalar::from(100u64).into(),
        JubJubAffine::from(dusk_jubjub::GENERATOR_EXTENDED * JubJubScalar::from(2u64)).into(),
    ];
    circuit::verify_proof(
        &pp,
        &vd.key(),
        &proof,
        &public_inputs,
        &vd.pi_pos(),
        b"Test",
    )
}
