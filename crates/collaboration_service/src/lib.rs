//! collaboration_service — comments, review states, shared workspaces.
//!
//! # Phase B additions
//! - Threaded comments with parent/child relationships
//! - Review states (draft, in_review, approved, rejected)
//! - Comment statistics per model version

/// A comment thread on a model.
#[derive(Debug, Clone)]
pub struct Comment {
    pub id: u64,
    pub author_id: String,
    pub text: String,
    pub resolved: bool,
    pub parent_id: Option<u64>,
    pub created_at: u64,
}

/// Review state for a model version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReviewState {
    #[default]
    Draft,
    InReview,
    Approved,
    Rejected,
}

impl ReviewState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::InReview => "in_review",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

/// Collaboration session for a model version.
#[derive(Debug, Default)]
pub struct CollaborationSession {
    comments: Vec<Comment>,
    next_id: u64,
    review_state: ReviewState,
}

impl CollaborationSession {
    pub fn new() -> Self {
        Self {
            comments: vec![],
            next_id: 1,
            review_state: ReviewState::Draft,
        }
    }

    /// Add a top-level comment.
    pub fn add_comment(&mut self, author_id: &str, text: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.comments.push(Comment {
            id,
            author_id: author_id.into(),
            text: text.into(),
            resolved: false,
            parent_id: None,
            created_at: 0,
        });
        id
    }

    /// Add a reply to an existing comment.
    pub fn reply_to(&mut self, parent_id: u64, author_id: &str, text: &str) -> Option<u64> {
        if !self.comments.iter().any(|c| c.id == parent_id) {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.comments.push(Comment {
            id,
            author_id: author_id.into(),
            text: text.into(),
            resolved: false,
            parent_id: Some(parent_id),
            created_at: 0,
        });
        Some(id)
    }

    /// Resolve a comment thread (marks top-level comment and all replies resolved).
    pub fn resolve_thread(&mut self, root_id: u64) {
        for c in &mut self.comments {
            if c.id == root_id || c.parent_id == Some(root_id) {
                c.resolved = true;
            }
        }
    }

    /// Set the review state.
    pub fn set_review_state(&mut self, state: ReviewState) {
        self.review_state = state;
    }

    /// Get the current review state.
    pub fn review_state(&self) -> ReviewState {
        self.review_state
    }

    /// Total comment count.
    pub fn comment_count(&self) -> usize {
        self.comments.len()
    }

    /// Unresolved comment count.
    pub fn unresolved_count(&self) -> usize {
        self.comments.iter().filter(|c| !c.resolved).count()
    }

    /// Get all top-level comments (no parent).
    pub fn top_level_comments(&self) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|c| c.parent_id.is_none())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_comment() {
        let mut session = CollaborationSession::new();
        session.add_comment("u1", "Great work");
        assert_eq!(session.comment_count(), 1);
    }

    #[test]
    fn threaded_reply() {
        let mut session = CollaborationSession::new();
        let root = session.add_comment("u1", "Check this area");
        let _reply = session.reply_to(root, "u2", "Fixed in v2").unwrap();
        assert_eq!(session.comment_count(), 2);
        assert_eq!(session.unresolved_count(), 2);
        session.resolve_thread(root);
        assert_eq!(session.unresolved_count(), 0);
    }

    #[test]
    fn review_states() {
        let mut session = CollaborationSession::new();
        assert_eq!(session.review_state(), ReviewState::Draft);
        session.set_review_state(ReviewState::InReview);
        assert_eq!(session.review_state(), ReviewState::InReview);
    }
}
