# Networking Envelopes (MVP)

## Message Types
- Input: buttons/axes per client
- Intent: AI `PlanIntent`
- Snapshot: world state (delta-compressed)
- ReplayFrame: deterministic verification frame

## Snapshot Delta
- Serialize component tables with change masks
- Interest filtering by region and proximity

## Transport
- WebSockets default; QUIC/UDP optional for low-latency
- Server authoritative; client prediction for local movement (post-MVP)
