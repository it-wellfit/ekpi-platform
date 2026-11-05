#![forbid(unsafe_code)]

//! Authorization rules.
//!
//! Authorization is expressed as capabilities with a minimum role. Every check
//! fails closed: an unknown permission, a missing role or a malformed context
//! is a denial, never an implicit allow.

use ekp_domain::auth::{Permission, Role};

use crate::ports::GovernanceError;

/// Return `Ok(())` when `role` may exercise `permission`.
pub fn authorize(role: Role, permission: Permission) -> Result<(), GovernanceError> {
    if role.satisfies(permission.minimum_role()) {
        Ok(())
    } else {
        Err(GovernanceError::validation(format!(
            "role {} may not exercise {}",
            role.as_str(),
            permission_name(permission)
        )))
    }
}

/// The stable name of a permission, used in audit records and error messages.
pub fn permission_name(permission: Permission) -> &'static str {
    match permission {
        Permission::ViewGovernance => "view_governance",
        Permission::EditGovernance => "edit_governance",
        Permission::ApproveGovernance => "approve_governance",
        Permission::ManageUsers => "manage_users",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_are_allowed_at_or_above_the_minimum() {
        assert!(authorize(Role::Viewer, Permission::ViewGovernance).is_ok());
        assert!(authorize(Role::Editor, Permission::EditGovernance).is_ok());
        assert!(authorize(Role::Approver, Permission::ApproveGovernance).is_ok());
        assert!(authorize(Role::Admin, Permission::ManageUsers).is_ok());
    }

    #[test]
    fn roles_below_the_minimum_are_denied() {
        assert!(authorize(Role::Viewer, Permission::EditGovernance).is_err());
        assert!(authorize(Role::Editor, Permission::ApproveGovernance).is_err());
        assert!(authorize(Role::Approver, Permission::ManageUsers).is_err());
    }

    #[test]
    fn denial_messages_name_the_permission() {
        let error = authorize(Role::Viewer, Permission::ManageUsers).expect_err("denied");
        assert!(error.message().contains("manage_users"));
    }
}
