//! auth_identity — SSO/OIDC, RBAC, tenancy management.
#![forbid(unsafe_code)]
#[derive(Debug, Clone)]
pub struct UserIdentity {
    pub id: String,
    pub email: String,
    pub roles: Vec<String>,
}
impl UserIdentity {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn user_role_check() {
        let user = UserIdentity {
            id: "u1".into(),
            email: "a@b.com".into(),
            roles: vec!["admin".into()],
        };
        assert!(user.has_role("admin"));
        assert!(!user.has_role("editor"));
    }
}
