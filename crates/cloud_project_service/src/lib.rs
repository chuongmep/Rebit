//! cloud_project_service — project management, model versioning API.
#![forbid(unsafe_code)]
use auth_identity::UserIdentity;
#[derive(Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub owner: String,
}
#[derive(Debug, Default)]
pub struct ProjectService {
    projects: Vec<Project>,
}
impl ProjectService {
    pub fn new() -> Self {
        Self { projects: vec![] }
    }
    pub fn create(&mut self, name: &str, owner: &UserIdentity) -> &Project {
        let proj = Project {
            id: format!("p{}", self.projects.len() + 1),
            name: name.into(),
            owner: owner.id.clone(),
        };
        self.projects.push(proj);
        self.projects.last().unwrap()
    }
    pub fn len(&self) -> usize {
        self.projects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.projects.is_empty()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_project() {
        let user = UserIdentity {
            id: "u1".into(),
            email: "a@b.com".into(),
            roles: vec![],
        };
        let mut svc = ProjectService::new();
        svc.create("Project A", &user);
        assert_eq!(svc.len(), 1);
        assert!(!svc.is_empty());
    }
}
