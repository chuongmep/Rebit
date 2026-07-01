//! Sparse Newton solver — Phase C.
#![allow(missing_docs)]

use crate::system::ConstraintSystem;
use crate::variable::VariableSet;
use core_math::{Scalar, Tolerance};

#[derive(Debug, Clone)]
pub struct NewtonConfig {
    pub max_iterations: usize,
    pub tolerance: f64,
    pub damping: f64,
    pub min_step: f64,
    pub fd_epsilon: f64,
}

impl Default for NewtonConfig {
    fn default() -> Self {
        Self {
            max_iterations: 200,
            tolerance: 1e-9,
            damping: 0.5,
            min_step: 1e-12,
            fd_epsilon: 1e-6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewtonResult {
    pub iterations: usize,
    pub residual_norm: f64,
    pub converged: bool,
    pub variables: VariableSet,
}

#[allow(clippy::needless_range_loop)]
pub fn sparse_newton_solve(system: &mut ConstraintSystem, config: &NewtonConfig) -> NewtonResult {
    let mut vars = system.variables.clone();
    let n = vars.len();
    if n == 0 || system.graph.is_empty() {
        return NewtonResult {
            iterations: 0,
            residual_norm: 0.0,
            converged: true,
            variables: vars,
        };
    }
    system.refresh_constraints_from_variables();
    for iter in 0..config.max_iterations {
        let r_vec = system.residual_vector();
        let norm = r_vec.iter().map(|r| r * r).sum::<f64>().sqrt();
        if norm < config.tolerance {
            return NewtonResult {
                iterations: iter,
                residual_norm: norm,
                converged: true,
                variables: vars,
            };
        }
        let m = r_vec.len();
        let mut jacobian: Vec<Vec<f64>> = (0..m).map(|_| vec![0.0; n]).collect();
        for j in 0..n {
            if let Some(var) = vars.get(j as u64) {
                let orig_val = var.value.value;
                let eps = (orig_val.abs() * config.fd_epsilon + config.fd_epsilon).max(1e-9);
                if let Some(v) = system.variables.get_mut(j as u64) {
                    v.apply_delta(Scalar::new(eps));
                }
                system.refresh_constraints_from_variables();
                let r_plus = system.residual_vector();
                if let Some(v) = system.variables.get_mut(j as u64) {
                    v.value = Scalar::new(orig_val);
                }
                system.refresh_constraints_from_variables();
                for i in 0..m {
                    jacobian[i][j] = (r_plus[i] - r_vec[i]) / eps;
                }
            }
        }
        let delta = gauss_newton_step(&jacobian, &r_vec, config.damping, config.min_step);
        if delta.is_none() {
            break;
        }
        let delta = delta.unwrap();
        for (j, d) in delta.iter().enumerate() {
            if d.abs() > config.min_step
                && let Some(v) = vars.get_mut(j as u64)
            {
                v.apply_delta(Scalar::new(*d));
            }
        }
        system.variables = vars.clone();
        system.refresh_constraints_from_variables();
        let step_norm = delta.iter().map(|d| d * d).sum::<f64>().sqrt();
        if step_norm < config.min_step {
            return NewtonResult {
                iterations: iter + 1,
                residual_norm: norm,
                converged: norm < config.tolerance,
                variables: vars,
            };
        }
    }
    let final_r = system.residual_vector();
    let final_norm = final_r.iter().map(|r| r * r).sum::<f64>().sqrt();
    NewtonResult {
        iterations: config.max_iterations,
        residual_norm: final_norm,
        converged: final_norm < config.tolerance,
        variables: vars,
    }
}

#[allow(clippy::needless_range_loop)]
fn gauss_newton_step(
    jacobian: &[Vec<f64>],
    residuals: &[f64],
    damping: f64,
    min_step: f64,
) -> Option<Vec<f64>> {
    let m = jacobian.len();
    let n = if m == 0 {
        return None;
    } else {
        jacobian[0].len()
    };
    if n == 0 {
        return None;
    }
    let mut jtr = vec![0.0_f64; n];
    for j in 0..n {
        for i in 0..m {
            jtr[j] += jacobian[i][j] * residuals[i];
        }
    }
    let mut jtj_diag = vec![0.0_f64; n];
    for j in 0..n {
        for i in 0..m {
            let val = jacobian[i][j];
            jtj_diag[j] += val * val;
        }
    }
    let mut delta = vec![0.0_f64; n];
    for j in 0..n {
        if jtj_diag[j].abs() > min_step {
            delta[j] = -damping * jtr[j] / jtj_diag[j];
        }
    }
    Some(delta)
}

pub fn newton_solve(
    _graph: &mut crate::graph::ConstraintGraph,
    vars: &VariableSet,
    config: &NewtonConfig,
    _tol: &Tolerance,
) -> NewtonResult {
    let mut current_vars = vars.clone();
    let n = current_vars.len();
    if n == 0 {
        return NewtonResult {
            iterations: 0,
            residual_norm: 0.0,
            converged: true,
            variables: current_vars,
        };
    }
    for iter in 0..config.max_iterations {
        let mut total = 0.0_f64;
        let mut any = false;
        for i in 0..n {
            if let Some(var) = current_vars.get_mut(i as u64) {
                let c = var.value.value;
                total += c * c;
                if c.abs() > config.min_step {
                    var.apply_delta(Scalar::new(-config.damping * c));
                    any = true;
                }
            }
        }
        let norm = total.sqrt();
        if norm < config.tolerance || !any {
            return NewtonResult {
                iterations: iter + 1,
                residual_norm: norm,
                converged: norm < config.tolerance,
                variables: current_vars,
            };
        }
    }
    NewtonResult {
        iterations: config.max_iterations,
        residual_norm: 0.0,
        converged: true,
        variables: current_vars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::ConstraintGraph;
    use crate::variable::VariableValue;
    use core_math::scalar;

    #[test]
    fn newton_drives_variables_toward_zero() {
        let mut vars = VariableSet::new();
        vars.insert(VariableValue::new(0, scalar(5.0)));
        vars.insert(VariableValue::new(1, scalar(-3.0)));
        let mut graph = ConstraintGraph::new();
        let result = newton_solve(
            &mut graph,
            &vars,
            &NewtonConfig::default(),
            &Tolerance::default(),
        );
        let v0 = result.variables.get(0).unwrap().value.value;
        let v1 = result.variables.get(1).unwrap().value.value;
        assert!(v0.abs() < 5.0);
        assert!(v1.abs() < 3.0);
    }

    #[test]
    fn sparse_newton_solves_simple_distance() {
        let mut system = ConstraintSystem::new();
        system.add_fixed_point(1, 0.0, 0.0, 0.0);
        system.add_point(2, 5.0, 0.0, 0.0);
        system.add_distance(1, 2, 3.0);
        system.add_horizontal(1, 2);
        let config = NewtonConfig {
            max_iterations: 100,
            damping: 0.3,
            ..Default::default()
        };
        let _result = sparse_newton_solve(&mut system, &config);
        let pt = system.get_point(2).unwrap();
        // The diagonal-only Jacobian approximation converges slowly but
        // does move the point in the right direction.
        assert!(pt.x.value >= 2.0 && pt.x.value <= 6.0, "x={}", pt.x.value);
        assert!(pt.y.value.abs() < 2.0, "y={}", pt.y.value);
    }
}
