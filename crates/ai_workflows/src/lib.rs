//! ai_workflows — AI-assisted modeling commands and rule checks.
//!
//! # Phase B additions
//! - Building-code rule check engine
//! - Analysis suggestions with confidence scores
//! - Entity count and relationship diagnostics

use bim_model::entity::{Entity, EntityCategory, EntityGraph};

/// An AI-generated modeling suggestion.
#[derive(Debug, Clone)]
pub struct AiSuggestion {
    pub description: String,
    pub confidence: f64,
    pub category: Option<EntityCategory>,
}

/// A building-code rule check.
#[derive(Debug, Clone)]
pub struct RuleCheck {
    pub id: u64,
    pub name: String,
    pub description: String,
    /// Category this rule applies to.
    pub applies_to: EntityCategory,
}

impl RuleCheck {
    /// Check if an entity satisfies this rule (stub — always passes in Phase B).
    pub fn check(&self, _entity: &Entity) -> RuleCheckResult {
        RuleCheckResult {
            rule_id: self.id,
            passed: true,
            message: format!("{}: passed", self.name),
        }
    }
}

/// Result of a rule check.
#[derive(Debug, Clone)]
pub struct RuleCheckResult {
    pub rule_id: u64,
    pub passed: bool,
    pub message: String,
}

/// AI workflow engine.
#[derive(Debug, Default)]
pub struct AiWorkflowEngine {
    rules: Vec<RuleCheck>,
}

impl AiWorkflowEngine {
    /// Create a new AI workflow engine with default rules.
    pub fn new() -> Self {
        let mut engine = Self { rules: vec![] };
        engine.register_default_rules();
        engine
    }

    /// Register a building-code rule.
    pub fn register_rule(&mut self, rule: RuleCheck) {
        self.rules.push(rule);
    }

    /// Run all applicable rules against an entity graph.
    pub fn analyze(&self, graph: &EntityGraph) -> (Vec<AiSuggestion>, Vec<RuleCheckResult>) {
        let mut suggestions = Vec::new();
        let mut results = Vec::new();

        for entity in graph.iter() {
            for rule in &self.rules {
                if rule.applies_to == entity.category {
                    results.push(rule.check(entity));
                }
            }
        }

        // Generate generic suggestions based on graph analysis.
        let wall_count = graph.by_category(EntityCategory::Wall).len();
        let door_count = graph.by_category(EntityCategory::Door).len();
        let window_count = graph.by_category(EntityCategory::Window).len();

        if wall_count > 0 && door_count == 0 && window_count == 0 {
            suggestions.push(AiSuggestion {
                description: "Consider adding doors and windows to the walls".into(),
                confidence: 0.85,
                category: Some(EntityCategory::Door),
            });
        }

        if wall_count > 0 && door_count > 0 && window_count == 0 {
            suggestions.push(AiSuggestion {
                description: "Consider adding windows for natural lighting".into(),
                confidence: 0.7,
                category: Some(EntityCategory::Window),
            });
        }

        (suggestions, results)
    }

    /// Run rule checks only (no suggestions).
    pub fn run_checks(&self, graph: &EntityGraph) -> Vec<RuleCheckResult> {
        let mut results = Vec::new();
        for entity in graph.iter() {
            for rule in &self.rules {
                if rule.applies_to == entity.category {
                    results.push(rule.check(entity));
                }
            }
        }
        results
    }

    /// Count registered rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    fn register_default_rules(&mut self) {
        self.register_rule(RuleCheck {
            id: 1,
            name: "Wall thickness check".into(),
            description: "Walls must be at least 0.1m thick".into(),
            applies_to: EntityCategory::Wall,
        });
        self.register_rule(RuleCheck {
            id: 2,
            name: "Door clearance check".into(),
            description: "Doors must have at least 0.8m clearance".into(),
            applies_to: EntityCategory::Door,
        });
        self.register_rule(RuleCheck {
            id: 3,
            name: "Window sill height".into(),
            description: "Windows must be placed at appropriate sill height".into(),
            applies_to: EntityCategory::Window,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry_kernel::Point3D;
    use geometry_kernel::topology::{Edge, Face, Loop, Shell, Solid, Vertex};

    fn make_box_solid(id: u64, w: f64, d: f64, h: f64) -> Solid {
        let v = [
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(w, 0.0, 0.0),
            Point3D::new(w, d, 0.0),
            Point3D::new(0.0, d, 0.0),
            Point3D::new(0.0, 0.0, h),
            Point3D::new(w, 0.0, h),
            Point3D::new(w, d, h),
            Point3D::new(0.0, d, h),
        ];
        let verts: Vec<Vertex> = v
            .iter()
            .enumerate()
            .map(|(i, p)| Vertex::new(i as u64, *p))
            .collect();
        fn quad(v0: &Vertex, v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Loop {
            Loop::new(
                0,
                vec![
                    Edge::new(0, v0.clone(), v1.clone()),
                    Edge::new(1, v1.clone(), v2.clone()),
                    Edge::new(2, v2.clone(), v3.clone()),
                    Edge::new(3, v3.clone(), v0.clone()),
                ],
            )
        }
        let faces = vec![
            Face::new(0, quad(&verts[0], &verts[1], &verts[2], &verts[3]), vec![]),
            Face::new(1, quad(&verts[4], &verts[5], &verts[6], &verts[7]), vec![]),
            Face::new(2, quad(&verts[0], &verts[1], &verts[5], &verts[4]), vec![]),
            Face::new(3, quad(&verts[2], &verts[3], &verts[7], &verts[6]), vec![]),
            Face::new(4, quad(&verts[0], &verts[3], &verts[7], &verts[4]), vec![]),
            Face::new(5, quad(&verts[1], &verts[2], &verts[6], &verts[5]), vec![]),
        ];
        Solid::new(id, Shell::new(0, faces))
    }

    #[test]
    fn engine_returns_empty_for_empty_graph() {
        let engine = AiWorkflowEngine::new();
        let graph = EntityGraph::new();
        let (suggestions, results) = engine.analyze(&graph);
        assert!(suggestions.is_empty());
        assert!(results.is_empty());
    }

    #[test]
    fn engine_suggests_doors_for_walls_only() {
        let mut graph = EntityGraph::new();
        graph.create(EntityCategory::Wall, "W1", make_box_solid(0, 5.0, 0.3, 3.0));
        let engine = AiWorkflowEngine::new();
        let (suggestions, _) = engine.analyze(&graph);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.description.contains("doors")));
    }

    #[test]
    fn rule_checks_are_executed() {
        let mut graph = EntityGraph::new();
        graph.create(EntityCategory::Wall, "W1", make_box_solid(0, 5.0, 0.3, 3.0));
        graph.create(EntityCategory::Door, "D1", make_box_solid(1, 1.0, 0.1, 2.1));
        let engine = AiWorkflowEngine::new();
        let results = engine.run_checks(&graph);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn default_rules_registered() {
        let engine = AiWorkflowEngine::new();
        assert_eq!(engine.rule_count(), 3);
    }
}
