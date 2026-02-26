use rustok_core::UserRole;

use super::shadow_decision::{compare_shadow_decision, ShadowCheck};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DualReadOutcome {
    Disabled,
    Skipped,
    Matched,
    Mismatch,
}

pub fn evaluate_dual_read(
    dual_read_enabled: bool,
    legacy_role: Option<&UserRole>,
    shadow_check: ShadowCheck<'_>,
    relation_allowed: bool,
) -> DualReadOutcome {
    if !dual_read_enabled {
        return DualReadOutcome::Disabled;
    }

    let Some(legacy_role) = legacy_role else {
        return DualReadOutcome::Skipped;
    };

    let shadow = compare_shadow_decision(legacy_role, shadow_check, relation_allowed);
    if shadow.mismatch() {
        DualReadOutcome::Mismatch
    } else {
        DualReadOutcome::Matched
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate_dual_read, DualReadOutcome};
    use crate::services::shadow_decision::ShadowCheck;
    use rustok_core::{Action, Permission, Resource, UserRole};

    fn permission(resource: Resource, action: Action) -> Permission {
        Permission::new(resource, action)
    }

    #[test]
    fn returns_disabled_when_mode_off() {
        let required = permission(Resource::Users, Action::Read);

        let outcome = evaluate_dual_read(
            false,
            Some(&UserRole::Editor),
            ShadowCheck::Single(&required),
            true,
        );

        assert_eq!(outcome, DualReadOutcome::Disabled);
    }

    #[test]
    fn returns_skipped_when_legacy_role_missing() {
        let required = permission(Resource::Users, Action::Read);

        let outcome = evaluate_dual_read(false, None, ShadowCheck::Single(&required), true);
        assert_eq!(outcome, DualReadOutcome::Disabled);

        let outcome = evaluate_dual_read(true, None, ShadowCheck::Single(&required), true);
        assert_eq!(outcome, DualReadOutcome::Skipped);
    }

    #[test]
    fn returns_matched_when_relation_equals_legacy() {
        let required = permission(Resource::BlogPost, Action::Read);

        let outcome = evaluate_dual_read(
            true,
            Some(&UserRole::Editor),
            ShadowCheck::Single(&required),
            true,
        );

        assert_eq!(outcome, DualReadOutcome::Matched);
    }

    #[test]
    fn returns_mismatch_when_relation_differs_from_legacy() {
        let required = permission(Resource::User, Action::Delete);

        let outcome = evaluate_dual_read(
            true,
            Some(&UserRole::Editor),
            ShadowCheck::Single(&required),
            true,
        );

        assert_eq!(outcome, DualReadOutcome::Mismatch);
    }
}
