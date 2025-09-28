use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

use astraweave_core::{
    validation::{validate_and_execute, ValidateCfg},
    IVec2, PlanIntent, Team, World,
};

#[derive(thiserror::Error, Debug)]
pub enum SdkError {
    #[error("schema mismatch: {0}")]
    Schema(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

pub trait GameAdapter {
    fn version(&self) -> Version;
    // future: hooks for feeding snapshots, receiving intents via IPC (gRPC/WebSocket)
}

// C ABI surface (MVP)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AWVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

#[no_mangle]
pub extern "C" fn aw_version() -> AWVersion {
    // Read from Cargo package version at compile time
    const V: &str = env!("CARGO_PKG_VERSION");
    // Simple parser for MAJOR.MINOR.PATCH
    let mut parts = V.split('.');
    let major = parts
        .next()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let minor = parts
        .next()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let patch = parts
        .next()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    AWVersion {
        major,
        minor,
        patch,
    }
}

#[no_mangle]
pub extern "C" fn aw_version_string(buf: *mut u8, len: usize) -> usize {
    let s = env!("CARGO_PKG_VERSION");
    let bytes = s.as_bytes();
    let n = bytes.len().min(len.saturating_sub(1));
    if buf.is_null() || len == 0 {
        return bytes.len() + 1;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n);
        *buf.add(n) = 0;
    }
    bytes.len() + 1 // required size including NUL
}

// Generic callback that receives a C string (NUL-terminated, UTF-8)
type CStringCallback = Option<unsafe extern "C" fn(msg: *const c_char)>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AWWorld(*mut AwWorldWrap);

struct AwWorldWrap {
    world: World,
    // Called synchronously at the end of aw_world_tick with the current snapshot JSON
    snapshot_cb: CStringCallback,
    // Called synchronously with a stable delta JSON relative to the last emission
    delta_cb: CStringCallback,
    // Previous entity states for delta computation
    prev: HashMap<u32, SimpleEntity>,
}

#[no_mangle]
pub extern "C" fn aw_world_create() -> AWWorld {
    let mut w = World::new();
    // seed a minimal scene: three teams
    let _p = w.spawn("P", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let _c = w.spawn("C", IVec2 { x: 3, y: 2 }, Team { id: 1 }, 80, 10);
    let _e = w.spawn("E", IVec2 { x: 7, y: 2 }, Team { id: 2 }, 60, 0);
    let boxed = Box::new(AwWorldWrap {
        world: w,
        snapshot_cb: None,
        delta_cb: None,
        prev: HashMap::new(),
    });
    AWWorld(Box::into_raw(boxed))
}

#[no_mangle]
pub extern "C" fn aw_world_destroy(_w: AWWorld) {
    if !_w.0.is_null() {
        unsafe {
            let _ = Box::from_raw(_w.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn aw_world_tick(_w: AWWorld, _dt: f32) {
    if _w.0.is_null() {
        return;
    }
    let wrap = unsafe { &mut *(_w.0) };
    wrap.world.tick(_dt);
    // If a snapshot callback is registered, emit the snapshot JSON
    if let Some(cb) = wrap.snapshot_cb {
        let snap = crate_snapshot(&wrap.world);
        if let Ok(s) = serde_json::to_string(&snap) {
            unsafe {
                use std::ffi::CString;
                if let Ok(cs) = CString::new(s) {
                    cb(cs.as_ptr());
                }
            }
        }
    }
    // If a delta callback is registered, compute and emit delta JSON relative to previous state
    if let Some(cb) = wrap.delta_cb {
        let cur = current_map(&wrap.world);
        let mut changed: Vec<SimpleEntity> = Vec::new();
        let mut removed: Vec<u32> = Vec::new();
        // Detect changes and additions
        for (id, ent) in cur.iter() {
            match wrap.prev.get(id) {
                Some(prev) if prev == ent => { /* unchanged */ }
                _ => changed.push(ent.clone()),
            }
        }
        // Detect removals
        for id in wrap.prev.keys() {
            if !cur.contains_key(id) {
                removed.push(*id);
            }
        }
        wrap.prev = cur; // update baseline
        let d = SimpleDelta {
            t: wrap.world.t,
            changed,
            removed,
        };
        if let Ok(s) = serde_json::to_string(&d) {
            unsafe {
                use std::ffi::CString;
                if let Ok(cs) = CString::new(s) {
                    cb(cs.as_ptr());
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn aw_world_set_snapshot_callback(_w: AWWorld, cb: CStringCallback) {
    if _w.0.is_null() {
        return;
    }
    let wrap = unsafe { &mut *(_w.0) };
    wrap.snapshot_cb = cb;
}

#[no_mangle]
pub extern "C" fn aw_world_set_delta_callback(_w: AWWorld, cb: CStringCallback) {
    if _w.0.is_null() {
        return;
    }
    let wrap = unsafe { &mut *(_w.0) };
    wrap.delta_cb = cb;
}

#[no_mangle]
pub extern "C" fn aw_world_snapshot_json(_w: AWWorld, buf: *mut u8, len: usize) -> usize {
    if _w.0.is_null() {
        return 0;
    }
    let wrap = unsafe { &mut *(_w.0) };
    let snap = crate_snapshot(&wrap.world);
    let s = serde_json::to_string(&snap).unwrap_or_else(|_| "{}".into());
    write_cstr(s.as_bytes(), buf, len)
}

#[derive(Serialize, Deserialize)]
struct SimpleSnapshot {
    t: f32,
    tick: u64,
    entities: Vec<SimpleEntity>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct SimpleEntity {
    id: u32,
    x: i32,
    y: i32,
    team: u8,
    hp: i32,
    ammo: i32,
}

fn crate_snapshot(w: &World) -> SimpleSnapshot {
    let entities = w
        .all_of_team(0)
        .into_iter()
        .chain(w.all_of_team(1))
        .chain(w.all_of_team(2))
        .filter_map(|id| {
            let pos = w.pos_of(id)?;
            let team = w.team(id)?.id;
            let hp = w.health(id)?.hp;
            let ammo = w.ammo(id)?.rounds;
            Some(SimpleEntity {
                id,
                x: pos.x,
                y: pos.y,
                team,
                hp,
                ammo,
            })
        })
        .collect();
    SimpleSnapshot {
        t: w.t,
        tick: 0,
        entities,
    }
}

fn current_map(w: &World) -> HashMap<u32, SimpleEntity> {
    let mut map = HashMap::new();
    for team in [0u8, 1u8, 2u8] {
        for id in w.all_of_team(team) {
            if let (Some(pos), Some(t), Some(h), Some(am)) =
                (w.pos_of(id), w.team(id), w.health(id), w.ammo(id))
            {
                map.insert(
                    id,
                    SimpleEntity {
                        id,
                        x: pos.x,
                        y: pos.y,
                        team: t.id,
                        hp: h.hp,
                        ammo: am.rounds,
                    },
                );
            }
        }
    }
    map
}

#[derive(Serialize)]
struct SimpleDelta {
    t: f32,
    changed: Vec<SimpleEntity>,
    removed: Vec<u32>,
}

// Error codes (exported for C consumers)
pub const AW_OK: i32 = 0;
pub const AW_ERR_NULL: i32 = -1;
pub const AW_ERR_PARAM: i32 = -2;
pub const AW_ERR_PARSE: i32 = -3;
pub const AW_ERR_EXEC: i32 = -4;

static LAST_ERROR: OnceLock<Mutex<String>> = OnceLock::new();

fn set_last_error(msg: &str) {
    let m = LAST_ERROR.get_or_init(|| Mutex::new(String::new()));
    if let Ok(mut s) = m.lock() {
        *s = msg.to_string();
    }
}

#[no_mangle]
pub extern "C" fn aw_last_error_string(buf: *mut u8, len: usize) -> usize {
    let s = LAST_ERROR
        .get()
        .and_then(|m| m.lock().ok().map(|s| s.clone()))
        .unwrap_or_default();
    write_cstr(s.as_bytes(), buf, len)
}

#[no_mangle]
pub extern "C" fn aw_world_submit_intent_json(
    _w: AWWorld,
    actor_id: u32,
    intent_json: *const c_char,
    cb: CStringCallback,
) -> i32 {
    if _w.0.is_null() {
        set_last_error("null world handle");
        return AW_ERR_NULL;
    }
    let wrap = unsafe { &mut *(_w.0) };
    let cstr = unsafe {
        if intent_json.is_null() {
            set_last_error("intent_json is null");
            return AW_ERR_PARAM;
        }
        CStr::from_ptr(intent_json).to_string_lossy().to_string()
    };
    let intent: Result<PlanIntent, _> = serde_json::from_str(&cstr);
    let intent = match intent {
        Ok(i) => i,
        Err(e) => {
            set_last_error(&format!("parse error: {}", e));
            return AW_ERR_PARSE;
        }
    };
    let mut log = |s: String| {
        if let Some(func) = cb {
            unsafe {
                use std::ffi::CString;
                if let Ok(cs) = CString::new(s) {
                    func(cs.as_ptr());
                }
            }
        }
    };
    let cfg = ValidateCfg {
        world_bounds: (0, 0, 19, 9),
    };
    match validate_and_execute(&mut wrap.world, actor_id, &intent, &cfg, &mut log) {
        Ok(_) => {
            set_last_error("");
            AW_OK
        }
        Err(e) => {
            set_last_error(&format!("exec error: {}", e));
            AW_ERR_EXEC
        }
    }
}

fn write_cstr(bytes: &[u8], buf: *mut u8, len: usize) -> usize {
    let n = bytes.len().min(len.saturating_sub(1));
    if buf.is_null() || len == 0 {
        return bytes.len() + 1;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n);
        *buf.add(n) = 0;
    }
    bytes.len() + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SNAP_COUNT: AtomicUsize = AtomicUsize::new(0);
    unsafe extern "C" fn snap_cb(_msg: *const c_char) {
        SNAP_COUNT.fetch_add(1, Ordering::SeqCst);
    }
    static DELTA_COUNT: AtomicUsize = AtomicUsize::new(0);
    unsafe extern "C" fn delta_cb(_msg: *const c_char) {
        DELTA_COUNT.fetch_add(1, Ordering::SeqCst);
    }
    #[test]
    fn c_abi_version() {
        let v = aw_version();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 4);
    }

    #[test]
    fn snapshot_callback_invoked_on_tick() {
        let w = aw_world_create();
        // register callback and tick once
        aw_world_set_snapshot_callback(w, Some(snap_cb));
        aw_world_tick(w, 0.016);
        assert_eq!(SNAP_COUNT.load(Ordering::SeqCst), 1);
        aw_world_destroy(w);
    }

    #[test]
    fn delta_callback_invoked_on_tick() {
        let w = aw_world_create();
        aw_world_set_delta_callback(w, Some(delta_cb));
        aw_world_tick(w, 0.016);
        assert_eq!(DELTA_COUNT.load(Ordering::SeqCst), 1);
        aw_world_destroy(w);
    }

    #[test]
    fn last_error_is_set_on_intent_parse_error() {
        let w = aw_world_create();
        let rc = unsafe {
            aw_world_submit_intent_json(
                w,
                1,
                std::ffi::CString::new("not json").unwrap().as_ptr(),
                None,
            )
        };
        assert_eq!(rc, AW_ERR_PARSE);
        let mut buf = [0u8; 128];
        let n = aw_last_error_string(buf.as_mut_ptr(), buf.len());
        assert!(n > 1);
        let s = std::ffi::CStr::from_bytes_until_nul(&buf)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(s.contains("parse error"));
        aw_world_destroy(w);
    }
}
