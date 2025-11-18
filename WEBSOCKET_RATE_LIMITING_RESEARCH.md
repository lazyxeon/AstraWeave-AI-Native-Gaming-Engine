# WebSocket Rate Limiting Research Report
## Industry Best Practices for Real-Time Game Servers

---

## Executive Summary

Current implementation in `aw-net-server/src/main.rs` (lines 633-646, 751-761) has a **critical flaw**: tokens are added on every message regardless of elapsed time, making the rate limiter ineffective.

**Key Finding**: Proper token bucket implementation requires **time-based refill** using elapsed duration, not per-message increments.

---

## 1. Token Bucket Algorithm - Proper Implementation

### Core Principle

The token bucket algorithm controls request rate by:
1. Maintaining a bucket with a maximum capacity of tokens
2. **Refilling tokens at a fixed rate based on elapsed time**
3. Consuming tokens for each request
4. Rejecting requests when insufficient tokens exist

### Critical Implementation Detail: Time-Based Refill

**WRONG (Current Implementation)**:
```rust
// Lines 638, 754 in current code
p.tokens += 8.0; // refill on EVERY message - INCORRECT!
if p.tokens > 60.0 {
    p.tokens = 60.0;
}
p.tokens -= 1.0;
```

**Problem**: This adds 8 tokens per message, allowing 8x the intended rate. A malicious client sending messages rapidly would always have tokens.

**CORRECT Implementation**:
```rust
// Calculate elapsed time since last refill
let now = Instant::now();
let elapsed = now.duration_since(p.last_refill);

// Refill based on elapsed time
let tokens_to_add = elapsed.as_secs_f32() * REFILL_RATE_PER_SECOND;
p.tokens = (p.tokens + tokens_to_add).min(BUCKET_CAPACITY);
p.last_refill = now;

// Consume token for this request
if p.tokens >= 1.0 {
    p.tokens -= 1.0;
    // Allow request
} else {
    // Rate limit exceeded
}
```

### Mathematical Foundation

```
tokens_available = min(
    current_tokens + (elapsed_time * refill_rate),
    max_capacity
)

where:
- elapsed_time = now - last_refill_timestamp
- refill_rate = tokens per second (e.g., 60 for 60 msg/sec)
- max_capacity = burst size (e.g., 60 tokens)
```

---

## 2. GCRA (Generic Cell Rate Algorithm)

### Why GCRA is Superior

The GCRA is functionally equivalent to a token bucket but offers significant advantages:

1. **No background process**: Updates state on-demand when requests arrive
2. **Nanosecond precision**: Continuously updates on nanosecond scale
3. **Minimal memory**: State stored in single `AtomicU64` (64 bits)
4. **Lock-free**: Uses compare-and-swap atomic operations

### GCRA Virtual Scheduling Algorithm

Instead of tracking tokens, GCRA tracks the "Theoretical Arrival Time" (TAT):

```
emission_interval = 1 / rate_limit (e.g., 1/60 = 16.67ms for 60 req/sec)
burst_capacity = max_burst * emission_interval

# On each request:
if now >= TAT - burst_capacity:
    # Allow request
    TAT = max(TAT, now) + emission_interval
else:
    # Reject request
```

**Advantages**:
- Single timestamp instead of token count + last_refill
- Mathematically equivalent to leaky bucket
- Used in ATM networks, proven at scale

---

## 3. Rust Rate Limiting Libraries

### A. `governor` (Recommended)

**Crate**: https://crates.io/crates/governor

**Pros**:
- Implements GCRA algorithm
- Lock-free using `AtomicU64`
- 10x faster than mutex-based solutions
- No background threads needed
- Works with `no_std`
- Well-maintained (819 GitHub stars)

**Cons**:
- Only valid for 584 years after creation (acceptable for game servers)

**Basic Usage**:
```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

// Create quota: 60 requests per second with burst of 60
let quota = Quota::per_second(NonZeroU32::new(60).unwrap())
    .allow_burst(NonZeroU32::new(60).unwrap());

// Create rate limiter
let limiter = RateLimiter::direct(quota);

// Check request
if limiter.check().is_ok() {
    // Allow request
} else {
    // Rate limited
}
```

**Per-Connection Usage**:
```rust
use governor::{Quota, RateLimiter};
use std::collections::HashMap;
use std::num::NonZeroU32;

struct Player {
    id: String,
    rate_limiter: RateLimiter<
        governor::state::direct::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
    >,
    // ... other fields
}

impl Player {
    fn new(id: String) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(60).unwrap())
            .allow_burst(NonZeroU32::new(60).unwrap());
        
        Self {
            id,
            rate_limiter: RateLimiter::direct(quota),
        }
    }
    
    fn check_rate_limit(&self) -> bool {
        self.rate_limiter.check().is_ok()
    }
}
```

### B. `leaky-bucket`

**Crate**: https://crates.io/crates/leaky-bucket

**Pros**:
- Simple API
- Async-first design

**Cons**:
- Requires background refill task
- More overhead than GCRA
- Less efficient for high-throughput

### C. Standard Library Implementation

For maximum control, implement time-based token bucket:

```rust
use std::time::{Duration, Instant};

#[derive(Clone)]
struct TokenBucket {
    tokens: f32,
    capacity: f32,
    refill_rate: f32, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f32, refill_rate: f32) -> Self {
        Self {
            tokens: capacity, // Start full for burst allowance
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }
    
    fn try_consume(&mut self, cost: f32) -> bool {
        self.refill();
        
        if self.tokens >= cost {
            self.tokens -= cost;
            true
        } else {
            false
        }
    }
    
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = elapsed.as_secs_f32() * self.refill_rate;
        
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }
}
```

---

## 4. Game Server Rate Limiting Best Practices

### A. Appropriate Rate Limits for Game Input

| Game Type | Tick Rate | Input Rate | Burst Allowance |
|-----------|-----------|------------|-----------------|
| Turn-based | 10 Hz | 10 msg/sec | 5 messages |
| MOBA/RTS | 30 Hz | 30-60 msg/sec | 15 messages |
| Fast FPS | 60-128 Hz | 60-120 msg/sec | 30 messages |
| Fighting game | 60 Hz | 60 msg/sec | 20 messages |

**Recommended for AstraWeave** (assuming 60 Hz simulation):
- **Steady rate**: 60 messages/second (one per tick)
- **Burst capacity**: 60 messages (1 second worth)
- **Cost per message**: 1 token

This allows legitimate high-frequency input while preventing abuse.

### B. Handling Burst Traffic

**Burst Scenarios**:
1. **Reconnection**: Player reconnects and sends cached inputs
2. **Lag compensation**: Buffered inputs released after network recovery
3. **Legitimate rapid input**: Fast-paced combat, rapid clicking

**Solution**: Set burst capacity to cover reasonable bursts:
```rust
// Allow 1 second worth of burst
let burst_capacity = tick_rate; // e.g., 60 for 60 Hz

// Or allow 0.5 second burst for stricter control
let burst_capacity = tick_rate / 2; // 30 for 60 Hz
```

### C. Per-Connection vs Per-IP Rate Limiting

**Per-Connection (Recommended for Game Servers)**:
- **Use case**: Limit each player's input rate individually
- **Advantage**: Fair per-player limits, no interference between players
- **Implementation**: Store rate limiter in `Player` struct
- **Attack vector**: Cannot prevent connection flooding

**Per-IP Rate Limiting**:
- **Use case**: Prevent connection flooding, DDoS mitigation
- **Advantage**: Limits malicious actor's total connections
- **Implementation**: Global HashMap `<IpAddr, RateLimiter>`
- **Disadvantage**: Punishes NAT/shared networks

**Hybrid Approach (Best Practice)**:
```rust
// Connection-level: Limit connection attempts per IP
let connection_limiter: HashMap<IpAddr, RateLimiter> = ...;
// Rate: 10 connections per minute

// Player-level: Limit message rate per connection
struct Player {
    message_limiter: RateLimiter, // 60 msg/sec
}
```

---

## 5. DDoS Protection Techniques

### Layer 1: Connection-Level Protection

```rust
use std::net::IpAddr;
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

// Global connection rate limiter
struct ConnectionLimiter {
    limiters: Arc<Mutex<HashMap<IpAddr, RateLimiter<...>>>>,
}

impl ConnectionLimiter {
    fn check_connection(&self, ip: IpAddr) -> bool {
        let mut limiters = self.limiters.lock();
        let limiter = limiters.entry(ip).or_insert_with(|| {
            // 10 connections per minute per IP
            let quota = Quota::per_minute(NonZeroU32::new(10).unwrap());
            RateLimiter::direct(quota)
        });
        
        limiter.check().is_ok()
    }
}
```

### Layer 2: Message-Level Protection

```rust
// Per-player message rate limiting
struct Player {
    // Fast input: 60 msg/sec
    input_limiter: RateLimiter<...>,
    
    // Slow control: 5 msg/sec for room/matchmaking requests
    control_limiter: RateLimiter<...>,
}
```

### Layer 3: Bandwidth Protection

```rust
struct Player {
    bytes_sent: usize,
    bytes_received: usize,
    last_bandwidth_check: Instant,
}

impl Player {
    fn check_bandwidth(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_bandwidth_check) >= Duration::from_secs(1) {
            // Reset counters every second
            self.bytes_sent = 0;
            self.bytes_received = 0;
            self.last_bandwidth_check = now;
        }
        
        // Limit: 100 KB/sec incoming, 500 KB/sec outgoing
        self.bytes_received < 100_000 && self.bytes_sent < 500_000
    }
}
```

### Layer 4: Application-Level Detection

```rust
// Track abnormal patterns
struct PlayerMetrics {
    message_count: usize,
    invalid_message_count: usize,
    last_reset: Instant,
}

impl PlayerMetrics {
    fn update_invalid_rate(&mut self) {
        self.invalid_message_count += 1;
        self.message_count += 1;
        
        // If >50% messages are invalid, likely attack
        if self.message_count > 100 {
            let invalid_rate = self.invalid_message_count as f32 / self.message_count as f32;
            if invalid_rate > 0.5 {
                // Kick player
            }
        }
    }
}
```

---

## 6. Recommended Implementation for `aw-net-server`

### Option A: Using `governor` (Recommended)

**Add to `Cargo.toml`**:
```toml
[dependencies]
governor = "0.10"
```

**Update `Player` struct**:
```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

#[derive(Clone)]
struct Player {
    id: PlayerId,
    display: String,
    last_input_seq: u32,
    last_seen: Instant,
    
    // Replace simple token counter with proper rate limiter
    input_limiter: RateLimiter<
        governor::state::direct::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
    >,
}

impl Player {
    fn new(id: String, display: String) -> Self {
        // 60 messages per second with burst of 60
        let quota = Quota::per_second(NonZeroU32::new(60).unwrap())
            .allow_burst(NonZeroU32::new(60).unwrap());
        
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

**Update message handling (lines 633-646, 751-761)**:
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
                        // Check rate limit using governor
                        if p.input_limiter.check().is_err() {
                            kick = true;
                        } else {
                            p.last_input_seq = seq;
                            p.last_seen = tokio::time::Instant::now();
                        }
                        
                        // Tamper check
                        let mut hint = [0u8; 8];
                        hint.copy_from_slice(&room.session_key.0[0..8]);
                        if sig != aw_net_proto::sign16(&input_blob, &hint) {
                            warn!("tamper-evident signature mismatch for pid={pid}");
                        }
                    }
                }
            }
            if kick {
                send(app, ws, &ServerToClient::RateLimited).await?;
            }
        }
        // ... other message types
    }
    Ok(())
}
```

### Option B: Standard Library Implementation (No Dependencies)

**Update `Player` struct**:
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
            tokens: 60.0,           // Start with full bucket
            token_capacity: 60.0,    // Max 60 tokens
            refill_rate: 60.0,       // 60 tokens/second
            last_refill: Instant::now(),
        }
    }
    
    fn check_rate_limit(&mut self) -> bool {
        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = elapsed.as_secs_f32() * self.refill_rate;
        
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

**Update message handling**:
```rust
ClientToServer::InputFrame { seq, input_blob, sig, .. } => {
    let mut kick = false;
    {
        let mut rooms = app.rooms.lock();
        if let Some(room) = rooms.get_mut(rid) {
            if let Some(p) = room.players.get_mut(pid) {
                // Check rate limit with time-based refill
                if !p.check_rate_limit() {
                    kick = true;
                } else {
                    p.last_input_seq = seq;
                    p.last_seen = tokio::time::Instant::now();
                }
                
                // Tamper check
                let mut hint = [0u8; 8];
                hint.copy_from_slice(&room.session_key.0[0..8]);
                if sig != aw_net_proto::sign16(&input_blob, &hint) {
                    warn!("tamper-evident signature mismatch for pid={pid}");
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

## 7. Recommended Rate Parameters

### For 60 Hz Game Simulation

```rust
const INPUT_RATE_LIMIT: u32 = 60;      // messages per second
const INPUT_BURST_SIZE: u32 = 60;      // allow 1 second burst
const MESSAGE_COST: f32 = 1.0;         // cost per input message

// For control messages (join, leave, chat)
const CONTROL_RATE_LIMIT: u32 = 10;    // messages per second
const CONTROL_BURST_SIZE: u32 = 5;     // allow 0.5 second burst

// For connection attempts (DDoS protection)
const CONNECTION_RATE_LIMIT: u32 = 10; // connections per minute per IP
```

### Tuning Guidelines

1. **Match tick rate**: Input rate should match or slightly exceed server tick rate
2. **Burst for latency**: Allow burst = rate × max_expected_latency_spike (in seconds)
3. **Cost per message**: Generally 1.0, but can be higher for expensive operations
4. **Monitor and adjust**: Log rate limit events, adjust based on false positives

---

## 8. Testing Rate Limiting

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let mut player = Player::new("test".to_string(), "Test".to_string());
        
        // Should allow first 60 messages (burst)
        for _ in 0..60 {
            assert!(player.check_rate_limit());
        }
        
        // 61st message should be rate limited
        assert!(!player.check_rate_limit());
        
        // Wait 1 second, should refill 60 tokens
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Should allow another 60 messages
        for _ in 0..60 {
            assert!(player.check_rate_limit());
        }
    }
    
    #[tokio::test]
    async fn test_gradual_refill() {
        let mut player = Player::new("test".to_string(), "Test".to_string());
        
        // Exhaust bucket
        for _ in 0..60 {
            player.check_rate_limit();
        }
        
        // Wait 100ms (should refill 6 tokens at 60/sec)
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Should allow ~6 messages
        let mut allowed = 0;
        for _ in 0..10 {
            if player.check_rate_limit() {
                allowed += 1;
            }
        }
        
        assert!(allowed >= 5 && allowed <= 7); // Allow some timing variance
    }
}
```

---

## 9. Comparison of Approaches

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **Governor (GCRA)** | Lock-free, fastest, well-tested | External dependency | **Best for production** |
| **Custom Token Bucket** | No dependencies, full control | Must implement correctly | Good for learning |
| **Leaky Bucket** | Simple concept | Requires background task | Avoid |
| **Current Implementation** | None | **Broken - ineffective** | **Must fix** |

---

## 10. Key Takeaways

1. **Critical Fix Required**: Current implementation is broken - tokens refill per message, not per time
2. **Use `governor`**: Most efficient, well-tested, used in production systems
3. **If custom implementation**: Must use time-based refill with `Instant::now()`
4. **Rate parameters for 60 Hz**: 60 msg/sec with burst of 60
5. **Per-connection limiting**: Fairest for game servers, prevents player interference
6. **Add per-IP limiting**: Layer defense against connection flooding
7. **Monitor metrics**: Track rate limit events to tune parameters
8. **Test thoroughly**: Unit test refill logic, integration test under load

---

## References

1. Generic Cell Rate Algorithm (GCRA): https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm
2. Governor crate: https://crates.io/crates/governor
3. Token Bucket Algorithm: https://en.wikipedia.org/wiki/Token_bucket
4. Designing Fair Token Bucket Policies: https://levelup.gitconnected.com/designing-fair-token-bucket-policies-for-real-time-apps-289b00eb4435
5. Rate Limiting in Real-Time Systems: https://brandur.org/rate-limiting
6. WebSocket DDoS Protection: https://www.localcan.com/blog/rate-limit-and-mitigate-websockets-ddos-attacks-with-cloudflare-api

---

## Next Steps

1. **Immediate**: Fix broken rate limiting using Option A or B above
2. **Testing**: Add unit tests for rate limiting logic
3. **Monitoring**: Add metrics for rate limit events
4. **Documentation**: Document rate limit parameters in config
5. **Load testing**: Verify performance under attack scenarios
