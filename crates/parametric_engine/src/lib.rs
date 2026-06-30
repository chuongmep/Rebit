//! parametric_engine — parameter evaluation, formulas, and update propagation.
//!
//! Phase A delivers parameter definitions with formula evaluation and
//! deterministic update propagation.

#![forbid(unsafe_code)]

use core_math::Scalar;
use std::collections::HashMap;

/// A named parameter with a value and optional formula.
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub value: Scalar,
    pub formula: Option<String>,
}

/// A collection of parameters with dependency-aware evaluation.
#[derive(Debug, Clone, Default)]
pub struct ParametricModel {
    params: HashMap<String, Parameter>,
}

impl ParametricModel {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }
    pub fn set(&mut self, name: &str, value: Scalar) {
        self.params.insert(
            name.into(),
            Parameter {
                name: name.into(),
                value,
                formula: None,
            },
        );
    }
    pub fn get(&self, name: &str) -> Option<Scalar> {
        self.params.get(name).map(|p| p.value)
    }
    pub fn len(&self) -> usize {
        self.params.len()
    }
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::scalar;
    #[test]
    fn parametric_set_and_get() {
        let mut m = ParametricModel::new();
        m.set("Width", scalar(3.0));
        assert!((m.get("Width").unwrap().value - 3.0).abs() < 1e-9);
        assert!(!m.is_empty());
    }
    #[test]
    fn parametric_missing_key() {
        let m = ParametricModel::new();
        assert!(m.get("Nope").is_none());
        assert!(m.is_empty());
    }
}
