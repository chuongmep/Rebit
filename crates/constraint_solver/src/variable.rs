//! Constraint variables — degrees of freedom in the constraint graph.
//!
//! Each variable holds a scalar value and an optional domain (lower/upper
//! bounds). The solver modifies values to satisfy constraints while
//! respecting bounds.

use core_math::Scalar;

// ---------------------------------------------------------------------------
// VariableValue
// ---------------------------------------------------------------------------

/// A single degree of freedom with optional bounds.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableValue {
    /// Unique identifier.
    pub id: u64,
    /// Current value.
    pub value: Scalar,
    /// Optional lower bound (inclusive).
    pub lower: Option<Scalar>,
    /// Optional upper bound (inclusive).
    pub upper: Option<Scalar>,
}

impl VariableValue {
    /// Create an unbounded variable.
    #[inline]
    pub fn new(id: u64, value: Scalar) -> Self {
        Self {
            id,
            value,
            lower: None,
            upper: None,
        }
    }

    /// Create a variable with lower and upper bounds.
    #[inline]
    pub fn bounded(id: u64, value: Scalar, lower: Scalar, upper: Scalar) -> Self {
        debug_assert!(lower.value <= upper.value, "inverted bounds");
        Self {
            id,
            value,
            lower: Some(lower),
            upper: Some(upper),
        }
    }

    /// Clamp the value to `[lower, upper]` if bounds are set.
    #[inline]
    pub fn clamp(&mut self) {
        if let Some(lo) = self.lower
            && self.value.value < lo.value
        {
            self.value = lo;
        }
        if let Some(hi) = self.upper
            && self.value.value > hi.value
        {
            self.value = hi;
        }
    }

    /// Adjust the value by `delta`, then clamp.
    #[inline]
    pub fn apply_delta(&mut self, delta: Scalar) {
        self.value = self.value + delta;
        self.clamp();
    }

    /// `true` when the value is at or beyond its lower bound.
    #[inline]
    pub fn at_lower(&self) -> bool {
        self.lower.is_some_and(|lo| self.value.value <= lo.value)
    }
    /// `true` when the value is at or beyond its upper bound.
    #[inline]
    pub fn at_upper(&self) -> bool {
        self.upper.is_some_and(|hi| self.value.value >= hi.value)
    }
}

// ---------------------------------------------------------------------------
// VariableSet
// ---------------------------------------------------------------------------

/// A collection of variables indexed by id, supporting fast lookup and
/// id-ordered iteration (deterministic traversal).
#[derive(Debug, Clone)]
pub struct VariableSet {
    vars: Vec<VariableValue>,
}

impl VariableSet {
    /// Create an empty variable set.
    pub fn new() -> Self {
        Self { vars: Vec::new() }
    }

    /// Insert a variable (id must not already exist).
    pub fn insert(&mut self, var: VariableValue) {
        // Keep vec sorted by id for deterministic traversal.
        let pos = self
            .vars
            .binary_search_by(|v| v.id.cmp(&var.id))
            .expect_err("duplicate variable id");
        self.vars.insert(pos, var);
    }

    /// Get a variable by id.
    #[inline]
    pub fn get(&self, id: u64) -> Option<&VariableValue> {
        self.vars
            .binary_search_by(|v| v.id.cmp(&id))
            .ok()
            .map(|i| &self.vars[i])
    }

    /// Get mutable access by id.
    #[inline]
    pub fn get_mut(&mut self, id: u64) -> Option<&mut VariableValue> {
        self.vars
            .binary_search_by(|v| v.id.cmp(&id))
            .ok()
            .map(|i| &mut self.vars[i])
    }

    /// Number of variables.
    #[inline]
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    /// `true` when empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }

    /// Iterate over variables in id order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &VariableValue> {
        self.vars.iter()
    }

    /// Iterate over variables mutably in id order.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut VariableValue> {
        self.vars.iter_mut()
    }
}

impl Default for VariableSet {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::scalar;

    #[test]
    fn variable_clamp_respects_bounds() {
        let mut v = VariableValue::bounded(0, scalar(5.0), scalar(0.0), scalar(10.0));
        v.apply_delta(scalar(-10.0));
        assert!((v.value.value - 0.0).abs() < 1e-9);
        v.apply_delta(scalar(20.0));
        assert!((v.value.value - 10.0).abs() < 1e-9);
    }

    #[test]
    fn variable_unbounded_no_clamp() {
        let mut v = VariableValue::new(0, scalar(42.0));
        v.apply_delta(scalar(-100.0));
        assert!((v.value.value - (-58.0)).abs() < 1e-9);
    }

    #[test]
    fn variable_set_sorted_insertion() {
        let mut set = VariableSet::new();
        set.insert(VariableValue::new(10, scalar(0.0)));
        set.insert(VariableValue::new(2, scalar(1.0)));
        set.insert(VariableValue::new(5, scalar(2.0)));
        let ids: Vec<u64> = set.iter().map(|v| v.id).collect();
        assert_eq!(ids, vec![2, 5, 10]);
    }

    #[test]
    #[should_panic(expected = "duplicate variable id")]
    fn variable_set_duplicate_rejected() {
        let mut set = VariableSet::new();
        set.insert(VariableValue::new(1, scalar(0.0)));
        set.insert(VariableValue::new(1, scalar(1.0)));
    }
}
