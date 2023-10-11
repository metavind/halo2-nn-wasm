use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use std::sync::Arc;

use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use halo2_base::gates::flex_gate::{GateChip, GateInstructions};
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use halo2_base::AssignedValue;
use halo2_base::QuantumCell::Constant;

use halo2_wasm::Halo2Wasm;

mod nn_params;

#[wasm_bindgen]
pub struct NnWasm {
    gate: GateChip<Fr>,
    builder: Arc<RefCell<BaseCircuitBuilder<Fr>>>,
}

#[wasm_bindgen]
impl NnWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(circuit: &Halo2Wasm) -> Self {
        let gate = GateChip::new();
        let lookup_bits = circuit.circuit_params.clone().unwrap().lookup_bits.unwrap();
        NnWasm {
            gate,
            builder: Arc::clone(&circuit.circuit),
        }
    }

    pub fn forward(&mut self, circuit: &mut Halo2Wasm, input: &[u32]) -> String {
        let input = self
            .builder
            .borrow_mut()
            .main(0)
            .assign_witnesses(input.iter().map(|&x| Fr::from(x as u64)));

        let w1 = nn_params::W1
            .into_iter()
            .map(Fr::from_raw)
            .map(Constant)
            .collect::<Vec<_>>();

        let b1 = nn_params::B1
            .into_iter()
            .map(Fr::from_raw)
            .map(Constant)
            .collect::<Vec<_>>();

        let mut l1: Vec<AssignedValue<Fr>> = Vec::new();
        for i in 0..(w1.len() / input.len()) {
            let mut sum = self.builder.borrow_mut().main(0).load_constant(Fr::from(0));
            for j in 0..input.len() {
                let temp = self.gate.mul(
                    self.builder.borrow_mut().main(0),
                    w1[i * input.len() + j],
                    input[j],
                );
                sum = self.gate.add(self.builder.borrow_mut().main(0), sum, temp);
            }
            sum = self.gate.add(self.builder.borrow_mut().main(0), sum, b1[i]);
            l1.push(sum);
        }

        let alpha = self
            .builder
            .borrow_mut()
            .main(0)
            .load_constant(Fr::from(10u64.pow(12)));

        let mut o1 = Vec::new();

        for i in 0..l1.len() {
            let alpha_x = self
                .gate
                .mul(self.builder.borrow_mut().main(0), alpha, l1[i]);
            let x_sq = self
                .gate
                .mul(self.builder.borrow_mut().main(0), l1[i], l1[i]);

            let temp = self
                .gate
                .add(self.builder.borrow_mut().main(0), alpha_x, x_sq);
            o1.push(temp);
        }

        let w2 = nn_params::W2
            .into_iter()
            .map(Fr::from_raw)
            .map(Constant)
            .collect::<Vec<_>>();

        // let b2 = B2
        //     .into_iter()
        //     .map(Fr::from_raw)
        //     .map(Constant)
        //     .collect::<Vec<_>>();

        let mut l2: Vec<AssignedValue<Fr>> = Vec::new();
        for i in 0..(w2.len() / o1.len()) {
            let mut sum = self.builder.borrow_mut().main(0).load_constant(Fr::from(0));
            for j in 0..o1.len() {
                let temp = self.gate.mul(
                    self.builder.borrow_mut().main(0),
                    w2[i * o1.len() + j],
                    o1[j],
                );
                sum = self.gate.add(self.builder.borrow_mut().main(0), sum, temp);
            }
            //sum = self.gate.add(self.builder.borrow_mut().main(0), sum, b2[i]);
            l2.push(sum);
        }

        for elem in &l2 {
            circuit.public.get_mut(0).unwrap().push(*elem);
        }

        // l2.iter().map(|x| format!("{:?}", x.value())).collect()
        l2.iter()
            .fold(String::new(), |acc, x| acc + &format!("{:?}\n", x.value()))
    }
}
