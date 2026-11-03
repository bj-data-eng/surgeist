//! Windows connection-cost observations and the native reduced-data policy.

use crate::ReductionPreference;

#[derive(Clone, Copy)]
enum CostKind {
    Unrestricted,
    Fixed,
    Variable,
}

/// Partial native facts stay unavailable until every required read succeeded.
struct ConnectionCostObservation {
    kind: Option<CostKind>,
    roaming: Option<bool>,
    over_limit: Option<bool>,
    approaching_limit: Option<bool>,
}

impl ConnectionCostObservation {
    fn preference(self) -> Option<ReductionPreference> {
        let kind = self.kind?;
        let roaming = self.roaming?;
        let over_limit = self.over_limit?;
        let approaching_limit = self.approaching_limit?;
        // MQ5 permits UA inference from unlimited versus metered connectivity.
        // This host also reduces data for the three explicit cost constraints;
        // it does not claim these observations are a Windows CSS preference.
        Some(match kind {
            CostKind::Unrestricted if !roaming && !over_limit && !approaching_limit => {
                ReductionPreference::NoPreference
            }
            _ => ReductionPreference::Reduce,
        })
    }
}

#[cfg(target_os = "windows")]
fn known_cost(kind: windows::Networking::Connectivity::NetworkCostType) -> Option<CostKind> {
    use windows::Networking::Connectivity::NetworkCostType;

    match kind {
        NetworkCostType::Unrestricted => Some(CostKind::Unrestricted),
        NetworkCostType::Fixed => Some(CostKind::Fixed),
        NetworkCostType::Variable => Some(CostKind::Variable),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
pub(super) fn capture() -> Option<ReductionPreference> {
    use windows::Networking::Connectivity::NetworkInformation;

    // Get a fresh preferred profile on every sample. A missing profile is a
    // failed projected object result, not proof of an unrestricted connection.
    let profile = NetworkInformation::GetInternetConnectionProfile().ok()?;
    let cost = profile.GetConnectionCost().ok()?;
    ConnectionCostObservation {
        kind: cost.NetworkCostType().ok().and_then(known_cost),
        roaming: cost.Roaming().ok(),
        over_limit: cost.OverDataLimit().ok(),
        approaching_limit: cost.ApproachingDataLimit().ok(),
    }
    .preference()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_cost_and_constraint_combinations_select_the_native_data_policy() {
        for kind in [CostKind::Unrestricted, CostKind::Fixed, CostKind::Variable] {
            for (flags, unrestricted_expected) in [
                ([false, false, false], ReductionPreference::NoPreference),
                ([true, false, false], ReductionPreference::Reduce),
                ([false, true, false], ReductionPreference::Reduce),
                ([false, false, true], ReductionPreference::Reduce),
                ([true, true, false], ReductionPreference::Reduce),
                ([true, false, true], ReductionPreference::Reduce),
                ([false, true, true], ReductionPreference::Reduce),
                ([true, true, true], ReductionPreference::Reduce),
            ] {
                let expected = match kind {
                    CostKind::Unrestricted => unrestricted_expected,
                    CostKind::Fixed | CostKind::Variable => ReductionPreference::Reduce,
                };
                assert_eq!(
                    ConnectionCostObservation {
                        kind: Some(kind),
                        roaming: Some(flags[0]),
                        over_limit: Some(flags[1]),
                        approaching_limit: Some(flags[2]),
                    }
                    .preference(),
                    Some(expected)
                );
            }
        }
    }

    #[test]
    fn any_missing_cost_read_keeps_the_data_preference_unavailable() {
        for kind in [
            None,
            Some(CostKind::Unrestricted),
            Some(CostKind::Fixed),
            Some(CostKind::Variable),
        ] {
            for missing in 0..4 {
                assert_eq!(
                    ConnectionCostObservation {
                        kind: if missing == 0 { None } else { kind },
                        roaming: (missing != 1).then_some(true),
                        over_limit: (missing != 2).then_some(true),
                        approaching_limit: (missing != 3).then_some(true),
                    }
                    .preference(),
                    None
                );
            }
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn unknown_and_future_native_cost_values_stay_unavailable() {
        use windows::Networking::Connectivity::NetworkCostType;

        for kind in [
            NetworkCostType::Unknown,
            NetworkCostType(-1),
            NetworkCostType(4),
        ] {
            assert!(known_cost(kind).is_none());
        }
        assert!(matches!(
            known_cost(NetworkCostType::Unrestricted),
            Some(CostKind::Unrestricted)
        ));
        assert!(matches!(
            known_cost(NetworkCostType::Fixed),
            Some(CostKind::Fixed)
        ));
        assert!(matches!(
            known_cost(NetworkCostType::Variable),
            Some(CostKind::Variable)
        ));
    }
}
