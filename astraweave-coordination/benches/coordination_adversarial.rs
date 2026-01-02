//! Adversarial Coordination Benchmarks
//!
//! Stress testing for multi-agent coordination, social graphs, and narrative coherence.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hint::black_box as std_black_box;

// ============================================================================
// LOCAL TYPES (Mirror astraweave-coordination API)
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct AgentId(u64);

#[derive(Clone, Debug)]
struct Agent {
    id: AgentId,
    position: [f32; 3],
    faction: u32,
    role: AgentRole,
    relationships: HashMap<AgentId, f32>, // -1.0 (hostile) to 1.0 (friendly)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum AgentRole {
    Leader,
    Follower,
    Scout,
    Support,
    Attacker,
}

#[derive(Clone, Debug)]
struct Squad {
    id: u64,
    leader: AgentId,
    members: Vec<AgentId>,
    objective: String,
    formation: Formation,
}

#[derive(Clone, Copy, Debug)]
enum Formation {
    Line,
    Wedge,
    Circle,
    Scattered,
}

#[derive(Clone, Debug)]
struct WorldEvent {
    id: u64,
    event_type: EventType,
    position: [f32; 3],
    timestamp: f64,
    involved_agents: Vec<AgentId>,
}

#[derive(Clone, Copy, Debug)]
enum EventType {
    Combat,
    Discovery,
    Trade,
    Dialogue,
    Death,
}

#[derive(Clone, Debug)]
struct NarrativeThread {
    id: u64,
    name: String,
    participants: Vec<AgentId>,
    events: Vec<u64>, // Event IDs
    coherence_score: f32,
}

fn generate_agents(count: usize) -> Vec<Agent> {
    (0..count)
        .map(|i| Agent {
            id: AgentId(i as u64),
            position: [
                (i % 100) as f32,
                ((i / 100) % 100) as f32,
                (i / 10000) as f32,
            ],
            faction: (i % 4) as u32,
            role: match i % 5 {
                0 => AgentRole::Leader,
                1 => AgentRole::Follower,
                2 => AgentRole::Scout,
                3 => AgentRole::Support,
                _ => AgentRole::Attacker,
            },
            relationships: HashMap::new(),
        })
        .collect()
}

fn generate_social_graph(agents: &[Agent]) -> HashMap<AgentId, Vec<(AgentId, f32)>> {
    let mut graph = HashMap::new();

    for agent in agents {
        let connections: Vec<(AgentId, f32)> = agents
            .iter()
            .filter(|other| other.id != agent.id && other.faction == agent.faction)
            .take(10)
            .map(|other| {
                let relationship = if other.faction == agent.faction {
                    0.5 + (other.id.0 as f32 * 0.01).sin() * 0.5
                } else {
                    -0.5 + (other.id.0 as f32 * 0.01).cos() * 0.5
                };
                (other.id, relationship)
            })
            .collect();

        graph.insert(agent.id, connections);
    }

    graph
}

// ============================================================================
// CATEGORY 1: SOCIAL GRAPH OPERATIONS
// ============================================================================

fn bench_social_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordination_adversarial/social_graph");

    // Test 1: Graph construction
    for agent_count in [100, 500, 1000] {
        group.throughput(Throughput::Elements(agent_count as u64));

        group.bench_with_input(
            BenchmarkId::new("graph_construction", agent_count),
            &agent_count,
            |bencher, &count| {
                let agents = generate_agents(count);

                bencher.iter(|| {
                    let graph = generate_social_graph(&agents);
                    std_black_box(graph.len())
                });
            },
        );
    }

    // Test 2: Relationship lookup
    group.bench_function("relationship_lookup_10000", |bencher| {
        let agents = generate_agents(1000);
        let graph = generate_social_graph(&agents);

        bencher.iter(|| {
            let mut total_relationship = 0.0f32;

            for _ in 0..10000 {
                let agent_id = AgentId(rand_simple(1000) as u64);
                if let Some(connections) = graph.get(&agent_id) {
                    if let Some((_, rel)) = connections.first() {
                        total_relationship += rel;
                    }
                }
            }

            std_black_box(total_relationship)
        });
    });

    // Test 3: Path finding in social graph (BFS)
    group.bench_function("social_path_finding_100", |bencher| {
        let agents = generate_agents(500);
        let graph = generate_social_graph(&agents);

        bencher.iter(|| {
            let mut paths_found = 0;

            for i in 0..100 {
                let start = AgentId(i as u64);
                let end = AgentId((i + 50) as u64);

                // BFS for path
                let mut visited = HashSet::new();
                let mut queue = VecDeque::new();
                queue.push_back((start, 0));

                while let Some((current, depth)) = queue.pop_front() {
                    if current == end {
                        paths_found += 1;
                        break;
                    }

                    if depth > 10 || visited.contains(&current) {
                        continue;
                    }

                    visited.insert(current);

                    if let Some(neighbors) = graph.get(&current) {
                        for (neighbor, rel) in neighbors {
                            if *rel > 0.0 && !visited.contains(neighbor) {
                                queue.push_back((*neighbor, depth + 1));
                            }
                        }
                    }
                }
            }

            std_black_box(paths_found)
        });
    });

    // Test 4: Faction clustering
    group.bench_function("faction_clustering_1000", |bencher| {
        let agents = generate_agents(1000);

        bencher.iter(|| {
            let mut factions: HashMap<u32, Vec<AgentId>> = HashMap::new();

            for agent in &agents {
                factions
                    .entry(agent.faction)
                    .or_default()
                    .push(agent.id);
            }

            let sizes: Vec<usize> = factions.values().map(|v| v.len()).collect();
            std_black_box(sizes.iter().sum::<usize>())
        });
    });

    // Test 5: Relationship decay over time
    group.bench_function("relationship_decay_10000", |bencher| {
        let agents = generate_agents(500);
        let mut graph = generate_social_graph(&agents);

        bencher.iter(|| {
            let decay_rate = 0.99f32;

            for connections in graph.values_mut() {
                for (_, rel) in connections.iter_mut() {
                    *rel *= decay_rate;
                }
            }

            let total: f32 = graph
                .values()
                .flat_map(|c| c.iter().map(|(_, r)| r))
                .sum();

            std_black_box(total)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 2: SQUAD COORDINATION
// ============================================================================

fn bench_squad_coordination(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordination_adversarial/squad_coordination");

    // Test 1: Squad formation
    for squad_size in [5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::new("squad_formation", squad_size),
            &squad_size,
            |bencher, &size| {
                let agents = generate_agents(size);

                bencher.iter(|| {
                    let squad = Squad {
                        id: 1,
                        leader: agents[0].id,
                        members: agents.iter().map(|a| a.id).collect(),
                        objective: "attack".to_string(),
                        formation: Formation::Wedge,
                    };

                    std_black_box(squad.members.len())
                });
            },
        );
    }

    // Test 2: Formation position calculation
    group.bench_function("formation_positions_20", |bencher| {
        let leader_pos = [50.0f32, 50.0, 0.0];
        let member_count = 20;
        let formation = Formation::Wedge;

        bencher.iter(|| {
            let positions: Vec<[f32; 3]> = (0..member_count)
                .map(|i| match formation {
                    Formation::Line => [
                        leader_pos[0] + (i as f32 - member_count as f32 / 2.0) * 2.0,
                        leader_pos[1],
                        leader_pos[2],
                    ],
                    Formation::Wedge => {
                        let row = (i as f32).sqrt() as i32;
                        let col = i as i32 - row * row;
                        [
                            leader_pos[0] + col as f32 * 2.0 - row as f32,
                            leader_pos[1] - row as f32 * 2.0,
                            leader_pos[2],
                        ]
                    }
                    Formation::Circle => {
                        let angle = (i as f32 / member_count as f32) * std::f32::consts::TAU;
                        let radius = 5.0;
                        [
                            leader_pos[0] + angle.cos() * radius,
                            leader_pos[1] + angle.sin() * radius,
                            leader_pos[2],
                        ]
                    }
                    Formation::Scattered => [
                        leader_pos[0] + (i as f32 * 7.0 % 10.0) - 5.0,
                        leader_pos[1] + (i as f32 * 11.0 % 10.0) - 5.0,
                        leader_pos[2],
                    ],
                })
                .collect();

            std_black_box(positions.len())
        });
    });

    // Test 3: Squad objective assignment
    group.bench_function("objective_assignment_50_squads", |bencher| {
        let objectives = ["attack", "defend", "patrol", "scout", "escort"];
        let squads: Vec<Squad> = (0..50)
            .map(|i| Squad {
                id: i as u64,
                leader: AgentId(i as u64 * 10),
                members: (0..10).map(|j| AgentId(i as u64 * 10 + j)).collect(),
                objective: String::new(),
                formation: Formation::Line,
            })
            .collect();

        bencher.iter(|| {
            let assignments: Vec<(&Squad, &str)> = squads
                .iter()
                .enumerate()
                .map(|(i, squad)| (squad, objectives[i % objectives.len()]))
                .collect();

            std_black_box(assignments.len())
        });
    });

    // Test 4: Squad communication propagation
    group.bench_function("communication_propagation", |bencher| {
        let squads: Vec<Squad> = (0..10)
            .map(|i| Squad {
                id: i as u64,
                leader: AgentId(i as u64 * 10),
                members: (0..10).map(|j| AgentId(i as u64 * 10 + j)).collect(),
                objective: "hold".to_string(),
                formation: Formation::Line,
            })
            .collect();

        bencher.iter(|| {
            // Simulate message propagation from one squad to all others
            let source_squad = 0;
            let message = "retreat";
            let mut received: HashSet<u64> = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(source_squad);

            while let Some(current) = queue.pop_front() {
                if received.insert(current) {
                    // Propagate to nearby squads
                    for i in 0..squads.len() {
                        if !received.contains(&(i as u64)) {
                            queue.push_back(i as u64);
                        }
                    }
                }
            }

            std_black_box(received.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 3: WORLD EVENTS
// ============================================================================

fn bench_world_events(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordination_adversarial/world_events");

    // Test 1: Event creation
    group.bench_function("event_creation_1000", |bencher| {
        bencher.iter(|| {
            let events: Vec<WorldEvent> = (0..1000)
                .map(|i| WorldEvent {
                    id: i as u64,
                    event_type: match i % 5 {
                        0 => EventType::Combat,
                        1 => EventType::Discovery,
                        2 => EventType::Trade,
                        3 => EventType::Dialogue,
                        _ => EventType::Death,
                    },
                    position: [(i % 100) as f32, ((i / 100) % 100) as f32, 0.0],
                    timestamp: i as f64 * 0.1,
                    involved_agents: (0..(i % 5 + 1)).map(|j| AgentId((i * 10 + j) as u64)).collect(),
                })
                .collect();

            std_black_box(events.len())
        });
    });

    // Test 2: Event filtering by type
    group.bench_function("event_filtering_5000", |bencher| {
        let events: Vec<WorldEvent> = (0..5000)
            .map(|i| WorldEvent {
                id: i as u64,
                event_type: match i % 5 {
                    0 => EventType::Combat,
                    1 => EventType::Discovery,
                    2 => EventType::Trade,
                    3 => EventType::Dialogue,
                    _ => EventType::Death,
                },
                position: [0.0, 0.0, 0.0],
                timestamp: i as f64,
                involved_agents: vec![],
            })
            .collect();

        bencher.iter(|| {
            let combat_events: Vec<_> = events
                .iter()
                .filter(|e| matches!(e.event_type, EventType::Combat))
                .collect();

            std_black_box(combat_events.len())
        });
    });

    // Test 3: Spatial event query
    group.bench_function("spatial_event_query_1000", |bencher| {
        let events: Vec<WorldEvent> = (0..1000)
            .map(|i| WorldEvent {
                id: i as u64,
                event_type: EventType::Combat,
                position: [
                    (i % 100) as f32,
                    ((i / 100) % 100) as f32,
                    0.0,
                ],
                timestamp: 0.0,
                involved_agents: vec![],
            })
            .collect();

        let query_pos = [50.0f32, 50.0, 0.0];
        let query_radius = 20.0f32;

        bencher.iter(|| {
            let nearby: Vec<_> = events
                .iter()
                .filter(|e| {
                    let dx = e.position[0] - query_pos[0];
                    let dy = e.position[1] - query_pos[1];
                    let dist_sq = dx * dx + dy * dy;
                    dist_sq <= query_radius * query_radius
                })
                .collect();

            std_black_box(nearby.len())
        });
    });

    // Test 4: Event timeline ordering
    group.bench_function("timeline_ordering_2000", |bencher| {
        let mut events: Vec<WorldEvent> = (0..2000)
            .map(|i| WorldEvent {
                id: i as u64,
                event_type: EventType::Combat,
                position: [0.0, 0.0, 0.0],
                timestamp: ((i as f64 * 17.0) % 1000.0), // Scrambled timestamps
                involved_agents: vec![],
            })
            .collect();

        bencher.iter(|| {
            events.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
            std_black_box(events.first().map(|e| e.id))
        });
    });

    // Test 5: Agent involvement tracking
    group.bench_function("agent_involvement_tracking", |bencher| {
        let events: Vec<WorldEvent> = (0..1000)
            .map(|i| WorldEvent {
                id: i as u64,
                event_type: EventType::Combat,
                position: [0.0, 0.0, 0.0],
                timestamp: 0.0,
                involved_agents: (0..(i % 5 + 1))
                    .map(|j| AgentId(((i + j) % 100) as u64))
                    .collect(),
            })
            .collect();

        bencher.iter(|| {
            let mut agent_events: HashMap<AgentId, Vec<u64>> = HashMap::new();

            for event in &events {
                for agent in &event.involved_agents {
                    agent_events.entry(*agent).or_default().push(event.id);
                }
            }

            let busiest_agent = agent_events
                .iter()
                .max_by_key(|(_, events)| events.len())
                .map(|(id, _)| *id);

            std_black_box(busiest_agent)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 4: NARRATIVE COHERENCE
// ============================================================================

fn bench_narrative_coherence(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordination_adversarial/narrative_coherence");

    // Test 1: Thread creation
    group.bench_function("thread_creation_100", |bencher| {
        bencher.iter(|| {
            let threads: Vec<NarrativeThread> = (0..100)
                .map(|i| NarrativeThread {
                    id: i as u64,
                    name: format!("thread_{}", i),
                    participants: (0..5).map(|j| AgentId((i * 5 + j) as u64)).collect(),
                    events: (0..10).map(|j| (i * 10 + j) as u64).collect(),
                    coherence_score: 1.0,
                })
                .collect();

            std_black_box(threads.len())
        });
    });

    // Test 2: Coherence calculation
    group.bench_function("coherence_calculation_50", |bencher| {
        let threads: Vec<NarrativeThread> = (0..50)
            .map(|i| NarrativeThread {
                id: i as u64,
                name: format!("thread_{}", i),
                participants: (0..5).map(|j| AgentId((i * 5 + j) as u64)).collect(),
                events: (0..20).map(|j| (i * 20 + j) as u64).collect(),
                coherence_score: 1.0,
            })
            .collect();

        let events: Vec<WorldEvent> = (0..1000)
            .map(|i| WorldEvent {
                id: i as u64,
                event_type: match i % 5 {
                    0 => EventType::Combat,
                    1 => EventType::Discovery,
                    2 => EventType::Trade,
                    3 => EventType::Dialogue,
                    _ => EventType::Death,
                },
                position: [(i % 100) as f32, 0.0, 0.0],
                timestamp: i as f64,
                involved_agents: vec![AgentId((i % 250) as u64)],
            })
            .collect();

        let event_map: HashMap<u64, &WorldEvent> = events.iter().map(|e| (e.id, e)).collect();

        bencher.iter(|| {
            let scores: Vec<f32> = threads
                .iter()
                .map(|thread| {
                    let thread_events: Vec<_> = thread
                        .events
                        .iter()
                        .filter_map(|id| event_map.get(id))
                        .collect();

                    if thread_events.len() < 2 {
                        return 1.0;
                    }

                    // Calculate temporal coherence
                    let mut coherence = 0.0f32;
                    for window in thread_events.windows(2) {
                        let time_diff = (window[1].timestamp - window[0].timestamp).abs();
                        coherence += 1.0 / (1.0 + time_diff as f32 * 0.01);
                    }

                    coherence / (thread_events.len() - 1) as f32
                })
                .collect();

            let avg_coherence: f32 = scores.iter().sum::<f32>() / scores.len() as f32;
            std_black_box(avg_coherence)
        });
    });

    // Test 3: Thread merging
    group.bench_function("thread_merging_20", |bencher| {
        let threads: Vec<NarrativeThread> = (0..20)
            .map(|i| NarrativeThread {
                id: i as u64,
                name: format!("thread_{}", i),
                participants: (0..5).map(|j| AgentId(((i * 3 + j) % 50) as u64)).collect(), // Overlapping participants
                events: (0..10).map(|j| (i * 10 + j) as u64).collect(),
                coherence_score: 0.8,
            })
            .collect();

        bencher.iter(|| {
            // Find threads that can be merged (share participants)
            let mut merge_candidates: Vec<(usize, usize)> = Vec::new();

            for i in 0..threads.len() {
                for j in (i + 1)..threads.len() {
                    let shared: HashSet<_> = threads[i]
                        .participants
                        .iter()
                        .filter(|p| threads[j].participants.contains(p))
                        .collect();

                    if shared.len() >= 2 {
                        merge_candidates.push((i, j));
                    }
                }
            }

            std_black_box(merge_candidates.len())
        });
    });

    // Test 4: Conflict detection
    group.bench_function("conflict_detection_100", |bencher| {
        let threads: Vec<NarrativeThread> = (0..100)
            .map(|i| NarrativeThread {
                id: i as u64,
                name: format!("thread_{}", i),
                participants: (0..3).map(|j| AgentId((i * 3 + j) as u64)).collect(),
                events: vec![i as u64],
                coherence_score: 1.0,
            })
            .collect();

        let events: Vec<WorldEvent> = (0..100)
            .map(|i| WorldEvent {
                id: i as u64,
                event_type: if i % 2 == 0 { EventType::Combat } else { EventType::Trade },
                position: [(i % 10) as f32, 0.0, 0.0],
                timestamp: i as f64,
                involved_agents: vec![AgentId(i as u64), AgentId((i + 1) as u64)],
            })
            .collect();

        bencher.iter(|| {
            // Detect conflicting events (same agent in combat and trade at similar times)
            let mut conflicts = Vec::new();

            for event in &events {
                if matches!(event.event_type, EventType::Combat) {
                    for other in &events {
                        if event.id != other.id
                            && matches!(other.event_type, EventType::Trade)
                            && (event.timestamp - other.timestamp).abs() < 1.0
                        {
                            // Check for shared agents
                            for agent in &event.involved_agents {
                                if other.involved_agents.contains(agent) {
                                    conflicts.push((event.id, other.id, *agent));
                                }
                            }
                        }
                    }
                }
            }

            std_black_box(conflicts.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 5: MULTI-AGENT DECISION MAKING
// ============================================================================

fn bench_decision_making(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordination_adversarial/decision_making");

    // Test 1: Utility-based action selection
    group.bench_function("utility_action_selection_500", |bencher| {
        let agents = generate_agents(500);
        let actions = ["attack", "defend", "flee", "heal", "scout"];

        bencher.iter(|| {
            let decisions: Vec<(&str, f32)> = agents
                .iter()
                .map(|agent| {
                    let utilities: Vec<(usize, f32)> = actions
                        .iter()
                        .enumerate()
                        .map(|(i, _)| {
                            let utility = match i {
                                0 => 0.5 + agent.position[0] * 0.01, // attack
                                1 => 0.3 + agent.position[1] * 0.01, // defend
                                2 => 0.1 + (agent.id.0 as f32 % 10.0) * 0.05, // flee
                                3 => 0.4 - agent.position[0] * 0.005, // heal
                                _ => 0.2, // scout
                            };
                            (i, utility)
                        })
                        .collect();

                    let best = utilities.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
                    (actions[best.0], best.1)
                })
                .collect();

            let attack_count = decisions.iter().filter(|(a, _)| *a == "attack").count();
            std_black_box(attack_count)
        });
    });

    // Test 2: Consensus building
    group.bench_function("consensus_building_100_agents", |bencher| {
        let agents = generate_agents(100);
        let options = ["option_a", "option_b", "option_c"];

        bencher.iter(|| {
            // Each agent votes, weighted by role
            let mut votes: HashMap<&str, f32> = HashMap::new();

            for agent in &agents {
                let preferred = options[(agent.id.0 as usize) % options.len()];
                let weight = match agent.role {
                    AgentRole::Leader => 3.0,
                    AgentRole::Follower => 1.0,
                    AgentRole::Scout => 1.5,
                    AgentRole::Support => 1.5,
                    AgentRole::Attacker => 2.0,
                };

                *votes.entry(preferred).or_insert(0.0) += weight;
            }

            let winner = votes.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(k, _)| *k);
            std_black_box(winner)
        });
    });

    // Test 3: Coalition formation
    group.bench_function("coalition_formation_200", |bencher| {
        let agents = generate_agents(200);
        let graph = generate_social_graph(&agents);

        bencher.iter(|| {
            let mut coalitions: Vec<Vec<AgentId>> = Vec::new();
            let mut assigned: HashSet<AgentId> = HashSet::new();

            for agent in &agents {
                if assigned.contains(&agent.id) {
                    continue;
                }

                // Form coalition with friendly neighbors
                let mut coalition = vec![agent.id];
                assigned.insert(agent.id);

                if let Some(neighbors) = graph.get(&agent.id) {
                    for (neighbor, rel) in neighbors {
                        if *rel > 0.3 && !assigned.contains(neighbor) {
                            coalition.push(*neighbor);
                            assigned.insert(*neighbor);

                            if coalition.len() >= 5 {
                                break;
                            }
                        }
                    }
                }

                coalitions.push(coalition);
            }

            std_black_box(coalitions.len())
        });
    });

    // Test 4: Resource allocation
    group.bench_function("resource_allocation_50_agents", |bencher| {
        let agents = generate_agents(50);
        let total_resources = 1000u32;

        bencher.iter(|| {
            // Allocate based on role priority
            let mut priorities: Vec<(AgentId, u32)> = agents
                .iter()
                .map(|a| {
                    let priority = match a.role {
                        AgentRole::Leader => 5,
                        AgentRole::Attacker => 4,
                        AgentRole::Support => 3,
                        AgentRole::Scout => 2,
                        AgentRole::Follower => 1,
                    };
                    (a.id, priority)
                })
                .collect();

            priorities.sort_by_key(|(_, p)| std::cmp::Reverse(*p));

            let total_priority: u32 = priorities.iter().map(|(_, p)| *p).sum();
            let allocations: Vec<(AgentId, u32)> = priorities
                .iter()
                .map(|(id, p)| (*id, total_resources * p / total_priority))
                .collect();

            let allocated: u32 = allocations.iter().map(|(_, a)| *a).sum();
            std_black_box(allocated)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 6: COMMUNICATION SYSTEM
// ============================================================================

fn bench_communication(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordination_adversarial/communication");

    // Test 1: Message broadcasting
    group.bench_function("message_broadcast_1000", |bencher| {
        let agents = generate_agents(1000);
        let sender = agents[0].id;
        let message = "alert";

        bencher.iter(|| {
            let recipients: Vec<AgentId> = agents
                .iter()
                .filter(|a| a.id != sender && a.faction == agents[0].faction)
                .map(|a| a.id)
                .collect();

            std_black_box(recipients.len())
        });
    });

    // Test 2: Range-based communication
    group.bench_function("range_communication_500", |bencher| {
        let agents = generate_agents(500);
        let sender_pos = agents[0].position;
        let comm_range = 20.0f32;

        bencher.iter(|| {
            let in_range: Vec<AgentId> = agents
                .iter()
                .skip(1)
                .filter(|a| {
                    let dx = a.position[0] - sender_pos[0];
                    let dy = a.position[1] - sender_pos[1];
                    let dist_sq = dx * dx + dy * dy;
                    dist_sq <= comm_range * comm_range
                })
                .map(|a| a.id)
                .collect();

            std_black_box(in_range.len())
        });
    });

    // Test 3: Message queue processing
    group.bench_function("message_queue_processing_5000", |bencher| {
        let messages: Vec<(AgentId, AgentId, &str, f64)> = (0..5000)
            .map(|i| {
                (
                    AgentId((i % 100) as u64),
                    AgentId(((i + 1) % 100) as u64),
                    "update",
                    i as f64 * 0.001,
                )
            })
            .collect();

        bencher.iter(|| {
            let mut sorted = messages.clone();
            sorted.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());

            // Process in order
            let processed: Vec<_> = sorted
                .iter()
                .take(1000) // Process batch
                .collect();

            std_black_box(processed.len())
        });
    });

    // Test 4: Gossip protocol simulation
    group.bench_function("gossip_protocol_100_rounds", |bencher| {
        let agent_count = 100;
        let mut informed: HashSet<u64> = HashSet::new();
        informed.insert(0); // Initial informant

        bencher.iter(|| {
            let mut current_informed = informed.clone();

            for _ in 0..10 { // 10 rounds
                let newly_informed: Vec<u64> = current_informed
                    .iter()
                    .flat_map(|&informer| {
                        // Each informed agent tells 2 random others
                        vec![
                            (informer + 1) % agent_count as u64,
                            (informer * 7 + 3) % agent_count as u64,
                        ]
                    })
                    .collect();

                for id in newly_informed {
                    current_informed.insert(id);
                }
            }

            std_black_box(current_informed.len())
        });
    });

    group.finish();
}

// Simple deterministic "random" for benchmarks
fn rand_simple(max: u64) -> u64 {
    static mut SEED: u64 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        SEED % max
    }
}

criterion_group!(
    benches,
    bench_social_graph,
    bench_squad_coordination,
    bench_world_events,
    bench_narrative_coherence,
    bench_decision_making,
    bench_communication,
);

criterion_main!(benches);
