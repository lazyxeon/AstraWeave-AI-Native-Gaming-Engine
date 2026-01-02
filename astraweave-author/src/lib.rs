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
    let engine = Engine::new();
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
    let mut scope = rhai::Scope::new();
    let out: Dynamic = engine
        .call_fn(&mut scope, &ast, "configure", (m,))
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
    } else {
        Ok(serde_json::Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Dynamic;

    #[test]
    fn test_map_meta_creation() {
        let meta = MapMeta {
            width: 100,
            height: 100,
            enemy_count: 10,
            difficulty: 3,
        };
        assert_eq!(meta.width, 100);
        assert_eq!(meta.height, 100);
        assert_eq!(meta.enemy_count, 10);
        assert_eq!(meta.difficulty, 3);
    }

    #[test]
    fn test_map_meta_clone() {
        let meta = MapMeta {
            width: 50,
            height: 75,
            enemy_count: 5,
            difficulty: 2,
        };
        let cloned = meta.clone();
        assert_eq!(cloned.width, meta.width);
        assert_eq!(cloned.height, meta.height);
    }

    #[test]
    fn test_rhai_to_json_i64() {
        let d = Dynamic::from(42_i64);
        let json = rhai_to_json(&d).unwrap();
        assert_eq!(json, serde_json::Value::from(42));
    }

    #[test]
    fn test_rhai_to_json_f64() {
        let d = Dynamic::from(3.14_f64);
        let json = rhai_to_json(&d).unwrap();
        assert_eq!(json, serde_json::Value::from(3.14));
    }

    #[test]
    fn test_rhai_to_json_bool() {
        let d_true = Dynamic::from(true);
        let d_false = Dynamic::from(false);
        assert_eq!(rhai_to_json(&d_true).unwrap(), serde_json::Value::from(true));
        assert_eq!(rhai_to_json(&d_false).unwrap(), serde_json::Value::from(false));
    }

    #[test]
    fn test_rhai_to_json_string() {
        let d = Dynamic::from("hello world".to_string());
        let json = rhai_to_json(&d).unwrap();
        assert_eq!(json, serde_json::Value::from("hello world"));
    }

    #[test]
    fn test_rhai_to_json_null() {
        let d = Dynamic::UNIT;
        let json = rhai_to_json(&d).unwrap();
        assert_eq!(json, serde_json::Value::Null);
    }

    #[test]
    fn test_rhai_to_json_array() {
        let mut arr = rhai::Array::new();
        arr.push(Dynamic::from(1_i64));
        arr.push(Dynamic::from(2_i64));
        arr.push(Dynamic::from(3_i64));
        let d = Dynamic::from(arr);
        
        let json = rhai_to_json(&d).unwrap();
        let expected = serde_json::json!([1, 2, 3]);
        assert_eq!(json, expected);
    }

    #[test]
    fn test_rhai_to_json_map() {
        let mut m = rhai::Map::new();
        m.insert("name".into(), Dynamic::from("test".to_string()));
        m.insert("value".into(), Dynamic::from(42_i64));
        let d = Dynamic::from(m);
        
        let json = rhai_to_json(&d).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap(), &serde_json::Value::from("test"));
        assert_eq!(obj.get("value").unwrap(), &serde_json::Value::from(42));
    }

    #[test]
    fn test_rhai_to_json_nested() {
        let mut inner = rhai::Map::new();
        inner.insert("x".into(), Dynamic::from(10_i64));
        inner.insert("y".into(), Dynamic::from(20_i64));
        
        let mut outer = rhai::Map::new();
        outer.insert("position".into(), Dynamic::from(inner));
        outer.insert("active".into(), Dynamic::from(true));
        let d = Dynamic::from(outer);
        
        let json = rhai_to_json(&d).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("position"));
        assert!(obj.contains_key("active"));
        
        let pos = obj.get("position").unwrap().as_object().unwrap();
        assert_eq!(pos.get("x").unwrap(), &serde_json::Value::from(10));
    }

    #[test]
    fn test_director_budget_from_defaults() {
        // Test that DirectorBudget has expected defaults
        let budget = astraweave_core::DirectorBudget {
            traps: 1,
            terrain_edits: 2,
            spawns: 1,
        };
        assert_eq!(budget.traps, 1);
        assert_eq!(budget.terrain_edits, 2);
        assert_eq!(budget.spawns, 1);
    }

    #[test]
    fn test_map_meta_different_difficulties() {
        for difficulty in 1..=5 {
            let meta = MapMeta {
                width: 100 + difficulty * 10,
                height: 100 + difficulty * 10,
                enemy_count: difficulty * 5,
                difficulty,
            };
            assert_eq!(meta.difficulty, difficulty);
            assert!(meta.width > 100);
            assert!(meta.enemy_count >= 5);
        }
    }
}
