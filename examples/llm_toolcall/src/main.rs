use astraweave_core::*;
use astraweave_llm::{plan_from_llm, MockLlm};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tiny snapshot
    let snap = WorldSnapshot {
        t: 1.0,
        player: PlayerState {
            hp: 100,
            pos: IVec2 { x: 2, y: 2 },
            stance: "stand".into(),
            orders: vec![],
        },
        me: CompanionState {
            ammo: 30,
            cooldowns: Default::default(),
            morale: 0.9,
            pos: IVec2 { x: 3, y: 2 },
        },
        enemies: vec![EnemyState {
            id: 99,
            pos: IVec2 { x: 12, y: 2 },
            hp: 60,
            cover: "low".into(),
            last_seen: 1.0,
        }],
        pois: vec![],
        objective: Some("extract".into()),
    };
    let reg = ToolRegistry {
        tools: vec![
            ToolSpec {
                name: "move_to".into(),
                args: [("x", "i32"), ("y", "i32")]
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            },
            ToolSpec {
                name: "throw".into(),
                args: [("item", "enum[smoke,grenade]"), ("x", "i32"), ("y", "i32")]
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            },
            ToolSpec {
                name: "cover_fire".into(),
                args: [("target_id", "u32"), ("duration", "f32")]
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            },
        ],
        constraints: Constraints {
            enforce_cooldowns: true,
            enforce_los: true,
            enforce_stamina: true,
        },
    };
    let client = MockLlm;
    let plan = plan_from_llm(&client, &snap, &reg).await?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(())
}
