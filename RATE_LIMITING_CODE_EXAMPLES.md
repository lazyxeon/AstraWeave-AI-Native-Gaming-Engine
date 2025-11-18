# Rate Limiting Implementation Examples
## Ready-to-Use Code for `aw-net-server`

---

## Quick Reference: Current Implementation Issues

**CRITICAL BUG** in `net/aw-net-server/src/main.rs` (lines 638, 754):

```rust
// BROKEN - adds tokens on EVERY message, not based on time!
p.tokens += 8.0; // refill
if p.tokens > 60.0 {
    p.tokens = 60.0;
}
p.tokens -= 1.0;
```

**Problem**: This allows 8x the intended rate. An attacker sending 480 msg/sec would always pass.

---

## Solution 1: Using `governor` Crate (RECOMMENDED)

### Step 1: Add Dependency

Add to `net/aw-net-server/Cargo.toml`:

```toml
[dependencies]
governor = "0.10"
nonzero_ext = "0.3"  # Helper for NonZeroU32
```

### Step 2: Update Player Struct

Replace lines 26-36 in `main.rs`:

```rust
use governor::{Quota, RateLimiter};
use nonzero_ext::nonzero;

#[derive(Clone)]
struct Player {
    id: PlayerId,
    display: String,
    last_input_seq: u32,
    last_seen: Instant,
    
    // Proper rate limiting using governor
    input_limiter: RateLimiter<
        governor::state::direct::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
    >,
}

impl Player {
    fn new(id: String, display: String) -> Self {
        // 60 messages per second, allow burst of 60
        let quota = Quota::per_second(nonzero!(60u32))
            .allow_burst(nonzero!(60u32));
        
        Self {
            id,
            display,
            last_input_seq: 0,
            last_seen: Instant::now(),
            input_limiter: RateLimiter::direct(quota),
        }
    }
}
```

### Step 3: Update Message Handling (Lines 632-662)

Replace the broken rate limiting code:

```rust
async fn handle_in_session_message(
    app: &AppState,
    rid: &str,
    pid: &str,
    ws: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    msg: ClientToServer,
) -> Result<()> {
    match msg {
        ClientToServer::InputFrame {
            seq,
            input_blob,
            sig,
            ..
        } => {
            let mut kick = false;
            {
                let mut rooms = app.rooms.lock();
                if let Some(room) = rooms.get_mut(rid) {
                    if let Some(p) = room.players.get_mut(pid) {
                        // Proper rate limiting with governor
                        if p.input_limiter.check().is_err() {
                            warn!("Rate limit exceeded for pid={pid}");
                            kick = true;
                        } else {
                            p.last_input_seq = seq;
                            p.last_seen = tokio::time::Instant::now();
                            
                            // Tamper check
                            let mut hint = [0u8; 8];
                            hint.copy_from_slice(&room.session_key.0[0..8]);
                            if sig != aw_net_proto::sign16(&input_blob, &hint) {
                                warn!("tamper-evident signature mismatch for pid={pid}");
                            }
                        }
                    }
                }
            }
            if kick {
                send(app, ws, &ServerToClient::RateLimited).await?;
            }
        }
        ClientToServer::Ping { nano } => {
            send(app, ws, &ServerToClient::Pong { nano }).await?;
        }
        ClientToServer::Ack { .. } => {}
        _ => {}
    }
    Ok(())
}
```

### Step 4: Update Duplicate Code (Lines 748-786)

Apply the same fix to the second occurrence of message handling:

```rust
ClientToServer::InputFrame {
    seq,
    input_blob,
    sig,
    ..
} => {
    let mut kick = false;
    {
        let mut rooms = app.rooms.lock();
        if let Some(room) = rooms.get_mut(rid) {
            if let Some(p) = room.players.get_mut(pid) {
                // Proper rate limiting with governor
                if p.input_limiter.check().is_err() {
                    warn!("Rate limit exceeded for pid={pid}");
                    kick = true;
                } else {
                    p.last_input_seq = seq;
                    p.last_seen = tokio::time::Instant::now();
                    
                    // Tamper check
                    let mut hint = [0u8; 8];
                    hint.copy_from_slice(&room.session_key.0[0..8]);
                    if sig != aw_net_proto::sign16(&input_blob, &hint) {
                        warn!("tamper-evident signature mismatch for pid={pid}");
                    }
                }
            }
        }
    }
    if kick {
        send(app, ws, &ServerToClient::RateLimited).await?;
    }
}
```

---

## Solution 2: Custom Time-Based Token Bucket (No Dependencies)

### Step 1: Update Player Struct

Replace lines 26-36 in `main.rs`:

```rust
#[derive(Clone)]
struct Player {
    id: PlayerId,
    display: String,
    last_input_seq: u32,
    last_seen: Instant,
    
    // Token bucket state
    tokens: f32,
    token_capacity: f32,
    refill_rate: f32, // tokens per second
    last_refill: Instant,
}

impl Player {
    fn new(id: String, display: String) -> Self {
        Self {
            id,
            display,
            last_input_seq: 0,
            last_seen: Instant::now(),
            tokens: 60.0,           // Start full (allows immediate burst)
            token_capacity: 60.0,    // Max 60 tokens
            refill_rate: 60.0,       // Refill 60 tokens per second
            last_refill: Instant::now(),
        }
    }
    
    /// Check rate limit and consume 1 token if available
    /// Returns true if allowed, false if rate limited
    fn check_rate_limit(&mut self) -> bool {
        // Calculate elapsed time and refill tokens
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = elapsed.as_secs_f32() * self.refill_rate;
        
        // Update tokens (capped at capacity)
        self.tokens = (self.tokens + tokens_to_add).min(self.token_capacity);
        self.last_refill = now;
        
        // Try to consume 1 token
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
```

### Step 2: Update Message Handling (Lines 632-662)

```rust
async fn handle_in_session_message(
    app: &AppState,
    rid: &str,
    pid: &str,
    ws: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    msg: ClientToServer,
) -> Result<()> {
    match msg {
        ClientToServer::InputFrame {
            seq,
            input_blob,
            sig,
            ..
        } => {
            let mut kick = false;
            {
                let mut rooms = app.rooms.lock();
                if let Some(room) = rooms.get_mut(rid) {
                    if let Some(p) = room.players.get_mut(pid) {
                        // Time-based rate limiting
                        if !p.check_rate_limit() {
                            warn!("Rate limit exceeded for pid={pid}");
                            kick = true;
                        } else {
                            p.last_input_seq = seq;
                            p.last_seen = tokio::time::Instant::now();
                            
                            // Tamper check
                            let mut hint = [0u8; 8];
                            hint.copy_from_slice(&room.session_key.0[0..8]);
                            if sig != aw_net_proto::sign16(&input_blob, &hint) {
                                warn!("tamper-evident signature mismatch for pid={pid}");
                            }
                        }
                    }
                }
            }
            if kick {
                send(app, ws, &ServerToClient::RateLimited).await?;
            }
        }
        ClientToServer::Ping { nano } => {
            send(app, ws, &ServerToClient::Pong { nano }).await?;
        }
        ClientToServer::Ack { .. } => {}
        _ => {}
    }
    Ok(())
}
```

### Step 3: Update Duplicate Code (Lines 748-786)

Apply the same change:

```rust
ClientToServer::InputFrame {
    seq,
    input_blob,
    sig,
    ..
} => {
    let mut kick = false;
    {
        let mut rooms = app.rooms.lock();
        if let Some(room) = rooms.get_mut(rid) {
            if let Some(p) = room.players.get_mut(pid) {
                // Time-based rate limiting
                if !p.check_rate_limit() {
                    warn!("Rate limit exceeded for pid={pid}");
                    kick = true;
                } else {
                    p.last_input_seq = seq;
                    p.last_seen = tokio::time::Instant::now();
                    
                    // Tamper check
                    let mut hint = [0u8; 8];
                    hint.copy_from_slice(&room.session_key.0[0..8]);
                    if sig != aw_net_proto::sign16(&input_blob, &hint) {
                        warn!("tamper-evident signature mismatch for pid={pid}");
                    }
                }
            }
        }
    }
    if kick {
        send(app, ws, &ServerToClient::RateLimited).await?;
    }
}
```

---

## Bonus: Connection-Level Rate Limiting

Add connection-level DDoS protection:

### Step 1: Add Connection Limiter to AppState

```rust
use std::net::IpAddr;

#[derive(Clone)]
struct AppState {
    rooms: Arc<Mutex<HashMap<RoomId, Room>>>,
    db: sled::Db,
    codec: Codec,
    
    // Connection rate limiting per IP
    connection_limiters: Arc<Mutex<HashMap<IpAddr, RateLimiter<...>>>>,
}
```

### Step 2: Check on Connection

```rust
async fn handle_connection(
    app: AppState,
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
) -> Result<()> {
    let ip = addr.ip();
    
    // Check connection rate limit
    {
        let mut limiters = app.connection_limiters.lock();
        let limiter = limiters.entry(ip).or_insert_with(|| {
            // 20 connections per minute per IP
            let quota = Quota::per_minute(nonzero!(20u32));
            RateLimiter::direct(quota)
        });
        
        if limiter.check().is_err() {
            warn!("Connection rate limit exceeded for IP: {}", ip);
            return Ok(()); // Drop connection
        }
    }
    
    // Continue with WebSocket upgrade...
}
```

---

## Testing Your Implementation

### Unit Test for Rate Limiting

Add to `net/aw-net-server/src/main.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_rate_limiting_allows_burst() {
        let mut player = Player::new("test".to_string(), "Test".to_string());
        
        // Should allow first 60 messages (burst)
        for i in 0..60 {
            assert!(player.check_rate_limit(), "Message {} should be allowed", i);
        }
        
        // 61st message should be rate limited
        assert!(!player.check_rate_limit(), "61st message should be blocked");
    }
    
    #[tokio::test]
    async fn test_rate_limiting_refills_over_time() {
        let mut player = Player::new("test".to_string(), "Test".to_string());
        
        // Exhaust bucket
        for _ in 0..60 {
            player.check_rate_limit();
        }
        assert!(!player.check_rate_limit(), "Bucket should be empty");
        
        // Wait 1 second - should refill 60 tokens
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Should allow another 60 messages
        for i in 0..60 {
            assert!(player.check_rate_limit(), "Message {} after refill should be allowed", i);
        }
    }
    
    #[tokio::test]
    async fn test_rate_limiting_gradual_refill() {
        let mut player = Player::new("test".to_string(), "Test".to_string());
        
        // Exhaust bucket
        for _ in 0..60 {
            player.check_rate_limit();
        }
        
        // Wait 100ms - should refill ~6 tokens (60/sec * 0.1sec)
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let mut allowed = 0;
        for _ in 0..10 {
            if player.check_rate_limit() {
                allowed += 1;
            }
        }
        
        // Should allow approximately 6 messages (with timing variance)
        assert!(allowed >= 5 && allowed <= 7, "Expected ~6 allowed, got {}", allowed);
    }
    
    #[tokio::test]
    async fn test_rate_limiting_sustained_rate() {
        let mut player = Player::new("test".to_string(), "Test".to_string());
        
        // Test sustained rate: 60 msg/sec for 3 seconds
        let start = Instant::now();
        let mut total_allowed = 0;
        
        while start.elapsed() < Duration::from_secs(3) {
            if player.check_rate_limit() {
                total_allowed += 1;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        // Should allow ~180 messages (60/sec * 3sec) + initial 60 burst
        // Allow 10% variance for timing
        assert!(
            total_allowed >= 216 && total_allowed <= 264,
            "Expected ~240 messages, got {}",
            total_allowed
        );
    }
}
```

### Run Tests

```bash
cd net/aw-net-server
cargo test --lib rate_limiting
```

---

## Quick Comparison

| Aspect | Governor | Custom Token Bucket |
|--------|----------|---------------------|
| **Lines of Code** | ~10 | ~30 |
| **Dependencies** | +1 crate | None |
| **Performance** | Lock-free (fastest) | Mutex required |
| **Maintenance** | Well-tested library | Must maintain yourself |
| **Memory per Player** | 64 bits | 160 bits |
| **Recommendation** | **Production use** | Learning/no-deps constraint |

---

## Recommended Parameters

```rust
// For 60 Hz game
const INPUT_RATE: u32 = 60;      // 60 messages per second
const INPUT_BURST: u32 = 60;     // Allow 1 second burst

// For control messages (chat, etc.)
const CONTROL_RATE: u32 = 10;    // 10 messages per second
const CONTROL_BURST: u32 = 5;    // Allow 0.5 second burst

// For connections (DDoS protection)
const CONN_RATE: u32 = 20;       // 20 connections per minute per IP
```

---

## Summary

1. **Fix immediately**: Current code is broken and ineffective
2. **Use Solution 1 (governor)**: Best for production - lock-free, well-tested
3. **Use Solution 2 (custom)**: If you can't add dependencies
4. **Add tests**: Verify rate limiting works correctly
5. **Monitor**: Log rate limit events to tune parameters

Both solutions fix the critical security vulnerability in the current implementation.
