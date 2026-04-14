//! Hard delegation rules — structural constraints, not behavioral guidelines.
//! Principle 5 — Hierarchical Context Distribution.

use crate::identity::Role;

#[derive(Debug, thiserror::Error)]
#[error("Delegation violation: {0}")]
pub struct DelegationError(pub String);

#[derive(Debug, Clone)]
pub struct DelegationRules {
    pub my_role: Role,
    pub my_vertical: String,
}

impl DelegationRules {
    pub fn new(role: Role, vertical: &str) -> Self {
        Self {
            my_role: role,
            my_vertical: vertical.to_string(),
        }
    }

    /// Can this mind spawn a child of the given role?
    pub fn can_spawn(&self, child_role: Role) -> Result<(), DelegationError> {
        match (self.my_role, child_role) {
            (Role::Primary, Role::TeamLead) => Ok(()),
            (Role::TeamLead, Role::Agent) => Ok(()),
            (Role::Agent, _) => Err(DelegationError(
                "Agents CANNOT spawn children".into(),
            )),
            (Role::Primary, Role::Agent) => Err(DelegationError(
                "Primary can ONLY spawn Team Leads, not Agents".into(),
            )),
            (Role::Primary, Role::Primary) => Err(DelegationError(
                "Primary CANNOT spawn another Primary".into(),
            )),
            (Role::TeamLead, Role::TeamLead) => Err(DelegationError(
                "Team Leads CANNOT spawn other Team Leads".into(),
            )),
            (Role::TeamLead, Role::Primary) => Err(DelegationError(
                "Team Leads CANNOT spawn Primary".into(),
            )),
        }
    }

    /// Can this mind delegate to another mind?
    pub fn can_delegate_to(
        &self,
        target_role: Role,
        target_vertical: &str,
    ) -> Result<(), DelegationError> {
        match (self.my_role, target_role) {
            (Role::Primary, Role::TeamLead) => Ok(()),
            (Role::TeamLead, Role::Agent) => {
                if target_vertical == self.my_vertical {
                    Ok(())
                } else {
                    Err(DelegationError(format!(
                        "Team Lead ({}) can ONLY delegate to Agents in same vertical ({}), not {}",
                        self.my_vertical, self.my_vertical, target_vertical
                    )))
                }
            }
            (Role::Agent, _) => Err(DelegationError(
                "Agents CANNOT delegate to anyone".into(),
            )),
            (Role::Primary, Role::Agent) => Err(DelegationError(
                "Primary CANNOT delegate directly to Agents — must go through Team Leads".into(),
            )),
            (Role::Primary, Role::Primary) => Err(DelegationError(
                "Primary CANNOT delegate to another Primary".into(),
            )),
            (Role::TeamLead, Role::TeamLead) => Err(DelegationError(
                "Team Leads CANNOT delegate to other Team Leads — go through Primary".into(),
            )),
            (Role::TeamLead, Role::Primary) => Err(DelegationError(
                "Team Leads CANNOT delegate to Primary — report to Primary, don't delegate".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_can_spawn_team_lead() {
        let rules = DelegationRules::new(Role::Primary, "all");
        assert!(rules.can_spawn(Role::TeamLead).is_ok());
    }

    #[test]
    fn team_lead_can_spawn_agent() {
        let rules = DelegationRules::new(Role::TeamLead, "research");
        assert!(rules.can_spawn(Role::Agent).is_ok());
    }

    #[test]
    fn agent_cannot_spawn() {
        let rules = DelegationRules::new(Role::Agent, "research");
        assert!(rules.can_spawn(Role::Agent).is_err());
    }

    #[test]
    fn primary_cannot_spawn_agent() {
        let rules = DelegationRules::new(Role::Primary, "all");
        assert!(rules.can_spawn(Role::Agent).is_err());
    }

    #[test]
    fn team_lead_can_delegate_to_same_vertical() {
        let rules = DelegationRules::new(Role::TeamLead, "research");
        assert!(rules.can_delegate_to(Role::Agent, "research").is_ok());
    }

    #[test]
    fn team_lead_cannot_delegate_to_different_vertical() {
        let rules = DelegationRules::new(Role::TeamLead, "research");
        assert!(rules.can_delegate_to(Role::Agent, "code").is_err());
    }

    #[test]
    fn agent_cannot_delegate() {
        let rules = DelegationRules::new(Role::Agent, "research");
        assert!(rules.can_delegate_to(Role::Agent, "research").is_err());
    }

    #[test]
    fn primary_cannot_delegate_to_agent_directly() {
        let rules = DelegationRules::new(Role::Primary, "all");
        assert!(rules.can_delegate_to(Role::Agent, "research").is_err());
    }
}
