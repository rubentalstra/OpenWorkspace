//! The pure resource-visibility (segmentation) model.
//!
//! A resource's effective organization/team is resolved by overriding from the
//! most specific source to the least: an explicit resource binding wins over
//! its floor zone, which wins over its location. [`ResourceSegmentation::effective`]
//! computes that pair; [`visible`] then decides whether a given viewer may see
//! the resource under the configured [`SegmentationMode`].
//!
//! All functions are total, I/O-free and clock-free: the caller materializes
//! the org/team bindings and the viewer's org/team sets from the database.

use std::collections::HashSet;

use crate::enums::SegmentationMode;
use crate::ids::{OrganizationId, TeamId};

/// The org/team bindings at each level of a resource's hierarchy, most-specific
/// first.
///
/// `effective()` resolves the binding by override precedence: resource → zone →
/// location for the organization, and resource → zone for the team (a location
/// has no team binding in the schema).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ResourceSegmentation {
    /// Organization explicitly set on the resource row, if any.
    pub resource_org: Option<OrganizationId>,
    /// Team explicitly set on the resource row, if any.
    pub resource_team: Option<TeamId>,
    /// Organization of the resource's floor zone, if it sits in one.
    pub zone_org: Option<OrganizationId>,
    /// Team of the resource's floor zone, if it sits in one.
    pub zone_team: Option<TeamId>,
    /// Organization of the resource's location, if the location is org-bound.
    pub location_org: Option<OrganizationId>,
}

/// A resource's effective (organization, team) after override resolution.
///
/// The effective team carries its **owning organization**, sourced from the
/// same level that supplied the team. Visibility under
/// [`SegmentationMode::ByOrganizationAndTeam`] requires that owning org to match
/// the effective org, so a team binding can never be honored against an
/// unrelated effective org (mixed-provenance binding).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EffectiveSegmentation {
    /// The effective organization, if any.
    pub organization: Option<OrganizationId>,
    /// The effective team paired with its owning organization, if any.
    pub team: Option<(OrganizationId, TeamId)>,
}

impl ResourceSegmentation {
    /// Resolves the effective organization/team by override precedence.
    ///
    /// Organization: `resource_org` → `zone_org` → `location_org`. Team:
    /// `resource_team` → `zone_team` (locations carry no team), each paired
    /// with the org of the **same** level so the team's provenance org travels
    /// with it. The first `Some` at each, scanning most-specific to least,
    /// wins.
    #[must_use]
    pub fn effective(&self) -> EffectiveSegmentation {
        let team = match (self.resource_team, self.zone_team) {
            (Some(t), _) => self.resource_org.map(|o| (o, t)),
            (None, Some(t)) => self.zone_org.map(|o| (o, t)),
            (None, None) => None,
        };
        EffectiveSegmentation {
            organization: self.resource_org.or(self.zone_org).or(self.location_org),
            team,
        }
    }
}

/// What a viewer is allowed to see, materialized by the caller.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ViewerSegmentation {
    /// Whether the viewer is the instance admin (sees everything).
    pub is_instance_admin: bool,
    /// Organizations the viewer belongs to.
    pub orgs: HashSet<OrganizationId>,
    /// Teams the viewer belongs to.
    pub teams: HashSet<TeamId>,
}

/// Whether `viewer` may see a resource with effective binding `effective` under
/// `mode`. Total and fail-closed.
///
/// - The instance admin always sees the resource.
/// - [`SegmentationMode::Open`] ⇒ always visible.
/// - [`SegmentationMode::ByOrganization`] ⇒ the effective org must be one of
///   the viewer's orgs. A `None` effective org is **fail-closed** (not
///   visible).
/// - [`SegmentationMode::ByOrganizationAndTeam`] ⇒ the org must match as above,
///   **and** (the effective team is `None` — org-wide — or the team's owning
///   org equals the effective org *and* the team is one of the viewer's teams).
///   Requiring the team's owning org to match the effective org rejects a
///   mixed-provenance binding where a team from one org rides on a different
///   effective org.
#[must_use]
pub fn visible(
    effective: EffectiveSegmentation,
    viewer: &ViewerSegmentation,
    mode: SegmentationMode,
) -> bool {
    if viewer.is_instance_admin {
        return true;
    }
    match mode {
        SegmentationMode::Open => true,
        SegmentationMode::ByOrganization => effective
            .organization
            .is_some_and(|o| viewer.orgs.contains(&o)),
        SegmentationMode::ByOrganizationAndTeam => {
            let org_ok = effective
                .organization
                .is_some_and(|o| viewer.orgs.contains(&o));
            let team_ok = match effective.team {
                None => true,
                Some((team_org, t)) => {
                    effective.organization == Some(team_org) && viewer.teams.contains(&t)
                }
            };
            org_ok && team_ok
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use uuid::Uuid;

    fn org(n: u128) -> OrganizationId {
        OrganizationId::new(Uuid::from_u128(n))
    }
    fn team(n: u128) -> TeamId {
        TeamId::new(Uuid::from_u128(n))
    }

    fn viewer(orgs: &[u128], teams: &[u128]) -> ViewerSegmentation {
        ViewerSegmentation {
            is_instance_admin: false,
            orgs: orgs.iter().map(|&n| org(n)).collect(),
            teams: teams.iter().map(|&n| team(n)).collect(),
        }
    }

    /// Builds an effective binding where the team (if any) is owned by the same
    /// org as the effective org — the well-formed case.
    fn eff(o: Option<u128>, t: Option<u128>) -> EffectiveSegmentation {
        EffectiveSegmentation {
            organization: o.map(org),
            team: t.and_then(|tn| o.map(|on| (org(on), team(tn)))),
        }
    }

    /// Builds an effective binding where the team is owned by `team_org`,
    /// independent of the effective org — for mixed-provenance cases.
    fn eff_team_org(o: Option<u128>, team_org: u128, t: u128) -> EffectiveSegmentation {
        EffectiveSegmentation {
            organization: o.map(org),
            team: Some((org(team_org), team(t))),
        }
    }

    // --- effective() precedence ---

    #[test]
    fn effective_resource_org_overrides_zone_and_location() {
        let s = ResourceSegmentation {
            resource_org: Some(org(1)),
            resource_team: None,
            zone_org: Some(org(2)),
            zone_team: None,
            location_org: Some(org(3)),
        };
        assert_eq!(s.effective().organization, Some(org(1)));
    }

    #[test]
    fn effective_zone_org_overrides_location() {
        let s = ResourceSegmentation {
            resource_org: None,
            resource_team: None,
            zone_org: Some(org(2)),
            zone_team: None,
            location_org: Some(org(3)),
        };
        assert_eq!(s.effective().organization, Some(org(2)));
    }

    #[test]
    fn effective_falls_back_to_location_org() {
        let s = ResourceSegmentation {
            resource_org: None,
            resource_team: None,
            zone_org: None,
            zone_team: None,
            location_org: Some(org(3)),
        };
        assert_eq!(s.effective().organization, Some(org(3)));
        assert_eq!(s.effective().team, None);
    }

    #[test]
    fn effective_team_resource_over_zone() {
        let s = ResourceSegmentation {
            resource_org: Some(org(1)),
            resource_team: Some(team(5)),
            zone_org: Some(org(2)),
            zone_team: Some(team(6)),
            location_org: None,
        };
        // The resource-level team wins, paired with the resource-level org.
        assert_eq!(s.effective().team, Some((org(1), team(5))));
    }

    #[test]
    fn effective_team_falls_back_to_zone() {
        let s = ResourceSegmentation {
            resource_org: None,
            resource_team: None,
            zone_org: Some(org(2)),
            zone_team: Some(team(6)),
            location_org: None,
        };
        // The zone-level team carries the zone-level org as its provenance.
        assert_eq!(s.effective().team, Some((org(2), team(6))));
    }

    #[test]
    fn effective_team_without_owning_org_is_dropped() {
        // A team set with no org at the same level has unknown provenance, so
        // it is fail-closed (no team binding).
        let s = ResourceSegmentation {
            resource_org: None,
            resource_team: Some(team(5)),
            zone_org: None,
            zone_team: None,
            location_org: Some(org(3)),
        };
        assert_eq!(s.effective().organization, Some(org(3)));
        assert_eq!(s.effective().team, None);
    }

    // --- FIX 4 regression: mixed-provenance team binding ---

    #[test]
    fn mixed_provenance_team_is_rejected() {
        // Effective org = A, but the effective team T is owned by org B.
        // A viewer who is in org A and in team T must NOT see the resource:
        // the team belongs to a different org than the effective org.
        let org_a = 1;
        let org_b = 2;
        let team_t = 10;
        let mixed = eff_team_org(Some(org_a), org_b, team_t);
        let v = viewer(&[org_a], &[team_t]);
        assert!(!visible(mixed, &v, SegmentationMode::ByOrganizationAndTeam));

        // Sanity: the same team owned by org A (well-formed) is visible.
        let well_formed = eff_team_org(Some(org_a), org_a, team_t);
        assert!(visible(
            well_formed,
            &v,
            SegmentationMode::ByOrganizationAndTeam
        ));
    }

    // --- visibility matrix ---

    #[test]
    fn open_is_always_visible() {
        let v = viewer(&[], &[]);
        // Even with no matching org/team and a null effective org.
        assert!(visible(eff(None, None), &v, SegmentationMode::Open));
        assert!(visible(eff(Some(99), Some(99)), &v, SegmentationMode::Open));
    }

    #[test]
    fn by_organization_requires_org_membership() {
        let v = viewer(&[1], &[]);
        assert!(visible(
            eff(Some(1), None),
            &v,
            SegmentationMode::ByOrganization
        ));
        assert!(!visible(
            eff(Some(2), None),
            &v,
            SegmentationMode::ByOrganization
        ));
    }

    #[test]
    fn by_organization_null_org_fail_closed() {
        let v = viewer(&[1], &[]);
        assert!(!visible(
            eff(None, None),
            &v,
            SegmentationMode::ByOrganization
        ));
    }

    #[test]
    fn by_org_and_team_matrix() {
        let v = viewer(&[1], &[10]);
        // org match + team match → visible.
        assert!(visible(
            eff(Some(1), Some(10)),
            &v,
            SegmentationMode::ByOrganizationAndTeam
        ));
        // org match + org-wide (team None) → visible.
        assert!(visible(
            eff(Some(1), None),
            &v,
            SegmentationMode::ByOrganizationAndTeam
        ));
        // org match + wrong team → hidden.
        assert!(!visible(
            eff(Some(1), Some(11)),
            &v,
            SegmentationMode::ByOrganizationAndTeam
        ));
        // wrong org (even with team match) → hidden.
        assert!(!visible(
            eff(Some(2), Some(10)),
            &v,
            SegmentationMode::ByOrganizationAndTeam
        ));
        // null org → hidden (fail-closed).
        assert!(!visible(
            eff(None, None),
            &v,
            SegmentationMode::ByOrganizationAndTeam
        ));
    }

    #[test]
    fn instance_admin_always_visible() {
        let mut v = viewer(&[], &[]);
        v.is_instance_admin = true;
        for mode in [
            SegmentationMode::Open,
            SegmentationMode::ByOrganization,
            SegmentationMode::ByOrganizationAndTeam,
        ] {
            assert!(visible(eff(None, None), &v, mode));
            assert!(visible(eff(Some(99), Some(99)), &v, mode));
        }
    }

    // --- proptests: totality + invariants ---

    fn arb_mode() -> impl Strategy<Value = SegmentationMode> {
        prop_oneof![
            Just(SegmentationMode::Open),
            Just(SegmentationMode::ByOrganization),
            Just(SegmentationMode::ByOrganizationAndTeam),
        ]
    }

    fn arb_eff() -> impl Strategy<Value = EffectiveSegmentation> {
        // Explore well-formed bindings AND mixed-provenance ones where the
        // team's owning org may differ from the effective org.
        (
            proptest::option::of(0u128..4),
            proptest::option::of((0u128..4, 0u128..4)),
        )
            .prop_map(|(o, team_pair)| EffectiveSegmentation {
                organization: o.map(org),
                team: team_pair.map(|(to, t)| (org(to), team(t))),
            })
    }

    fn arb_viewer() -> impl Strategy<Value = ViewerSegmentation> {
        (
            any::<bool>(),
            proptest::collection::hash_set(0u128..4, 0..4),
            proptest::collection::hash_set(0u128..4, 0..4),
        )
            .prop_map(|(admin, orgs, teams)| ViewerSegmentation {
                is_instance_admin: admin,
                orgs: orgs.into_iter().map(org).collect(),
                teams: teams.into_iter().map(team).collect(),
            })
    }

    proptest! {
        #[test]
        fn visible_is_total_and_deterministic(e in arb_eff(), v in arb_viewer(), mode in arb_mode()) {
            // Never panics, and is a pure function: the same inputs give the
            // same answer.
            let first = visible(e, &v, mode);
            let second = visible(e, &v, mode);
            prop_assert_eq!(first, second);
        }

        #[test]
        fn open_is_always_true(e in arb_eff(), v in arb_viewer()) {
            prop_assert!(visible(e, &v, SegmentationMode::Open));
        }

        #[test]
        fn instance_admin_sees_all(e in arb_eff(), mut v in arb_viewer(), mode in arb_mode()) {
            v.is_instance_admin = true;
            prop_assert!(visible(e, &v, mode));
        }

        #[test]
        fn null_effective_org_fail_closed_for_org_modes(v in arb_viewer(), mode in arb_mode()) {
            // For a non-admin viewer with a null effective org, the org-scoped
            // modes must hide the resource.
            prop_assume!(!v.is_instance_admin);
            prop_assume!(mode != SegmentationMode::Open);
            let e = EffectiveSegmentation { organization: None, team: None };
            prop_assert!(!visible(e, &v, mode));
        }

        #[test]
        fn by_org_and_team_is_no_more_permissive_than_by_org(
            e in arb_eff(),
            v in arb_viewer(),
        ) {
            // Adding the team predicate can only ever remove visibility, never add it.
            prop_assume!(!v.is_instance_admin);
            let by_org = visible(e, &v, SegmentationMode::ByOrganization);
            let by_team = visible(e, &v, SegmentationMode::ByOrganizationAndTeam);
            if by_team {
                prop_assert!(by_org);
            }
        }

        #[test]
        fn team_match_requires_team_org_equals_effective_org(
            e in arb_eff(),
            v in arb_viewer(),
        ) {
            // If a resource is visible under team mode but NOT org-wide (i.e. it
            // had a team binding that mattered), the team's owning org must be
            // the effective org — a mixed-provenance team can never grant view.
            prop_assume!(!v.is_instance_admin);
            if visible(e, &v, SegmentationMode::ByOrganizationAndTeam)
                && let Some((team_org, _)) = e.team
            {
                prop_assert_eq!(e.organization, Some(team_org));
            }
        }
    }
}
