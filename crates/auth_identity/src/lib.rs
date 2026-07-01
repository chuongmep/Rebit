//! auth_identity — SSO/OIDC, RBAC, and tenancy management.
//!
//! # Phase B additions
//! - Role hierarchy (admin > editor > viewer)
//! - Permission checks based on role
//! - Session management with tokens

#![forbid(unsafe_code)]

/// Pre-defined role levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// Read-only access.
    Viewer,
    /// Can create and modify content.
    Editor,
    /// Full administrative access.
    Admin,
}

/// A user identity with roles.
#[derive(Debug, Clone)]
pub struct UserIdentity {
    pub id: String,
    pub email: String,
    pub roles: Vec<String>,
}

impl UserIdentity {
    /// Check if the user has a specific role string.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Get the highest role level.
    pub fn highest_role(&self) -> Role {
        if self.roles.contains(&"admin".to_string()) {
            Role::Admin
        } else if self.roles.contains(&"editor".to_string()) {
            Role::Editor
        } else {
            Role::Viewer
        }
    }

    /// Check if the user can edit (editor or admin).
    pub fn can_edit(&self) -> bool {
        matches!(self.highest_role(), Role::Editor | Role::Admin)
    }

    /// Check if the user is an admin.
    pub fn is_admin(&self) -> bool {
        self.highest_role() == Role::Admin
    }
}

/// A session token for an authenticated user.
#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub expires_at: u64,
    pub refresh_token: Option<String>,
}

/// Simple in-memory session store.
#[derive(Debug, Default)]
pub struct SessionStore {
    sessions: std::collections::HashMap<String, Session>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
        }
    }

    /// Create a new session for a user.
    pub fn create_session(&mut self, user: &UserIdentity) -> &Session {
        let token = format!("tok_{}_{}", user.id, self.sessions.len());
        self.sessions.insert(
            token.clone(),
            Session {
                token: token.clone(),
                user_id: user.id.clone(),
                expires_at: 9999999999,
                refresh_token: None,
            },
        );
        self.sessions.get(&token).unwrap()
    }

    /// Look up a session by token.
    pub fn get(&self, token: &str) -> Option<&Session> {
        self.sessions.get(token)
    }

    /// Number of active sessions.
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_role_hierarchy() {
        let admin = UserIdentity {
            id: "u1".into(),
            email: "admin@b.com".into(),
            roles: vec!["admin".into()],
        };
        assert!(admin.is_admin());
        assert!(admin.can_edit());
        assert_eq!(admin.highest_role(), Role::Admin);
    }

    #[test]
    fn viewer_cannot_edit() {
        let viewer = UserIdentity {
            id: "u2".into(),
            email: "viewer@b.com".into(),
            roles: vec!["viewer".into()],
        };
        assert!(!viewer.can_edit());
        assert_eq!(viewer.highest_role(), Role::Viewer);
    }

    #[test]
    fn session_store_creates_token() {
        let mut store = SessionStore::new();
        let user = UserIdentity {
            id: "u1".into(),
            email: "a@b.com".into(),
            roles: vec!["editor".into()],
        };
        let session = store.create_session(&user);
        assert!(session.token.starts_with("tok_"));
        assert_eq!(store.len(), 1);
    }
}
