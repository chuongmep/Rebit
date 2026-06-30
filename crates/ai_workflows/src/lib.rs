//! ai_workflows — AI-assisted modeling commands and rule checks.
#![forbid(unsafe_code)]
use bim_model::entity::EntityGraph;

#[derive(Debug, Clone)]
pub struct AiSuggestion {
    pub description: String,
    pub confidence: f64,
}

#[derive(Debug, Default)]
pub struct AiWorkflowEngine;
impl AiWorkflowEngine {
    pub fn new() -> Self {
        Self
    }
    pub fn analyze(&self, _graph: &EntityGraph) -> Vec<AiSuggestion> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn engine_returns_empty() {
        assert!(
            AiWorkflowEngine::new()
                .analyze(&EntityGraph::new())
                .is_empty()
        );
    }
}
