// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::ProverKey;
use crate::util::*;
use ark_ec::{PairingEngine, TEModelParameters};
use ark_ff::PrimeField;
use ark_poly::{
    univariate::DensePolynomial, GeneralEvaluationDomain, Polynomial,
};

#[allow(dead_code)]
/// Evaluations at points `z` or and `z * root of unity`
pub(crate) struct Evaluations<F: PrimeField> {
    pub(crate) proof: ProofEvaluations<F>,
    // Evaluation of the linearisation sigma polynomial at `z`
    pub(crate) quot_eval: F,
}

/// Subset of all of the evaluations. These evaluations
/// are added to the [`Proof`](super::Proof).
#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub(crate) struct ProofEvaluations<F: PrimeField> {
    // Evaluation of the witness polynomial for the left wire at `z`
    pub(crate) a_eval: F,
    // Evaluation of the witness polynomial for the right wire at `z`
    pub(crate) b_eval: F,
    // Evaluation of the witness polynomial for the output wire at `z`
    pub(crate) c_eval: F,
    // Evaluation of the witness polynomial for the fourth wire at `z`
    pub(crate) d_eval: F,
    //
    pub(crate) a_next_eval: F,
    //
    pub(crate) b_next_eval: F,
    // Evaluation of the witness polynomial for the fourth wire at `z * root of
    // unity`
    pub(crate) d_next_eval: F,
    // Evaluation of the arithmetic selector polynomial at `z`
    pub(crate) q_arith_eval: F,
    //
    pub(crate) q_c_eval: F,
    //
    pub(crate) q_l_eval: F,
    //
    pub(crate) q_r_eval: F,
    // Evaluation of the left sigma polynomial at `z`
    pub(crate) left_sigma_eval: F,
    // Evaluation of the right sigma polynomial at `z`
    pub(crate) right_sigma_eval: F,
    // Evaluation of the out sigma polynomial at `z`
    pub(crate) out_sigma_eval: F,

    // Evaluation of the linearisation sigma polynomial at `z`
    pub(crate) lin_poly_eval: F,

    // (Shifted) Evaluation of the permutation polynomial at `z * root of
    // unity`
    pub(crate) perm_eval: F,
}

/// Compute the linearisation polynomial.
pub(crate) fn compute<
    E: PairingEngine,
    P: TEModelParameters<BaseField = E::Fr>,
>(
    domain: &GeneralEvaluationDomain<E::Fr>,
    prover_key: &ProverKey<E::Fr, P>,
    (
        alpha,
        beta,
        gamma,
        range_separation_challenge,
        logic_separation_challenge,
        fixed_base_separation_challenge,
        var_base_separation_challenge,
        z_challenge,
    ): &(E::Fr, E::Fr, E::Fr, E::Fr, E::Fr, E::Fr, E::Fr, E::Fr),
    w_l_poly: &DensePolynomial<E::Fr>,
    w_r_poly: &DensePolynomial<E::Fr>,
    w_o_poly: &DensePolynomial<E::Fr>,
    w_4_poly: &DensePolynomial<E::Fr>,
    t_x_poly: &DensePolynomial<E::Fr>,
    z_poly: &DensePolynomial<E::Fr>,
) -> (DensePolynomial<E::Fr>, Evaluations<E::Fr>) {
    // Compute evaluations
    let quot_eval = t_x_poly.evaluate(z_challenge);
    let a_eval = w_l_poly.evaluate(z_challenge);
    let b_eval = w_r_poly.evaluate(z_challenge);
    let c_eval = w_o_poly.evaluate(z_challenge);
    let d_eval = w_4_poly.evaluate(z_challenge);
    let left_sigma_eval =
        prover_key.permutation.left_sigma.0.evaluate(z_challenge);
    let right_sigma_eval =
        prover_key.permutation.right_sigma.0.evaluate(z_challenge);
    let out_sigma_eval =
        prover_key.permutation.out_sigma.0.evaluate(z_challenge);
    let q_arith_eval = prover_key.arithmetic.q_arith.0.evaluate(z_challenge);
    let q_c_eval = prover_key.logic.q_c.0.evaluate(z_challenge);
    let q_l_eval = prover_key.fixed_base.q_l.0.evaluate(z_challenge);
    let q_r_eval = prover_key.fixed_base.q_r.0.evaluate(z_challenge);

    let group_gen = get_domain_attrs(domain, "group_gen");
    let a_next_eval = w_l_poly.evaluate(&(*z_challenge * group_gen));
    let b_next_eval = w_r_poly.evaluate(&(*z_challenge * group_gen));
    let d_next_eval = w_4_poly.evaluate(&(*z_challenge * group_gen));
    let perm_eval = z_poly.evaluate(&(*z_challenge * group_gen));

    let f_1 = compute_circuit_satisfiability::<E, P>(
        (
            range_separation_challenge,
            logic_separation_challenge,
            fixed_base_separation_challenge,
            var_base_separation_challenge,
        ),
        a_eval,
        b_eval,
        c_eval,
        d_eval,
        a_next_eval,
        b_next_eval,
        d_next_eval,
        q_arith_eval,
        q_c_eval,
        q_l_eval,
        q_r_eval,
        prover_key,
    );

    let f_2 = prover_key.permutation.compute_linearisation(
        *z_challenge,
        (*alpha, *beta, *gamma),
        (a_eval, b_eval, c_eval, d_eval),
        (left_sigma_eval, right_sigma_eval, out_sigma_eval),
        perm_eval,
        z_poly,
    );

    let lin_poly = &f_1 + &f_2;

    // Evaluate linearisation polynomial at z_challenge
    let lin_poly_eval = lin_poly.evaluate(z_challenge);

    (
        lin_poly,
        Evaluations {
            proof: ProofEvaluations {
                a_eval,
                b_eval,
                c_eval,
                d_eval,
                a_next_eval,
                b_next_eval,
                d_next_eval,
                q_arith_eval,
                q_c_eval,
                q_l_eval,
                q_r_eval,
                left_sigma_eval,
                right_sigma_eval,
                out_sigma_eval,
                lin_poly_eval,
                perm_eval,
            },
            quot_eval,
        },
    )
}

fn compute_circuit_satisfiability<
    E: PairingEngine,
    P: TEModelParameters<BaseField = E::Fr>,
>(
    (
        range_separation_challenge,
        logic_separation_challenge,
        fixed_base_separation_challenge,
        var_base_separation_challenge,
    ): (&E::Fr, &E::Fr, &E::Fr, &E::Fr),
    a_eval: E::Fr,
    b_eval: E::Fr,
    c_eval: E::Fr,
    d_eval: E::Fr,
    a_next_eval: E::Fr,
    b_next_eval: E::Fr,
    d_next_eval: E::Fr,
    q_arith_eval: E::Fr,
    q_c_eval: E::Fr,
    q_l_eval: E::Fr,
    q_r_eval: E::Fr,
    prover_key: &ProverKey<E::Fr, P>,
) -> DensePolynomial<E::Fr> {
    let a = prover_key.arithmetic.compute_linearisation(
        a_eval,
        b_eval,
        c_eval,
        d_eval,
        q_arith_eval,
    );

    let b = prover_key.range.compute_linearisation(
        *range_separation_challenge,
        a_eval,
        b_eval,
        c_eval,
        d_eval,
        d_next_eval,
    );

    let c = prover_key.logic.compute_linearisation(
        *logic_separation_challenge,
        a_eval,
        a_next_eval,
        b_eval,
        b_next_eval,
        c_eval,
        d_eval,
        d_next_eval,
        q_c_eval,
    );

    let d = prover_key.fixed_base.compute_linearisation(
        *fixed_base_separation_challenge,
        a_eval,
        a_next_eval,
        b_eval,
        b_next_eval,
        c_eval,
        d_eval,
        d_next_eval,
        q_l_eval,
        q_r_eval,
        q_c_eval,
    );

    let e = prover_key.variable_base.compute_linearisation(
        *var_base_separation_challenge,
        a_eval,
        a_next_eval,
        b_eval,
        b_next_eval,
        c_eval,
        d_eval,
        d_next_eval,
    );

    let mut linearisation_poly = &a + &b;
    linearisation_poly += &c;
    linearisation_poly += &d;
    linearisation_poly += &e;

    linearisation_poly
}
/*
#[cfg(test)]
mod evaluations_tests {
    use super::*;

    #[test]
    fn proof_evaluations_dusk_bytes_serde() {
        let proof_evals = ProofEvaluations::default();
        let bytes = proof_evals.to_bytes();
        let obtained_evals = ProofEvaluations::from_slice(&bytes)
            .expect("Deserialization error");
        assert_eq!(proof_evals.to_bytes(), obtained_evals.to_bytes())
    }
}
*/
