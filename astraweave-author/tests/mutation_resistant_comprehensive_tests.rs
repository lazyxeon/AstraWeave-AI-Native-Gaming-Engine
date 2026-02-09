//! Mutation-resistant comprehensive tests for astraweave-author.
//!
//! Targets: MapMeta (construction, clone, field access),
//! run_author_script (with a real Rhai script file).

use astraweave_author::MapMeta;
use std::io::Write;

// =========================================================================
// MapMeta — field access, clone, boundary values
// =========================================================================

#[test]
fn map_meta_fields() {
    let m = MapMeta {
        width: 100,
        height: 200,
        enemy_count: 10,
        difficulty: 3,
    };
    assert_eq!(m.width, 100);
    assert_eq!(m.height, 200);
    assert_eq!(m.enemy_count, 10);
    assert_eq!(m.difficulty, 3);
}

#[test]
fn map_meta_clone_preserves_all_fields() {
    let m = MapMeta {
        width: 50,
        height: 75,
        enemy_count: 5,
        difficulty: 1,
    };
    let c = m.clone();
    assert_eq!(c.width, 50);
    assert_eq!(c.height, 75);
    assert_eq!(c.enemy_count, 5);
    assert_eq!(c.difficulty, 1);
}

#[test]
fn map_meta_clone_independent() {
    let m = MapMeta {
        width: 10,
        height: 20,
        enemy_count: 3,
        difficulty: 2,
    };
    let mut c = m.clone();
    c.width = 999;
    // Original unchanged
    assert_eq!(m.width, 10);
    assert_eq!(c.width, 999);
}

#[test]
fn map_meta_zero_values() {
    let m = MapMeta {
        width: 0,
        height: 0,
        enemy_count: 0,
        difficulty: 0,
    };
    assert_eq!(m.width, 0);
    assert_eq!(m.height, 0);
    assert_eq!(m.enemy_count, 0);
    assert_eq!(m.difficulty, 0);
}

#[test]
fn map_meta_negative_values() {
    let m = MapMeta {
        width: -1,
        height: -1,
        enemy_count: -5,
        difficulty: -1,
    };
    assert_eq!(m.width, -1);
    assert_eq!(m.enemy_count, -5);
}

#[test]
fn map_meta_max_difficulty() {
    let m = MapMeta {
        width: 100,
        height: 100,
        enemy_count: 50,
        difficulty: 5,
    };
    assert_eq!(m.difficulty, 5);
}

#[test]
fn map_meta_min_difficulty() {
    let m = MapMeta {
        width: 100,
        height: 100,
        enemy_count: 1,
        difficulty: 1,
    };
    assert_eq!(m.difficulty, 1);
}

// =========================================================================
// run_author_script — with a real Rhai file
// =========================================================================

#[test]
fn run_author_script_simple() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = dir.path().join("test.rhai");
    let mut f = std::fs::File::create(&script_path).unwrap();
    write!(
        f,
        r#"
fn configure(meta) {{
    let traps = meta.difficulty * 2;
    let terrain_edits = meta.width / 10;
    let spawns = meta.enemy_count;
    let hints = #{{"tip": "use cover"}};
    #{{
        traps: traps,
        terrain_edits: terrain_edits,
        spawns: spawns,
        hints: hints
    }}
}}
"#
    )
    .unwrap();

    let meta = MapMeta {
        width: 100,
        height: 50,
        enemy_count: 8,
        difficulty: 3,
    };
    let (budget, hints) =
        astraweave_author::run_author_script(script_path.to_str().unwrap(), &meta).unwrap();

    assert_eq!(budget.traps, 6); // difficulty(3) * 2
    assert_eq!(budget.terrain_edits, 10); // width(100) / 10
    assert_eq!(budget.spawns, 8); // enemy_count
    assert_eq!(hints["tip"], "use cover");
}

#[test]
fn run_author_script_defaults_on_missing_keys() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = dir.path().join("minimal.rhai");
    let mut f = std::fs::File::create(&script_path).unwrap();
    // Return empty map — defaults: traps=1, terrain_edits=2, spawns=1
    write!(
        f,
        r#"
fn configure(meta) {{
    #{{}}
}}
"#
    )
    .unwrap();

    let meta = MapMeta {
        width: 10,
        height: 10,
        enemy_count: 0,
        difficulty: 1,
    };
    let (budget, hints) =
        astraweave_author::run_author_script(script_path.to_str().unwrap(), &meta).unwrap();

    assert_eq!(budget.traps, 1); // default
    assert_eq!(budget.terrain_edits, 2); // default
    assert_eq!(budget.spawns, 1); // default
    assert!(hints.is_object()); // empty hints map → empty JSON object
}

#[test]
fn run_author_script_nonexistent_file_errors() {
    let meta = MapMeta {
        width: 10,
        height: 10,
        enemy_count: 0,
        difficulty: 1,
    };
    let result = astraweave_author::run_author_script("/nonexistent/path.rhai", &meta);
    assert!(result.is_err());
}

#[test]
fn run_author_script_meta_fields_passed_correctly() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = dir.path().join("echo.rhai");
    let mut f = std::fs::File::create(&script_path).unwrap();
    // Echo back all meta fields as hints so we can verify they arrived
    write!(
        f,
        r#"
fn configure(meta) {{
    #{{
        traps: 1,
        terrain_edits: 2,
        spawns: 1,
        hints: #{{
            w: meta.width,
            h: meta.height,
            e: meta.enemy_count,
            d: meta.difficulty
        }}
    }}
}}
"#
    )
    .unwrap();

    let meta = MapMeta {
        width: 42,
        height: 99,
        enemy_count: 7,
        difficulty: 4,
    };
    let (_, hints) =
        astraweave_author::run_author_script(script_path.to_str().unwrap(), &meta).unwrap();

    assert_eq!(hints["w"], 42);
    assert_eq!(hints["h"], 99);
    assert_eq!(hints["e"], 7);
    assert_eq!(hints["d"], 4);
}
