//! collaboration_service — comments, review states, shared workspaces.
#![forbid(unsafe_code)]
#[derive(Debug, Clone)]
pub struct Comment {
    pub id: u64,
    pub author_id: String,
    pub text: String,
    pub resolved: bool,
}
#[derive(Debug, Default)]
pub struct CollaborationSession {
    comments: Vec<Comment>,
}
impl CollaborationSession {
    pub fn new() -> Self {
        Self { comments: vec![] }
    }
    pub fn add_comment(&mut self, c: Comment) {
        self.comments.push(c);
    }
    pub fn comment_count(&self) -> usize {
        self.comments.len()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_comment() {
        let mut s = CollaborationSession::new();
        s.add_comment(Comment {
            id: 1,
            author_id: "u1".into(),
            text: "review".into(),
            resolved: false,
        });
        assert_eq!(s.comment_count(), 1);
    }
}
