//! cloud_project_service — project management, model versioning API.

use auth_identity::UserIdentity;
use std::collections::HashMap;

/// A model version within a project.
#[derive(Debug, Clone)]
pub struct ModelVersion {
    pub id: u64,
    pub label: String,
    pub timestamp: u64,
    pub entity_count: usize,
    pub description: String,
}

/// A cloud project with versioned models.
#[derive(Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub versions: Vec<ModelVersion>,
    pub collaborators: Vec<String>,
}

impl Project {
    pub fn add_version(&mut self, label: &str, entity_count: usize, desc: &str) {
        let id = self.versions.len() as u64 + 1;
        self.versions.push(ModelVersion {
            id,
            label: label.into(),
            timestamp: 0,
            entity_count,
            description: desc.into(),
        });
    }
    pub fn latest_version(&self) -> Option<&ModelVersion> {
        self.versions.last()
    }
}

/// Project service with access control.
#[derive(Debug, Default)]
pub struct ProjectService {
    projects: Vec<Project>,
    access: HashMap<String, Vec<String>>,
}

impl ProjectService {
    pub fn new() -> Self {
        Self {
            projects: vec![],
            access: HashMap::new(),
        }
    }

    pub fn create(&mut self, name: &str, owner: &UserIdentity) -> String {
        let id = format!("p{}", self.projects.len() + 1);
        self.projects.push(Project {
            id: id.clone(),
            name: name.into(),
            owner: owner.id.clone(),
            versions: vec![],
            collaborators: vec![owner.id.clone()],
        });
        self.access.insert(id.clone(), vec![owner.id.clone()]);
        id
    }

    pub fn get(&self, id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Project> {
        self.projects.iter_mut().find(|p| p.id == id)
    }

    pub fn can_access(&self, project_id: &str, user: &UserIdentity) -> bool {
        self.access
            .get(project_id)
            .map(|users| users.iter().any(|u| u == &user.id))
            .unwrap_or(false)
    }

    pub fn add_collaborator(&mut self, project_id: &str, user_id: &str) {
        if let Some(users) = self.access.get_mut(project_id)
            && !users.contains(&user_id.to_string())
        {
            users.push(user_id.into());
        }
        if let Some(proj) = self.projects.iter_mut().find(|p| p.id == project_id)
            && !proj.collaborators.contains(&user_id.to_string())
        {
            proj.collaborators.push(user_id.into());
        }
    }

    pub fn by_owner(&self, owner_id: &str) -> Vec<&Project> {
        self.projects
            .iter()
            .filter(|p| p.owner == owner_id)
            .collect()
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
    fn admin() -> UserIdentity {
        UserIdentity {
            id: "u1".into(),
            email: "a@b.com".into(),
            roles: vec!["admin".into()],
        }
    }
    fn guest() -> UserIdentity {
        UserIdentity {
            id: "u2".into(),
            email: "c@d.com".into(),
            roles: vec!["viewer".into()],
        }
    }

    #[test]
    fn create_and_access() {
        let mut svc = ProjectService::new();
        let pid = svc.create("Project A", &admin());
        assert_eq!(svc.len(), 1);
        assert!(svc.can_access(&pid, &admin()));
    }

    #[test]
    fn project_versioning() {
        let mut proj = Project {
            id: "p1".into(),
            name: "Test".into(),
            owner: "u1".into(),
            versions: vec![],
            collaborators: vec![],
        };
        proj.add_version("v0.1", 100, "Initial");
        proj.add_version("v0.2", 120, "Updated");
        assert_eq!(proj.versions.len(), 2);
        assert_eq!(proj.latest_version().unwrap().label, "v0.2");
    }

    #[test]
    fn access_control() {
        let mut svc = ProjectService::new();
        let pid = svc.create("Secret", &admin());
        assert!(svc.can_access(&pid, &admin()));
        assert!(!svc.can_access(&pid, &guest()));
        svc.add_collaborator(&pid, "u2");
        assert!(svc.can_access(&pid, &guest()));
    }
}
