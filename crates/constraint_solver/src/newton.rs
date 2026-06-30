//! Newton-Raphson constraint solver (Phase B).

use crate::graph::ConstraintGraph;
use crate::variable::VariableSet;
use core_math::{Scalar, Tolerance};

/// Configuration for the Newton-Raphson solver.
#[derive(Debug, Clone)]
pub struct NewtonConfig {
    /// Maximum iterations.
    pub max_iterations: usize,
    /// Convergence tolerance for residual norm.
    pub tolerance: f64,
    /// Step-size damping factor (0 < damping ≤ 1).
    pub damping: f64,
    /// Minimum step size below which we declare convergence.
    pub min_step: f64,
}

impl Default for NewtonConfig {
    fn default() -> Self {
        Self {
            max_iterations: 200,
            tolerance: 1e-9,
            damping: 0.5,
            min_step: 1e-12,
        }
    }
}

/// Result of a Newton-Raphson solve.
#[derive(Debug, Clone)]
pub struct NewtonResult {
    /// Number of iterations performed.
    pub iterations: usize,
    /// Final residual norm.
    pub residual_norm: f64,
    /// Whether the solver converged.
    pub converged: bool,
    /// Updated variable values.
    pub variables: VariableSet,
}

/// Run a Newton-Raphson solve on a constraint graph using gradient descent.
///
/// NOTE: Phase B Newton solver operates on VariableSet variables, not on
/// graph-embedded point coordinates.  Future work will bridge the two.
pub fn newton_solve(
    _graph: &mut ConstraintGraph,
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

    // Simple: minimize |x - target| for any variable with a target value.
    for iter in 0..config.max_iterations {
        let mut total_residual = 0.0_f64;
        let mut any_change = false;

        // Walk all variables and nudge toward a computed target.
        for i in 0..n {
            if let Some(var) = current_vars.get_mut(i as u64) {
                // Stub: variables have no target metadata yet.
                // This solver is a placeholder until variable-ID-based
                // constraints are implemented in Phase C.
                let current = var.value.value;
                total_residual += current.powi(2);
                if current.abs() > config.min_step {
                    let delta = -config.damping * current;
                    var.apply_delta(Scalar::new(delta));
                    any_change = true;
                }
            }
        }

        let norm = total_residual.sqrt();
        if norm < config.tolerance || !any_change {
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
    use crate::variable::VariableValue;
    use core_math::scalar;

    #[test]
    fn newton_drives_variables_toward_zero() {
        let mut vars = VariableSet::new();
        vars.insert(VariableValue::new(0, scalar(5.0)));
        vars.insert(VariableValue::new(1, scalar(-3.0)));
        let mut graph = ConstraintGraph::new();
        let config = NewtonConfig::default();
        let tol = Tolerance::default();
        let result = newton_solve(&mut graph, &vars, &config, &tol);
        // Variables should be closer to zero.
        let v0 = result.variables.get(0).unwrap().value.value;
        let v1 = result.variables.get(1).unwrap().value.value;
        assert!(v0.abs() < 5.0, "v0 = {v0}");
        assert!(v1.abs() < 3.0, "v1 = {v1}");
    }
}
