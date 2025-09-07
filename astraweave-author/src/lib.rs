use anyhow::Result;
use astraweave_core::DirectorBudget;
use rhai::{Dynamic, Engine, Map};

#[derive(Clone)]
pub struct MapMeta {
    pub width: i32,
    pub height: i32,
    pub enemy_count: i32,
    pub difficulty: i32, // 1..5
}

pub fn run_author_script(
    path: &str,
    meta: &MapMeta,
) -> Result<(DirectorBudget, serde_json::Value)> {
    let mut engine = Engine::new();
    // Provide meta as a map
    let mut m = Map::new();
    m.insert("width".into(), Dynamic::from(meta.width));
    m.insert("height".into(), Dynamic::from(meta.height));
    m.insert("enemy_count".into(), Dynamic::from(meta.enemy_count));
    m.insert("difficulty".into(), Dynamic::from(meta.difficulty));

    let ast = engine
        .compile_file(path.into())
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    // `configure(meta)` returns object `{ traps, terrain_edits, spawns, hints: #{...} }`
    let out: Dynamic = engine
        .call_fn(&rhai::Scope::new(), &ast, "configure", (m,))
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let o: rhai::Map = out.cast();

    let traps = o
        .get("traps")
        .and_then(|d| d.clone().try_cast::<i64>())
        .unwrap_or(1) as i32;
    let terrain = o
        .get("terrain_edits")
        .and_then(|d| d.clone().try_cast::<i64>())
        .unwrap_or(2) as i32;
    let spawns = o
        .get("spawns")
        .and_then(|d| d.clone().try_cast::<i64>())
        .unwrap_or(1) as i32;

    // Hints map -> JSON
    let hints_dyn = o
        .get("hints")
        .cloned()
        .unwrap_or(Dynamic::from(rhai::Map::new()));
    let hints_json = rhai_to_json(&hints_dyn)?;

    Ok((
        DirectorBudget {
            traps,
            terrain_edits: terrain,
            spawns,
        },
        hints_json,
    ))
}

fn rhai_to_json(d: &rhai::Dynamic) -> Result<serde_json::Value> {
    if d.is::<rhai::Map>() {
        let m: rhai::Map = d.clone().cast();
        let mut out = serde_json::Map::new();
        for (k, v) in m {
            out.insert(k.into(), rhai_to_json(&v)?);
        }
        Ok(serde_json::Value::Object(out))
    } else if d.is::<rhai::Array>() {
        let arr: rhai::Array = d.clone().cast();
        let mut out = vec![];
        for v in arr {
            out.push(rhai_to_json(&v)?);
        }
        Ok(serde_json::Value::Array(out))
    } else if let Some(i) = d.clone().try_cast::<i64>() {
        Ok(serde_json::Value::from(i))
    } else if let Some(f) = d.clone().try_cast::<f64>() {
        Ok(serde_json::Value::from(f))
    } else if let Some(b) = d.clone().try_cast::<bool>() {
        Ok(serde_json::Value::from(b))
    } else if let Some(s) = d.clone().try_cast::<String>() {
        Ok(serde_json::Value::from(s))
    } else if d.is_unit() {
        Ok(serde_json::Value::Null)
    } else {
        Ok(serde_json::Value::Null)
    }
}
