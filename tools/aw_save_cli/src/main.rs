use anyhow::Result;
use aw_save::{SaveBundleV2, SaveManager, WorldState, PlayerInventory, ItemStack, CompanionProfile, SAVE_SCHEMA_VERSION};
use clap::{Parser, Subcommand};
use time::OffsetDateTime;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name="aw_save_cli", version, about="AstraWeave save/loader & migration utility")]
struct Cli {
    /// Save root directory (e.g., ./saves)
    #[arg(long, default_value = "./saves")]
    root: PathBuf,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List saves for a player
    List { player: String },
    /// Verify a specific save file (CRC/version/decode)
    Verify { file: PathBuf },
    /// Migrate a specific save file to latest version (in place)
    Migrate { file: PathBuf },
    /// Write a demo save into a slot (for smoke tests)
    DemoSave { player: String, #[arg(default_value_t=0)] slot: u8 },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::List { player } => {
            let sm = SaveManager::new(cli.root);
            for m in sm.list_saves(&player)? {
                println!("[slot {:02}] {}  id={}  schema={}", m.slot, m.created_at, m.save_id, m.schema);
            }
        }
        Cmd::Verify { file } => {
            let sm = SaveManager::new(cli.root);
            let v2 = sm.migrate_file_to_latest(&file, false)?;
            println!("OK: schema={} id={} player={} slot={}", v2.schema, v2.save_id, v2.player_id, v2.slot);
        }
        Cmd::Migrate { file } => {
            let sm = SaveManager::new(cli.root);
            let v2 = sm.migrate_file_to_latest(&file, true)?;
            println!("Migrated: schema={} id={}", v2.schema, v2.save_id);
        }
        Cmd::DemoSave { player, slot } => {
            let sm = SaveManager::new(cli.root);
            let bundle = SaveBundleV2 {
                schema: SAVE_SCHEMA_VERSION,
                save_id: Uuid::new_v4(),
                created_at: OffsetDateTime::now_utc(),
                player_id: player.clone(),
                slot,
                world: WorldState { tick: 123, ecs_blob: vec![1,2,3,4], state_hash: 0xdead_beef_cafe_babe },
                inventory: PlayerInventory {
                    credits: 777,
                    items: vec![
                        ItemStack { kind: "medkit".into(), qty: 3, attrs: Default::default() },
                        ItemStack { kind: "battery".into(), qty: 5, attrs: Default::default() },
                    ],
                },
                companions: vec![
                    CompanionProfile { id: "c1".into(), name: "Echo".into(), level: 4, skills: vec!["Scan".into()], facts: vec![], episodes_summarized: vec![] }
                ],
                meta: Default::default(),
            };
            let path = sm.save(&player, slot, bundle)?;
            println!("Wrote {}", path.display());
        }
    }
    Ok(())
}