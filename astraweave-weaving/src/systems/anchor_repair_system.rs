/// Anchor Repair System
///
/// Handles anchor repair mechanics:
/// - Check EchoCurrency balance >= repair_cost
/// - Deduct Echoes (transaction logged)
/// - Play 5s repair animation
/// - Apply +0.3 stability boost
/// - Unlock ability if specified
///
/// Integration:
/// - Input: RepairRequest events (from UI system)
/// - Input: Query<&mut Anchor>, Query<&mut EchoCurrency>
/// - Output: RepairEvents (for audio/VFX systems)
use crate::anchor::Anchor;
use crate::echo_currency::{EchoCurrency, TransactionReason};

/// Repair request from UI system
#[derive(Debug, Clone)]
pub struct RepairRequest {
    pub anchor_id: usize,
}

/// Repair event result
#[derive(Debug, Clone)]
pub struct RepairEvent {
    pub anchor_id: usize,
    pub result: RepairResult,
    pub ability_unlocked: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairResult {
    Success,             // Repair successful
    InsufficientEchoes,  // Not enough Echoes
    AlreadyMaxStability, // Already at 1.0
}

/// Anchor repair system (ECS system function)
pub fn anchor_repair_system(
    repair_requests: &[RepairRequest],
    anchors: &mut [(usize, &mut Anchor)],
    echo_currency: &mut EchoCurrency,
) -> Vec<RepairEvent> {
    let mut events = Vec::new();

    for request in repair_requests {
        // Find anchor
        let anchor_entry = anchors.iter_mut().find(|(id, _)| *id == request.anchor_id);
        if anchor_entry.is_none() {
            continue;
        }
        let (anchor_id, anchor) = anchor_entry.unwrap();

        // Check if already at max stability
        if anchor.stability() >= 1.0 {
            events.push(RepairEvent {
                anchor_id: *anchor_id,
                result: RepairResult::AlreadyMaxStability,
                ability_unlocked: None,
            });
            continue;
        }

        // Check Echo balance
        let cost = anchor.repair_cost();
        if !echo_currency.has(cost) {
            events.push(RepairEvent {
                anchor_id: *anchor_id,
                result: RepairResult::InsufficientEchoes,
                ability_unlocked: None,
            });
            continue;
        }

        // Deduct Echoes
        let anchor_name = format!("Anchor_{}", anchor_id);
        if !echo_currency.spend(cost, TransactionReason::RepairAnchor(anchor_name)) {
            // Should not happen (we checked balance above), but safety check
            events.push(RepairEvent {
                anchor_id: *anchor_id,
                result: RepairResult::InsufficientEchoes,
                ability_unlocked: None,
            });
            continue;
        }

        // Apply repair
        let _ability_before = anchor.unlocks_ability().map(|a| format!("{:?}", a));
        let was_broken = anchor.stability() < 0.7;
        anchor.repair();

        // Check if ability unlocked (only if anchor was broken/unstable before repair)
        let ability_unlocked = if was_broken {
            anchor.unlocks_ability().map(|a| format!("{:?}", a))
        } else {
            None
        };

        events.push(RepairEvent {
            anchor_id: *anchor_id,
            result: RepairResult::Success,
            ability_unlocked,
        });
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::{AbilityType, Anchor};

    #[test]
    fn test_successful_repair() {
        let mut anchor = Anchor::new(0.5, 5, Some(AbilityType::EchoDash));
        let mut anchors = vec![(1, &mut anchor)];
        let mut currency = EchoCurrency::with_balance(10);

        let requests = vec![RepairRequest { anchor_id: 1 }];
        let events = anchor_repair_system(&requests, &mut anchors, &mut currency);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].result, RepairResult::Success);
        assert_eq!(events[0].ability_unlocked, Some("EchoDash".to_string()));

        // Check anchor stability increased
        assert!((anchors[0].1.stability() - 0.8).abs() < 0.001); // 0.5 + 0.3 = 0.8

        // Check Echoes deducted
        assert_eq!(currency.count(), 5); // 10 - 5 = 5
    }

    #[test]
    fn test_insufficient_echoes() {
        let mut anchor = Anchor::new(0.5, 10, None);
        let mut anchors = vec![(1, &mut anchor)];
        let mut currency = EchoCurrency::with_balance(5); // Not enough

        let requests = vec![RepairRequest { anchor_id: 1 }];
        let events = anchor_repair_system(&requests, &mut anchors, &mut currency);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].result, RepairResult::InsufficientEchoes);

        // Check anchor unchanged
        assert_eq!(anchors[0].1.stability(), 0.5);

        // Check Echoes unchanged
        assert_eq!(currency.count(), 5);
    }

    #[test]
    fn test_already_max_stability() {
        let mut anchor = Anchor::new(1.0, 5, None);
        let mut anchors = vec![(1, &mut anchor)];
        let mut currency = EchoCurrency::with_balance(10);

        let requests = vec![RepairRequest { anchor_id: 1 }];
        let events = anchor_repair_system(&requests, &mut anchors, &mut currency);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].result, RepairResult::AlreadyMaxStability);

        // Check Echoes unchanged (no deduction)
        assert_eq!(currency.count(), 10);
    }

    #[test]
    fn test_repair_broken_anchor() {
        let mut anchor = Anchor::new(0.0, 1, Some(AbilityType::BarricadeDeploy));
        let mut anchors = vec![(1, &mut anchor)];
        let mut currency = EchoCurrency::with_balance(5);

        let requests = vec![RepairRequest { anchor_id: 1 }];
        let events = anchor_repair_system(&requests, &mut anchors, &mut currency);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].result, RepairResult::Success);
        assert_eq!(
            events[0].ability_unlocked,
            Some("BarricadeDeploy".to_string())
        );

        // Check stability: 0.0 + 0.3 = 0.3
        assert!((anchors[0].1.stability() - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_repair_caps_at_1_0() {
        let mut anchor = Anchor::new(0.9, 2, None);
        let mut anchors = vec![(1, &mut anchor)];
        let mut currency = EchoCurrency::with_balance(10);

        let requests = vec![RepairRequest { anchor_id: 1 }];
        let events = anchor_repair_system(&requests, &mut anchors, &mut currency);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].result, RepairResult::Success);

        // Check stability: 0.9 + 0.3 = 1.2 → capped at 1.0
        assert_eq!(anchors[0].1.stability(), 1.0);
    }

    #[test]
    fn test_multiple_repair_requests() {
        let mut anchor1 = Anchor::new(0.5, 3, None);
        let mut anchor2 = Anchor::new(0.3, 2, None);
        let mut anchors = vec![(1, &mut anchor1), (2, &mut anchor2)];
        let mut currency = EchoCurrency::with_balance(10);

        let requests = vec![
            RepairRequest { anchor_id: 1 },
            RepairRequest { anchor_id: 2 },
        ];
        let events = anchor_repair_system(&requests, &mut anchors, &mut currency);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].result, RepairResult::Success);
        assert_eq!(events[1].result, RepairResult::Success);

        // Check Echoes: 10 - 3 - 2 = 5
        assert_eq!(currency.count(), 5);
    }

    #[test]
    fn test_transaction_logged() {
        let mut anchor = Anchor::new(0.5, 5, None);
        let mut anchors = vec![(1, &mut anchor)];
        let mut currency = EchoCurrency::with_balance(10);

        let requests = vec![RepairRequest { anchor_id: 1 }];
        anchor_repair_system(&requests, &mut anchors, &mut currency);

        // Check transaction logged
        let last = currency.last_transaction();
        assert!(last.is_some());
        let txn = last.unwrap();
        assert_eq!(txn.amount(), -5); // Spent 5 Echoes
        assert!(matches!(txn.reason(), TransactionReason::RepairAnchor(_)));
    }

    #[test]
    fn test_ability_not_unlocked_if_stable() {
        let mut anchor = Anchor::new(0.75, 5, Some(AbilityType::EchoDash));
        let mut anchors = vec![(1, &mut anchor)];
        let mut currency = EchoCurrency::with_balance(10);

        let requests = vec![RepairRequest { anchor_id: 1 }];
        let events = anchor_repair_system(&requests, &mut anchors, &mut currency);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].result, RepairResult::Success);

        // Ability NOT unlocked (anchor was already stable)
        assert!(events[0].ability_unlocked.is_none());
    }

    #[test]
    fn test_zero_cost_repair() {
        let mut anchor = Anchor::new(0.5, 0, None); // Free repair
        let mut anchors = vec![(1, &mut anchor)];
        let mut currency = EchoCurrency::with_balance(0); // No Echoes

        let requests = vec![RepairRequest { anchor_id: 1 }];
        let events = anchor_repair_system(&requests, &mut anchors, &mut currency);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].result, RepairResult::Success);

        // Check stability increased
        assert!((anchors[0].1.stability() - 0.8).abs() < 0.001);
    }
}
