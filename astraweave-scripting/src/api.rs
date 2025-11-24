use rhai::{Engine, ImmutableString};
use glam::{Vec3, IVec2, Quat};

#[derive(Clone, Debug)]
pub enum ScriptCommand {
    Spawn { prefab: String, position: Vec3 },
    Despawn { entity: i64 },
    SetPosition { entity: i64, position: Vec3 },
}

#[derive(Clone, Debug, Default)]
pub struct ScriptCommands {
    pub commands: Vec<ScriptCommand>,
}

impl ScriptCommands {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, prefab: &str, position: Vec3) {
        self.commands.push(ScriptCommand::Spawn {
            prefab: prefab.to_string(),
            position,
        });
    }
    
    pub fn despawn(&mut self, entity: i64) {
        self.commands.push(ScriptCommand::Despawn { entity });
    }

    pub fn set_position(&mut self, entity: i64, position: Vec3) {
        self.commands.push(ScriptCommand::SetPosition { entity, position });
    }
}

pub fn register_api(engine: &mut Engine) {
    // Register Vec3
    engine.register_type_with_name::<Vec3>("Vec3")
        .register_fn("vec3", |x: f32, y: f32, z: f32| Vec3::new(x, y, z))
        .register_get("x", |v: &mut Vec3| v.x)
        .register_get("y", |v: &mut Vec3| v.y)
        .register_get("z", |v: &mut Vec3| v.z)
        .register_set("x", |v: &mut Vec3, val: f32| v.x = val)
        .register_set("y", |v: &mut Vec3, val: f32| v.y = val)
        .register_set("z", |v: &mut Vec3, val: f32| v.z = val)
        .register_fn("+", |a: Vec3, b: Vec3| a + b)
        .register_fn("-", |a: Vec3, b: Vec3| a - b)
        .register_fn("*", |a: Vec3, f: f32| a * f)
        .register_fn("to_string", |v: Vec3| ImmutableString::from(format!("Vec3({}, {}, {})", v.x, v.y, v.z)));

    // Register IVec2
    engine.register_type_with_name::<IVec2>("IVec2")
        .register_fn("ivec2", |x: i64, y: i64| IVec2::new(x as i32, y as i32))
        .register_get("x", |v: &mut IVec2| v.x as i64)
        .register_get("y", |v: &mut IVec2| v.y as i64)
        .register_set("x", |v: &mut IVec2, val: i64| v.x = val as i32)
        .register_set("y", |v: &mut IVec2, val: i64| v.y = val as i32)
        .register_fn("+", |a: IVec2, b: IVec2| a + b)
        .register_fn("-", |a: IVec2, b: IVec2| a - b)
        .register_fn("to_string", |v: IVec2| ImmutableString::from(format!("IVec2({}, {})", v.x, v.y)));

    // Register Quat
    engine.register_type_with_name::<Quat>("Quat")
        .register_fn("quat", |x: f32, y: f32, z: f32, w: f32| Quat::from_xyzw(x, y, z, w))
        .register_fn("to_string", |v: Quat| ImmutableString::from(format!("Quat({}, {}, {}, {})", v.x, v.y, v.z, v.w)));

    // Register Global Functions
    engine.register_fn("log", |s: ImmutableString| println!("[Script] {}", s));

    // Register ScriptCommands
    engine.register_type_with_name::<ScriptCommands>("Commands")
        .register_fn("spawn_prefab", |cmds: &mut ScriptCommands, prefab: &str, pos: Vec3| cmds.spawn(prefab, pos))
        .register_fn("despawn", |cmds: &mut ScriptCommands, entity: i64| cmds.despawn(entity))
        .register_fn("set_position", |cmds: &mut ScriptCommands, entity: i64, pos: Vec3| cmds.set_position(entity, pos));
}
