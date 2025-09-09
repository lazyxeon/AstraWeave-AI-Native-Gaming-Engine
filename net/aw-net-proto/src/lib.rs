use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROTOCOL_VERSION: u16 = 1;

/// For HMAC-like lightweight signing later, every room allocates a session key.
/// (You can upgrade to proper HMAC later; for now keep it minimal and portable.)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionKey(pub [u8; 32]);

impl SessionKey {
    pub fn random() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill(&mut bytes);
        SessionKey(bytes)
    }
}

/// Simple wire messages (focus on MVP end-to-end).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientToServer {
    Hello { protocol: u16 },
    /// Ask matchmaker for (or create) a room in a region.
    FindOrCreate {
        region: String,
        game_mode: String,
        party_size: u8,
    },
    /// Join a specific room if known (room_id from matchmaker).
    JoinRoom {
        room_id: String,
        display_name: String,
    },
    /// Per-frame input payload (prediction).
    InputFrame {
        seq: u32,
        tick_ms: u64,
        // e.g. movement vector, buttons; opaque to engine:
        input_blob: Vec<u8>,
        // lightweight signature: xor with session key for tamper-evident check (MVP)
        sig: [u8; 16],
    },
    /// Reliable pings for RTT estimate
    Ping { nano: u128 },
    /// Client acknowledges snapshot / reconciliation id
    Ack { last_snapshot_id: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerToClient {
    HelloAck { protocol: u16 },
    MatchResult { room_id: String, session_key_hint: [u8; 8] },
    JoinAccepted {
        room_id: String,
        player_id: String,
        session_key_hint: [u8; 8],
        tick_hz: u32,
    },
    /// Snapshot can contain either a full or delta state (opaque to engine)
    Snapshot {
        id: u32,
        server_tick: u64,
        base_id: Option<u32>,
        compressed: bool,
        payload: Vec<u8>, // engine-controlled data (bincode/postcard)
    },
    /// Correction vector for client reconciliation
    Reconcile {
        input_seq_ack: u32,
        corrected_state_hash: u64,
    },
    Pong { nano: u128 },
    /// Basic moderation / anti-cheat feedback
    RateLimited,
    ProtocolError { msg: String },
}

#[derive(Debug, Error)]
pub enum WireError {
    #[error("protocol mismatch (client={client}, server={server})")]
    ProtocolMismatch { client: u16, server: u16 },
    #[error("decode error: {0}")]
    Decode(String),
}

#[derive(Clone, Copy)]
pub enum Codec {
    /// Compact CBOR-like; great for small messages.
    PostcardLz4,
    /// Fallback / compatibility
    Bincode,
}

pub fn encode_msg(codec: Codec, msg: &impl Serialize) -> Vec<u8> {
    match codec {
        Codec::PostcardLz4 => {
            let raw = postcard::to_allocvec(msg).expect("serialize");
            lz4_flex::compress_prepend_size(&raw)
        }
        Codec::Bincode => bincode::serialize(msg).expect("serialize"),
    }
}

pub fn decode_msg<T: for<'de> Deserialize<'de>>(codec: Codec, bytes: &[u8]) -> Result<T, WireError> {
    match codec {
        Codec::PostcardLz4 => {
            let decompressed = lz4_flex::decompress_size_prepended(bytes)
                .map_err(|e| WireError::Decode(format!("lz4: {e}")))?;
            postcard::from_bytes(&decompressed).map_err(|e| WireError::Decode(format!("postcard: {e}")))
        }
        Codec::Bincode => bincode::deserialize(bytes).map_err(|e| WireError::Decode(format!("bincode: {e}"))),
    }
}

/// Minimal tamper-evident signature (MVP): xor 16 bytes of input hash with key hint
pub fn sign16(input: &[u8], session_key_hint: &[u8; 8]) -> [u8; 16] {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    let h = hasher.finish();
    let mut out = [0u8; 16];
    out[0..8].copy_from_slice(&h.to_le_bytes());
    out[8..16].copy_from_slice(&(!h).to_le_bytes());
    for i in 0..8 {
        out[i] ^= session_key_hint[i];
        out[8 + i] ^= session_key_hint[i];
    }
    out
}

/// Generate a short, URL-safe room id.
pub fn new_room_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}