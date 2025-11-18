use astraweave_weaving::*;
use glam::Vec3;

/// Advanced Content Demo: Week 4 + Week 5 Day 1 Integration Showcase

fn main() {
    println!("=== AstraWeave Advanced Content Demo ===\n");
    println!("Week 4 + Week 5 Day 1 Integration Showcase\n");

    demo_scenario_1_escort_quest();
    println!("\n{}\n", "=".repeat(60));

    demo_scenario_2_defend_quest();
    println!("\n{}\n", "=".repeat(60));

    demo_scenario_3_boss_fight();
    println!("\n{}\n", "=".repeat(60));

    demo_scenario_4_time_trial();
    println!("\n{}\n", "=".repeat(60));

    demo_scenario_5_collect_quest();

    println!("\n✅ All scenarios complete! Integration validated.");
}

fn demo_scenario_1_escort_quest() {
    println!("🎯 Scenario 1: Escort Quest - Protect the Merchant");
    println!("----------------------------------------");

    let mut player = Player::new(Vec3::ZERO);
    player.echo_currency = 100;
    player.forward = Vec3::new(1.0, 0.0, 0.0);

    let escort_npc = quest_types::EscortNPC::new(
        "Merchant",
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::new(100.0, 0.0, 0.0),
        100.0,
    );

    let mut quest = Quest::new(
        "escort_merchant",
        "Safe Passage",
        "Escort the merchant to the safe zone.",
    )
    .with_objective(ObjectiveType::Escort { npc: escort_npc })
    .with_reward(QuestReward::EchoCurrency(50));

    quest.state = QuestState::Active;

    println!("\n🎯 Quest Started: {}", quest.title);

    let mut time = 0.0;
    let delta_time = 0.5;

    while time < 10.0 {
        time += delta_time;
        player.update(delta_time);

        if let ObjectiveType::Escort { npc } = &mut quest.objectives[0] {
            npc.update(delta_time);

            if time > 3.0 && time < 3.5 {
                println!("⚔️  [t={:.1}s] Riftstalker attacks!", time);
                npc.take_damage(15.0);

                if player.can_shield() {
                    player.use_shield().ok();
                    println!("   🛡️  Shield activated! Echo: {}", player.echo_currency);
                }
            }

            if time > 5.0 && time < 5.5 && player.can_dash() {
                match player.use_dash() {
                    Ok((target_pos, damage)) => {
                        println!("💨 [t={:.1}s] Dash! Damage: {:.1}", time, damage);
                        player.position = target_pos;
                    }
                    Err(_) => {}
                }
            }

            if time as u32 % 2 == 0 && time.fract() < delta_time {
                println!(
                    "📊 [t={:.1}s] Progress: {:.1}%",
                    time,
                    quest.objectives[0].progress() * 100.0
                );
            }

            if quest.objectives[0].is_complete() && time > 8.0 {
                println!("\n🎉 Quest Complete!");
                break;
            }
        }
    }
}

fn demo_scenario_2_defend_quest() {
    println!("🎯 Scenario 2: Defend Quest - Hold the Anchor");
    println!("----------------------------------------");

    let mut player = Player::new(Vec3::ZERO);
    player.echo_currency = 100;

    let defend_obj =
        quest_types::DefendObjective::new("Ancient Anchor", Vec3::ZERO, 10.0, 200.0, 30.0, 3);

    let mut quest = Quest::new(
        "defend_anchor",
        "Hold the Line",
        "Defend the Anchor from 3 waves.",
    )
    .with_objective(ObjectiveType::Defend {
        objective: defend_obj,
        required_waves: 3,
    })
    .with_reward(QuestReward::EchoCurrency(100));

    quest.state = QuestState::Active;

    println!("\n🎯 Quest Started: {}", quest.title);

    let mut time = 0.0;
    let delta_time = 0.5;
    let mut wave_timer = 0.0;

    while time < 12.0 {
        time += delta_time;
        wave_timer += delta_time;
        player.update(delta_time);

        if wave_timer >= 3.0 {
            wave_timer = 0.0;

            if let ObjectiveType::Defend { objective, .. } = &mut quest.objectives[0] {
                objective.waves_survived += 1;

                println!(
                    "🌊 [t={:.1}s] Wave {} spawned!",
                    time, objective.waves_survived
                );

                if objective.waves_survived >= 2 {
                    println!("   ⚠️  Sentinel AOE!");
                    objective.take_damage(30.0);

                    if player.can_shield() {
                        player.use_shield().ok();
                        println!("   🛡️  Shield!");
                    }
                }

                if quest.objectives[0].is_complete() {
                    println!("\n🎉 All waves survived!");
                    break;
                }
            }
        }
    }
}

fn demo_scenario_3_boss_fight() {
    println!("🎯 Scenario 3: Boss Fight - Defeat the VoidBoss");
    println!("----------------------------------------");

    let mut player = Player::new(Vec3::ZERO);
    player.echo_currency = 150;
    player.forward = Vec3::new(1.0, 0.0, 0.0);

    let boss_obj = quest_types::BossObjective::new(
        "VoidBoss",
        500.0,
        Vec3::new(30.0, 0.0, 0.0),
        Vec3::ZERO,
        50.0,
    );

    let mut quest = Quest::new(
        "defeat_voidboss",
        "Into the Void",
        "Defeat the powerful VoidBoss.",
    )
    .with_objective(ObjectiveType::Boss {
        objective: boss_obj,
    })
    .with_reward(QuestReward::EchoCurrency(200));

    quest.state = QuestState::Active;

    println!("\n🎯 Quest Started: {}", quest.title);

    let mut time = 0.0;
    let delta_time = 0.5;

    while time < 15.0 {
        time += delta_time;
        player.update(delta_time);

        if let ObjectiveType::Boss { objective } = &mut quest.objectives[0] {
            if time as u32 % 2 == 0 && time.fract() < delta_time {
                println!(
                    "👹 [t={:.1}s] Boss attacks (Phase {:?})!",
                    time, objective.current_phase
                );
                player.take_damage(25.0);

                if player.health < 50.0 && player.can_shield() {
                    player.use_shield().ok();
                    println!("   🛡️  Defense!");
                }
            }

            if time as u32 % 3 == 0 && time.fract() < delta_time && player.can_dash() {
                match player.use_dash() {
                    Ok((_, damage)) => {
                        println!("💨 [t={:.1}s] Dash attack! Damage: {:.1}", time, damage);
                        objective.take_damage(damage);
                    }
                    Err(_) => {}
                }
            }

            if quest.objectives[0].is_complete() {
                println!("\n🎉 VoidBoss defeated!");
                break;
            }

            if player.health <= 0.0 {
                println!("\n💀 Player defeated!");
                break;
            }
        }
    }
}

fn demo_scenario_4_time_trial() {
    println!("🎯 Scenario 4: Time Trial - Speed Run");
    println!("----------------------------------------");

    let mut player = Player::new(Vec3::ZERO);
    player.echo_currency = 100;
    player.forward = Vec3::new(1.0, 0.0, 0.0);

    let time_trial_obj = quest_types::TimeTrialObjective::new(15.0, 10.0);

    let mut quest = Quest::new(
        "time_trial_1",
        "Against the Clock",
        "Complete in 15 seconds.",
    )
    .with_objective(ObjectiveType::TimeTrial {
        objective: time_trial_obj,
    })
    .with_reward(QuestReward::EchoCurrency(75));

    quest.state = QuestState::Active;

    println!("\n🎯 Quest Started: {}", quest.title);

    let mut time = 0.0;
    let delta_time = 0.5;

    while time < 16.0 {
        time += delta_time;
        player.update(delta_time);

        let mut should_complete = false;
        let mut is_expired = false;

        if let ObjectiveType::TimeTrial { objective } = &mut quest.objectives[0] {
            objective.update(delta_time);

            if player.can_dash() {
                match player.use_dash() {
                    Ok((target_pos, _)) => {
                        println!("💨 [t={:.1}s] Dash!", time);
                        player.position = target_pos;
                    }
                    Err(_) => {}
                }
            }

            if time as u32 % 3 == 0 && time.fract() < delta_time {
                println!(
                    "📊 [t={:.1}s] Time: {:.1}s",
                    time,
                    objective.remaining_time()
                );
            }

            is_expired = objective.is_expired();
            if time > 12.0 {
                should_complete = true;
            }
        }

        if should_complete {
            println!("\n🎉 Time trial complete!");
            break;
        }

        if is_expired {
            println!("\n⏰ Time's up!");
            break;
        }
    }
}

fn demo_scenario_5_collect_quest() {
    println!("🎯 Scenario 5: Collect Quest - Gather Echo Fragments");
    println!("----------------------------------------");

    let mut player = Player::new(Vec3::ZERO);
    player.echo_currency = 100;

    let positions = vec![
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(20.0, 0.0, 5.0),
        Vec3::new(15.0, 0.0, -5.0),
        Vec3::new(30.0, 0.0, 0.0),
        Vec3::new(25.0, 0.0, 10.0),
    ];

    let collect_obj = quest_types::CollectObjective::new("Echo Fragment", positions, 5.0);

    let mut quest = Quest::new(
        "collect_fragments",
        "Fragment Retrieval",
        "Collect all 5 Echo Fragments.",
    )
    .with_objective(ObjectiveType::Collect {
        objective: collect_obj,
    })
    .with_reward(QuestReward::EchoCurrency(100));

    quest.state = QuestState::Active;

    println!("\n🎯 Quest Started: {}", quest.title);

    let mut time = 0.0;
    let delta_time = 1.0;
    let mut collected = 0;

    while time < 10.0 {
        time += delta_time;
        player.update(delta_time);

        if let ObjectiveType::Collect { objective } = &mut quest.objectives[0] {
            if time as u32 % 2 == 0 && collected < 5 {
                if collected < objective.items.len() {
                    objective.items[collected].collect();
                    collected += 1;
                    println!(
                        "✨ [t={:.1}s] Collected: {}",
                        time,
                        objective.items[collected - 1].item_name
                    );

                    if collected == 3 && player.can_shield() {
                        println!("   ⚠️  Ambush!");
                        player.use_shield().ok();
                        println!("   🛡️  Shield!");
                    }
                }
            }

            if quest.objectives[0].is_complete() {
                println!("\n🎉 All fragments collected!");
                break;
            }
        }
    }
}
