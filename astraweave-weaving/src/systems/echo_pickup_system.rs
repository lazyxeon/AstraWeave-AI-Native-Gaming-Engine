/// Echo Pickup System
/// 
/// Grants Echoes to player on:
/// - Enemy kill (RiftStalker +1, Sentinel +2)
/// - Shard pickup (+1)
/// 
/// Integration:
/// - Input: EventReader<CombatEvent>, EventReader<PickupEvent>
/// - Input: Query<&mut EchoCurrency> (player component)
/// - Output: Modified EchoCurrency, transaction logging

use crate::echo_currency::{EchoCurrency, TransactionReason};

/// Combat event for Echo rewards
#[derive(Debug, Clone)]
pub struct CombatRewardEvent {
    pub enemy_type: EnemyType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    RiftStalker,  // +1 Echo
    Sentinel,     // +2 Echoes
}

impl EnemyType {
    pub fn echo_reward(&self) -> u32 {
        match self {
            EnemyType::RiftStalker => 1,
            EnemyType::Sentinel => 2,
        }
    }
}

/// Pickup event for Echo shards
#[derive(Debug, Clone)]
pub struct PickupEvent {
    pub pickup_type: PickupType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupType {
    EchoShard,  // +1 Echo
}

impl PickupType {
    pub fn echo_reward(&self) -> u32 {
        match self {
            PickupType::EchoShard => 1,
        }
    }
}

/// Echo pickup system (ECS system function)
pub fn echo_pickup_system(
    combat_events: &[CombatRewardEvent],
    pickup_events: &[PickupEvent],
    currency: &mut EchoCurrency,
) {
    // Process combat rewards
    for event in combat_events {
        let reward = event.enemy_type.echo_reward();
        let reason = match event.enemy_type {
            EnemyType::RiftStalker => TransactionReason::KillRiftStalker,
            EnemyType::Sentinel => TransactionReason::KillSentinel,
        };
        currency.add(reward, reason);
    }
    
    // Process pickups
    for event in pickup_events {
        let reward = event.pickup_type.echo_reward();
        currency.add(reward, TransactionReason::FoundShard);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_riftstalker_kill_reward() {
        let mut currency = EchoCurrency::new();
        
        let combat = vec![CombatRewardEvent { enemy_type: EnemyType::RiftStalker }];
        echo_pickup_system(&combat, &[], &mut currency);
        
        assert_eq!(currency.count(), 1);
        
        let txn = currency.last_transaction().unwrap();
        assert_eq!(txn.amount(), 1);
        assert!(matches!(txn.reason(), TransactionReason::KillRiftStalker));
    }
    
    #[test]
    fn test_sentinel_kill_reward() {
        let mut currency = EchoCurrency::new();
        
        let combat = vec![CombatRewardEvent { enemy_type: EnemyType::Sentinel }];
        echo_pickup_system(&combat, &[], &mut currency);
        
        assert_eq!(currency.count(), 2);
        
        let txn = currency.last_transaction().unwrap();
        assert_eq!(txn.amount(), 2);
        assert!(matches!(txn.reason(), TransactionReason::KillSentinel));
    }
    
    #[test]
    fn test_shard_pickup() {
        let mut currency = EchoCurrency::new();
        
        let pickups = vec![PickupEvent { pickup_type: PickupType::EchoShard }];
        echo_pickup_system(&[], &pickups, &mut currency);
        
        assert_eq!(currency.count(), 1);
        
        let txn = currency.last_transaction().unwrap();
        assert_eq!(txn.amount(), 1);
        assert!(matches!(txn.reason(), TransactionReason::FoundShard));
    }
    
    #[test]
    fn test_multiple_kills() {
        let mut currency = EchoCurrency::new();
        
        let combat = vec![
            CombatRewardEvent { enemy_type: EnemyType::RiftStalker },
            CombatRewardEvent { enemy_type: EnemyType::RiftStalker },
            CombatRewardEvent { enemy_type: EnemyType::Sentinel },
        ];
        echo_pickup_system(&combat, &[], &mut currency);
        
        // 1 + 1 + 2 = 4
        assert_eq!(currency.count(), 4);
    }
    
    #[test]
    fn test_combined_combat_and_pickups() {
        let mut currency = EchoCurrency::new();
        
        let combat = vec![
            CombatRewardEvent { enemy_type: EnemyType::RiftStalker },
            CombatRewardEvent { enemy_type: EnemyType::Sentinel },
        ];
        let pickups = vec![
            PickupEvent { pickup_type: PickupType::EchoShard },
            PickupEvent { pickup_type: PickupType::EchoShard },
        ];
        
        echo_pickup_system(&combat, &pickups, &mut currency);
        
        // 1 + 2 + 1 + 1 = 5
        assert_eq!(currency.count(), 5);
    }
    
    #[test]
    fn test_accumulation() {
        let mut currency = EchoCurrency::with_balance(10);
        
        let combat = vec![CombatRewardEvent { enemy_type: EnemyType::Sentinel }];
        echo_pickup_system(&combat, &[], &mut currency);
        
        // 10 + 2 = 12
        assert_eq!(currency.count(), 12);
    }
}
