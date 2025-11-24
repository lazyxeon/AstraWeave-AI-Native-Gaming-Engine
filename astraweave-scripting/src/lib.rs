use astraweave_ecs::{App, Plugin, SystemStage, World, Entity};
use astraweave_core::{CPos, IVec2, CHealth};
use astraweave_physics::{PhysicsWorld, CollisionEvent, ContactForceEvent, Layers};
use rhai::{Engine, Scope, AST, Dynamic};
use std::collections::HashMap;
use std::sync::Arc;
use api::{ScriptCommands, ScriptCommand};
use glam::Vec3;
use std::time::SystemTime;

pub mod loader;
pub mod api;

#[derive(Clone, Copy, Debug)]
pub struct CPhysicsBody {
    pub body_id: u64,
}

/// Rhai script attached to an ECS entity
#[derive(Clone)]
pub struct CScript {
    pub script_path: String,
    /// The actual script content. If empty, system may try to load from path (not implemented in MVP).
    pub source: String,
    pub cached_ast: Option<Arc<AST>>,
    /// Persistent variables maintained across ticks
    pub script_state: HashMap<String, Dynamic>,
    pub enabled: bool,
}

impl CScript {
    pub fn new(path: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            script_path: path.into(),
            source: source.into(),
            cached_ast: None,
            script_state: HashMap::new(),
            enabled: true,
        }
    }
}

/// Resource wrapping the Rhai engine
#[derive(Clone)]
pub struct ScriptEngineResource(pub Arc<Engine>);

impl Default for ScriptEngineResource {
    fn default() -> Self {
        let mut engine = Engine::new();
        
        // Security: Configure limits
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(50_000); // Limit operations per run
        engine.set_max_string_size(1024); // Limit string size
        engine.set_max_array_size(1024); // Limit array size
        engine.set_max_map_size(1024); // Limit map size
        
        // Register standard packages
        // engine.register_global_module(rhai::packages::Package::new().as_shared_module());
        
        // Register AstraWeave API
        api::register_api(&mut engine);

        Self(Arc::new(engine))
    }
}

fn spawn_prefab(world: &mut World, prefab: &str, position: Vec3) -> Entity {
    match prefab {
        "enemy_grunt" => {
            let e = world.spawn();
            world.insert(e, CPos { pos: IVec2::new(position.x as i32, position.z as i32) });
            world.insert(e, CHealth { hp: 100 });
            println!("[ScriptSystem] Spawning enemy_grunt at {} (Entity: {})", position, e);
            e
        },
        "crate" => {
            let e = world.spawn();
            world.insert(e, CPos { pos: IVec2::new(position.x as i32, position.z as i32) });
            println!("[ScriptSystem] Spawning crate at {} (Entity: {})", position, e);
            e
        },
        _ => {
            println!("[ScriptSystem] Unknown prefab: {}", prefab);
            // Spawn a default entity so we don't crash, but log error
            let e = world.spawn();
            world.insert(e, CPos { pos: IVec2::new(position.x as i32, position.z as i32) });
            e
        }
    }
}

/// System to execute entity scripts
pub fn script_system(world: &mut World) {
    // 1. Get the engine (cheap clone of Arc)
    let engine = if let Some(res) = world.get_resource::<ScriptEngineResource>() {
        res.0.clone()
    } else {
        return;
    };

    // Take cache out to avoid borrow conflicts
    let mut cache = if let Some(c) = world.get_resource_mut::<ScriptCache>() {
        std::mem::take(c)
    } else {
        ScriptCache::default()
    };

    let mut all_commands = Vec::new();

    // 2. Run Scripts (Main Body)
    let entities = world.entities_with::<CScript>();
    for entity in entities {
        // We need to re-acquire the component mutably
        if let Some(mut script) = world.get_mut::<CScript>(entity) {
            if !script.enabled { continue; }

            // Hot Reloading
            if !script.script_path.is_empty() {
                if let Ok(metadata) = std::fs::metadata(&script.script_path) {
                    if let Ok(modified) = metadata.modified() {
                        let needs_reload = if let Some(cached) = cache.scripts.get(&script.script_path) {
                            modified > cached.last_modified
                        } else {
                            true
                        };

                        if needs_reload {
                            // println!("[ScriptSystem] Reloading script: {}", script.script_path);
                            if let Ok(source) = std::fs::read_to_string(&script.script_path) {
                                match engine.compile(&source) {
                                    Ok(ast) => {
                                        let ast = Arc::new(ast);
                                        cache.scripts.insert(script.script_path.clone(), CachedScript {
                                            ast: ast.clone(),
                                            last_modified: modified,
                                        });
                                        script.cached_ast = Some(ast);
                                        script.source = source;
                                    }
                                    Err(e) => {
                                        eprintln!("Script compilation error for {}: {}", script.script_path, e);
                                    }
                                }
                            }
                        } else {
                            // Use cached AST if we don't have one
                            if script.cached_ast.is_none() {
                                if let Some(cached) = cache.scripts.get(&script.script_path) {
                                    script.cached_ast = Some(cached.ast.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Compile if needed (inline scripts or failed load)
            if script.cached_ast.is_none() {
                if script.source.is_empty() {
                    continue;
                }
                
                match engine.compile(&script.source) {
                    Ok(ast) => script.cached_ast = Some(Arc::new(ast)),
                    Err(e) => {
                        eprintln!("Script compilation error for {}: {}", script.script_path, e);
                        script.enabled = false; // Disable failing script
                        continue;
                    }
                }
            }

            // Execute
            if let Some(ast) = &script.cached_ast {
                let mut scope = Scope::new();
                
                // Push state to scope
                for (k, v) in &script.script_state {
                    scope.push_dynamic(k.clone(), v.clone());
                }
                
                // Inject Entity ID (raw u64 cast to i64 for Rhai)
                scope.push("entity_id", entity.to_raw() as i64);
                
                // Inject Commands
                let commands = ScriptCommands::new();
                scope.push("commands", commands);
                
                // Run the script
                if let Err(e) = engine.run_ast_with_scope(&mut scope, ast) {
                     eprintln!("Script runtime error for {}: {}", script.script_path, e);
                }
                
                // Update state from scope
                for (k, _, v) in scope.iter() {
                    if k == "commands" { continue; } // Don't save commands to state
                    script.script_state.insert(k.to_string(), v);
                }

                // Extract commands
                if let Some(cmds) = scope.get_value::<ScriptCommands>("commands") {
                    all_commands.extend(cmds.commands);
                }
            }
        }
    }

    // Put cache back
    world.insert_resource(cache);

    // 3. Process Physics Events
    // Build BodyId -> Entity map
    let mut body_to_entity = HashMap::new();
    for e in world.entities_with::<CPhysicsBody>() {
        if let Some(body) = world.get::<CPhysicsBody>(e) {
            body_to_entity.insert(body.body_id, e);
        }
    }

    // Collect events
    let mut callbacks = Vec::new();
    if let Some(physics) = world.get_resource::<PhysicsWorld>() {
        while let Ok(event) = physics.collision_recv.try_recv() {
            match event {
                CollisionEvent::Started(h1, h2, _) => {
                    let h1_body = physics.colliders.get(h1).and_then(|c| c.parent());
                    let h2_body = physics.colliders.get(h2).and_then(|c| c.parent());

                    let id1 = h1_body.and_then(|h| physics.id_of(h));
                    let id2 = h2_body.and_then(|h| physics.id_of(h));
                    
                    if let (Some(bid1), Some(bid2)) = (id1, id2) {
                        if let (Some(e1), Some(e2)) = (body_to_entity.get(&bid1), body_to_entity.get(&bid2)) {
                            callbacks.push((*e1, "on_collision", *e2));
                            callbacks.push((*e2, "on_collision", *e1));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Execute callbacks
    for (entity, callback, other) in callbacks {
        if let Some(script) = world.get_mut::<CScript>(entity) {
             if !script.enabled || script.cached_ast.is_none() { continue; }
             let ast = script.cached_ast.clone().unwrap();
             
             let mut scope = Scope::new();
             for (k, v) in &script.script_state { scope.push_dynamic(k.clone(), v.clone()); }
             scope.push("entity_id", entity.to_raw() as i64);
             scope.push("other_entity_id", other.to_raw() as i64);
             scope.push("commands", ScriptCommands::new());
             
             // Call function
             let other_id = other.to_raw() as i64;
             let result: Result<Dynamic, _> = engine.call_fn(&mut scope, &ast, callback, (other_id,));
             
             if let Err(e) = result {
                 // Only log if it's not a "function not found" error, which is common if script doesn't implement callback
                 // But Rhai returns EvalAltResult::ErrorFunctionNotFound.
                 // For now, we can just log everything as debug/info or ignore.
                 // Let's ignore for now to avoid spam if script doesn't have on_collision.
             }
             
             // Update state
             for (k, _, v) in scope.iter() {
                if k == "commands" { continue; }
                script.script_state.insert(k.to_string(), v);
            }
            if let Some(cmds) = scope.get_value::<ScriptCommands>("commands") {
                all_commands.extend(cmds.commands);
            }
        }
    }

    // 4. Execute commands
    for cmd in all_commands {
        match cmd {
            ScriptCommand::Spawn { prefab, position } => {
                let e = spawn_prefab(world, &prefab, position);
                // Add physics body
                if let Some(mut physics) = world.get_resource_mut::<PhysicsWorld>() {
                    let body_id = physics.add_dynamic_box(position, Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT);
                    world.insert(e, CPhysicsBody { body_id });
                }
            }
            ScriptCommand::Despawn { entity } => {
                let e = unsafe { Entity::from_raw(entity as u64) };
                if world.despawn(e) {
                    println!("[ScriptSystem] Despawned entity {}", e);
                } else {
                    println!("[ScriptSystem] Failed to despawn entity {} (not found or dead)", e);
                }
            }
            ScriptCommand::SetPosition { entity, position } => {
                let e = unsafe { Entity::from_raw(entity as u64) };
                if let Some(cpos) = world.get_mut::<CPos>(e) {
                    cpos.pos = IVec2::new(position.x as i32, position.z as i32);
                    println!("[ScriptSystem] Set position of {} to {}", e, position);
                } else {
                    println!("[ScriptSystem] Entity {} has no CPos component", e);
                }
                
                // Update physics
                let body_id = world.get::<CPhysicsBody>(e).map(|b| b.body_id);
                if let Some(bid) = body_id {
                    if let Some(mut physics) = world.get_resource_mut::<PhysicsWorld>() {
                         physics.set_body_position(bid, position);
                    }
                }
            }
        }
    }
}

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.world.insert_resource(ScriptEngineResource::default());
        app.world.insert_resource(ScriptCache::default());
        app.add_system(SystemStage::SIMULATION, script_system);
    }
}

pub struct CachedScript {
    pub ast: Arc<AST>,
    pub last_modified: SystemTime,
}

pub struct ScriptCache {
    pub scripts: HashMap<String, CachedScript>,
}

impl Default for ScriptCache {
    fn default() -> Self {
        Self { scripts: HashMap::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_execution() {
        let mut app = App::new();
        app = app.add_plugin(ScriptingPlugin);
        
        let e = app.world.spawn();
        let script_source = r#"
            let x = 10;
            let y = 20;
            result = x + y;
        "#;
        
        let mut script = CScript::new("test.rhai", script_source);
        script.script_state.insert("result".to_string(), Dynamic::from(0_i64));
        app.world.insert(e, script);
        
        // Run one tick
        app.schedule.run(&mut app.world);
        
        // Check result
        let script = app.world.get::<CScript>(e).unwrap();
        let result = script.script_state.get("result").unwrap().as_int().unwrap();
        assert_eq!(result, 30);
    }

    #[tokio::test]
    async fn test_script_loading_and_execution() {
        use std::path::PathBuf;
        
        let mut app = App::new();
        app = app.add_plugin(ScriptingPlugin);
        
        let asset_path = PathBuf::from("tests/assets/test_script.rhai");
        // Ensure we are running from crate root
        let asset = loader::ScriptLoader::load(&asset_path).await.expect("Failed to load script");
        
        let e = app.world.spawn();
        let mut script = CScript::new(asset.path.to_string_lossy(), asset.source);
        script.script_state.insert("result".to_string(), Dynamic::from(0_i64));
        app.world.insert(e, script);
        
        // Run one tick
        app.schedule.run(&mut app.world);
        
        // Check result
        let script = app.world.get::<CScript>(e).unwrap();
        let result = script.script_state.get("result").unwrap().as_int().unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_api_integration() {
        use std::path::PathBuf;
        use glam::IVec2;
        
        let mut app = App::new();
        app = app.add_plugin(ScriptingPlugin);
        
        let asset_path = PathBuf::from("tests/assets/test_api.rhai");
        let asset = loader::ScriptLoader::load(&asset_path).await.expect("Failed to load script");
        
        let e = app.world.spawn();
        let mut script = CScript::new(asset.path.to_string_lossy(), asset.source);
        // Initialize result_pos to verify it gets updated
        script.script_state.insert("result_pos".to_string(), Dynamic::from(IVec2::new(0, 0)));
        app.world.insert(e, script);
        
        // Run one tick
        app.schedule.run(&mut app.world);
        
        // Check result
        let script = app.world.get::<CScript>(e).unwrap();
        let result = script.script_state.get("result_pos").unwrap().clone().cast::<IVec2>();
        assert_eq!(result, IVec2::new(11, 21));
    }

    #[tokio::test]
    async fn test_ecs_commands() {
        use std::path::PathBuf;
        
        let mut app = App::new();
        app = app.add_plugin(ScriptingPlugin);
        
        let asset_path = PathBuf::from("tests/assets/test_ecs.rhai");
        let asset = loader::ScriptLoader::load(&asset_path).await.expect("Failed to load script");
        
        let e = app.world.spawn();
        let script = CScript::new(asset.path.to_string_lossy(), asset.source);
        app.world.insert(e, script);
        
        // Run one tick
        // We expect output to stdout: [ScriptSystem] Spawning ...
        app.schedule.run(&mut app.world);
        
        // Since we only print for now, we assume success if no panic and script runs.
        // In a real test, we would mock the command execution or check side effects.
    }

    #[test]
    fn test_set_position_command() {
        use astraweave_core::{CPos, IVec2};

        let mut app = App::new();
        app = app.add_plugin(ScriptingPlugin);

        // Spawn entity with initial position
        let e = app.world.spawn();
        app.world.insert(e, CPos { pos: IVec2::new(0, 0) });

        // Create script to move it
        // Note: We need to pass the entity ID to the script.
        // The script_system injects 'entity_id' automatically.
        let script_source = r#"
            let target = vec3(10.0, 0.0, 20.0);
            commands.set_position(entity_id, target);
        "#;

        let script = CScript::new("test_move.rhai", script_source);
        app.world.insert(e, script);

        // Run one tick
        app.schedule.run(&mut app.world);

        // Check if position was updated
        // Note: IVec2 uses (x, y), we map Vec3(x, y, z) -> IVec2(x, z)
        let pos = app.world.get::<CPos>(e).unwrap().pos;
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    }

    #[test]
    fn test_spawn_prefab() {
        use astraweave_core::{CPos, CHealth};

        let mut app = App::new();
        app = app.add_plugin(ScriptingPlugin);

        let e = app.world.spawn();
        let script_source = r#"
            let pos = vec3(5.0, 0.0, 5.0);
            commands.spawn_prefab("enemy_grunt", pos);
        "#;

        let script = CScript::new("test_spawn.rhai", script_source);
        app.world.insert(e, script);

        // Run one tick
        app.schedule.run(&mut app.world);

        // Check if entity was spawned
        // We iterate all entities and check for one with CHealth and CPos at (5, 5)
        let mut found = false;
        let entities = app.world.entities_with::<CHealth>();
        for entity in entities {
            if let Some(pos) = app.world.get::<CPos>(entity) {
                if pos.pos.x == 5 && pos.pos.y == 5 {
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "Spawned enemy_grunt not found");
    }

    #[test]
    fn test_collision_event() {
        use astraweave_physics::{PhysicsWorld, Layers};
        use glam::Vec3;

        let mut app = App::new();
        app = app.add_plugin(ScriptingPlugin);
        
        // Add PhysicsWorld resource
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        app.world.insert_resource(physics);

        // Spawn entity 1 (Listener)
        let e1 = app.world.spawn();
        let script_source = r#"
            fn on_collision(other) {
                collided_with = other;
            }
        "#;
        let mut script = CScript::new("test_collision.rhai", script_source);
        script.script_state.insert("collided_with".to_string(), Dynamic::from(0_i64));
        app.world.insert(e1, script);

        // Add physics body to e1
        // We need to drop the resource borrow before inserting components? 
        // No, app.world.get_resource_mut returns a RefMut, which borrows world.
        // We can't insert components while holding it.
        
        let body_id1 = {
            let mut physics = app.world.get_resource_mut::<PhysicsWorld>().unwrap();
            physics.add_dynamic_box(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT)
        };
        app.world.insert(e1, CPhysicsBody { body_id: body_id1 });

        // Spawn entity 2 (Collider)
        let e2 = app.world.spawn();
        let body_id2 = {
            let mut physics = app.world.get_resource_mut::<PhysicsWorld>().unwrap();
            physics.add_dynamic_box(Vec3::new(0.8, 0.0, 0.0), Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT)
        };
        app.world.insert(e2, CPhysicsBody { body_id: body_id2 });

        // Step physics to generate collision
        {
            let mut physics = app.world.get_resource_mut::<PhysicsWorld>().unwrap();
            for _ in 0..10 {
                physics.step();
            }
        }

        // Run script system to process events
        app.schedule.run(&mut app.world);

        // Check result
        let script = app.world.get::<CScript>(e1).unwrap();
        let collided_with = script.script_state.get("collided_with").unwrap().as_int().unwrap();
        
        // e2 entity ID
        let e2_id = e2.to_raw() as i64;
        assert_eq!(collided_with, e2_id);
    }
}
