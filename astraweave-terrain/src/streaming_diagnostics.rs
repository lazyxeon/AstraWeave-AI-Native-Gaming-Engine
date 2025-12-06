//! Diagnostics overlay for terrain streaming
//!
//! Provides visualization and telemetry for chunk streaming performance:
//! - Chunk load states (loaded, loading, pending, unloaded)
//! - Memory usage tracking
//! - Queue depth monitoring
//! - Frame hitch detection
//!
//! For use in debug builds and performance profiling.

use crate::{ChunkId, LodStats, StreamingStats};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Chunk load state for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkLoadState {
    /// Chunk is fully loaded and rendered
    Loaded,

    /// Chunk is being generated in background
    Loading,

    /// Chunk is in load queue (not started)
    Pending,

    /// Chunk is unloaded (too far from camera)
    Unloaded,
}

/// Frame hitch detection
#[derive(Debug, Clone)]
pub struct HitchDetector {
    /// Recent frame times (milliseconds)
    frame_times: VecDeque<f32>,

    /// Maximum history size
    max_history: usize,

    /// Hitch threshold (ms)
    hitch_threshold: f32,

    /// Hitch count in window
    hitch_count: usize,
}

impl HitchDetector {
    /// Create a new hitch detector
    pub fn new(max_history: usize, hitch_threshold: f32) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(max_history),
            max_history,
            hitch_threshold,
            hitch_count: 0,
        }
    }

    /// Record a frame time and check for hitch
    pub fn record_frame(&mut self, frame_time_ms: f32) -> bool {
        let is_hitch = frame_time_ms > self.hitch_threshold;

        if is_hitch {
            self.hitch_count += 1;
        }

        self.frame_times.push_back(frame_time_ms);

        // Remove oldest frame if over limit
        if self.frame_times.len() > self.max_history {
            let oldest = self.frame_times.pop_front().unwrap_or(0.0);
            if oldest > self.hitch_threshold {
                self.hitch_count = self.hitch_count.saturating_sub(1);
            }
        }

        is_hitch
    }

    /// Get average frame time in window
    pub fn average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.frame_times.iter().sum();
        sum / self.frame_times.len() as f32
    }

    /// Get p99 frame time (99th percentile)
    pub fn p99_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let mut sorted: Vec<f32> = self.frame_times.iter().copied().collect();
        // Use unwrap_or for partial_cmp to handle potential NaN values gracefully
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = ((sorted.len() as f32 * 0.99).ceil() as usize).min(sorted.len() - 1);
        sorted[index]
    }

    /// Get hitch count in window
    pub fn hitch_count(&self) -> usize {
        self.hitch_count
    }

    /// Get hitch rate (percent of frames)
    pub fn hitch_rate(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        (self.hitch_count as f32 / self.frame_times.len() as f32) * 100.0
    }
}

/// Memory usage tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total bytes allocated for chunks
    pub total_bytes: usize,

    /// Bytes per chunk (average)
    pub bytes_per_chunk: usize,

    /// Number of chunks in memory
    pub chunk_count: usize,

    /// Peak memory usage
    pub peak_bytes: usize,
}

impl MemoryStats {
    /// Update memory stats
    pub fn update(&mut self, chunk_count: usize, bytes_per_chunk: usize) {
        self.chunk_count = chunk_count;
        self.bytes_per_chunk = bytes_per_chunk;
        self.total_bytes = chunk_count * bytes_per_chunk;

        if self.total_bytes > self.peak_bytes {
            self.peak_bytes = self.total_bytes;
        }
    }

    /// Get memory delta from peak (percent)
    pub fn delta_from_peak_percent(&self) -> f32 {
        if self.peak_bytes == 0 {
            return 0.0;
        }

        ((self.total_bytes as f32 / self.peak_bytes as f32) - 1.0) * 100.0
    }

    /// Get memory in MB
    pub fn total_mb(&self) -> f32 {
        self.total_bytes as f32 / (1024.0 * 1024.0)
    }
}

/// Streaming diagnostics
pub struct StreamingDiagnostics {
    /// Chunk load states
    chunk_states: HashMap<ChunkId, ChunkLoadState>,

    /// Hitch detector
    hitch_detector: HitchDetector,

    /// Memory stats
    memory_stats: MemoryStats,

    /// Streaming stats snapshot
    streaming_stats: StreamingStats,

    /// LOD stats snapshot
    lod_stats: LodStats,

    /// Camera position
    camera_pos: Vec3,
}

impl StreamingDiagnostics {
    /// Create a new diagnostics overlay
    pub fn new(hitch_threshold_ms: f32, history_frames: usize) -> Self {
        Self {
            chunk_states: HashMap::new(),
            hitch_detector: HitchDetector::new(history_frames, hitch_threshold_ms),
            memory_stats: MemoryStats::default(),
            streaming_stats: StreamingStats::default(),
            lod_stats: LodStats::default(),
            camera_pos: Vec3::ZERO,
        }
    }

    /// Update chunk states
    pub fn update_chunk_states(
        &mut self,
        loaded: &[ChunkId],
        loading: &[ChunkId],
        pending: &[ChunkId],
    ) {
        // Clear old states
        self.chunk_states.clear();

        // Mark loaded
        for &chunk_id in loaded {
            self.chunk_states.insert(chunk_id, ChunkLoadState::Loaded);
        }

        // Mark loading
        for &chunk_id in loading {
            self.chunk_states.insert(chunk_id, ChunkLoadState::Loading);
        }

        // Mark pending
        for &chunk_id in pending {
            self.chunk_states.insert(chunk_id, ChunkLoadState::Pending);
        }
    }

    /// Record frame time
    pub fn record_frame(&mut self, frame_time_ms: f32) -> bool {
        self.hitch_detector.record_frame(frame_time_ms)
    }

    /// Update memory stats
    pub fn update_memory(&mut self, chunk_count: usize, bytes_per_chunk: usize) {
        self.memory_stats.update(chunk_count, bytes_per_chunk);
    }

    /// Update streaming stats
    pub fn update_streaming_stats(&mut self, stats: StreamingStats) {
        self.streaming_stats = stats;
    }

    /// Update LOD stats
    pub fn update_lod_stats(&mut self, stats: LodStats) {
        self.lod_stats = stats;
    }

    /// Update camera position
    pub fn update_camera(&mut self, camera_pos: Vec3) {
        self.camera_pos = camera_pos;
    }

    /// Get chunk state
    pub fn get_chunk_state(&self, chunk_id: ChunkId) -> ChunkLoadState {
        self.chunk_states
            .get(&chunk_id)
            .copied()
            .unwrap_or(ChunkLoadState::Unloaded)
    }

    /// Get all chunk states
    pub fn get_all_chunk_states(&self) -> &HashMap<ChunkId, ChunkLoadState> {
        &self.chunk_states
    }

    /// Get hitch detector
    pub fn hitch_detector(&self) -> &HitchDetector {
        &self.hitch_detector
    }

    /// Get memory stats
    pub fn memory_stats(&self) -> &MemoryStats {
        &self.memory_stats
    }

    /// Get streaming stats
    pub fn streaming_stats(&self) -> &StreamingStats {
        &self.streaming_stats
    }

    /// Get LOD stats
    pub fn lod_stats(&self) -> &LodStats {
        &self.lod_stats
    }

    /// Get camera position
    pub fn camera_pos(&self) -> Vec3 {
        self.camera_pos
    }

    /// Generate diagnostic report
    pub fn generate_report(&self) -> DiagnosticReport {
        DiagnosticReport {
            frame_stats: FrameStats {
                average_ms: self.hitch_detector.average_frame_time(),
                p99_ms: self.hitch_detector.p99_frame_time(),
                hitch_count: self.hitch_detector.hitch_count(),
                hitch_rate: self.hitch_detector.hitch_rate(),
            },
            memory: self.memory_stats.clone(),
            streaming: self.streaming_stats.clone(),
            lod: LodStatsReport {
                total_chunks: self.lod_stats.total_chunks,
                full_count: self.lod_stats.full_count,
                half_count: self.lod_stats.half_count,
                quarter_count: self.lod_stats.quarter_count,
                skybox_count: self.lod_stats.skybox_count,
                transitioning_count: self.lod_stats.transitioning_count,
            },
            chunk_counts: ChunkCounts {
                loaded: self
                    .chunk_states
                    .values()
                    .filter(|&&s| s == ChunkLoadState::Loaded)
                    .count(),
                loading: self
                    .chunk_states
                    .values()
                    .filter(|&&s| s == ChunkLoadState::Loading)
                    .count(),
                pending: self
                    .chunk_states
                    .values()
                    .filter(|&&s| s == ChunkLoadState::Pending)
                    .count(),
            },
        }
    }
}

/// Diagnostic report (serializable for telemetry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub frame_stats: FrameStats,
    pub memory: MemoryStats,
    pub streaming: StreamingStats,
    pub lod: LodStatsReport,
    pub chunk_counts: ChunkCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameStats {
    pub average_ms: f32,
    pub p99_ms: f32,
    pub hitch_count: usize,
    pub hitch_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodStatsReport {
    pub total_chunks: usize,
    pub full_count: usize,
    pub half_count: usize,
    pub quarter_count: usize,
    pub skybox_count: usize,
    pub transitioning_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkCounts {
    pub loaded: usize,
    pub loading: usize,
    pub pending: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hitch_detector_basic() {
        let mut detector = HitchDetector::new(100, 2.0);

        // Normal frames
        for _ in 0..50 {
            assert!(!detector.record_frame(1.0));
        }

        assert_eq!(detector.hitch_count(), 0);
        assert!(detector.average_frame_time() < 1.5);
    }

    #[test]
    fn hitch_detector_hitches() {
        let mut detector = HitchDetector::new(100, 2.0);

        // Record 10 normal, 1 hitch
        for _ in 0..10 {
            detector.record_frame(1.0);
        }

        assert!(detector.record_frame(5.0)); // Hitch
        assert_eq!(detector.hitch_count(), 1);
        assert!(detector.hitch_rate() > 0.0);
    }

    #[test]
    fn memory_stats() {
        let mut stats = MemoryStats::default();

        stats.update(100, 1024 * 1024); // 100 chunks, 1MB each
        assert_eq!(stats.chunk_count, 100);
        assert_eq!(stats.total_mb(), 100.0);

        stats.update(50, 1024 * 1024); // Drop to 50 chunks
        assert!(stats.delta_from_peak_percent() < 0.0); // Below peak
    }

    #[test]
    fn diagnostics_report() {
        let mut diag = StreamingDiagnostics::new(2.0, 100);

        let loaded = vec![ChunkId::new(0, 0), ChunkId::new(1, 0)];
        let loading = vec![ChunkId::new(2, 0)];
        let pending = vec![ChunkId::new(3, 0), ChunkId::new(4, 0)];

        diag.update_chunk_states(&loaded, &loading, &pending);

        let report = diag.generate_report();
        assert_eq!(report.chunk_counts.loaded, 2);
        assert_eq!(report.chunk_counts.loading, 1);
        assert_eq!(report.chunk_counts.pending, 2);
    }

    // Additional ChunkLoadState tests
    #[test]
    fn test_chunk_load_state_eq() {
        assert_eq!(ChunkLoadState::Loaded, ChunkLoadState::Loaded);
        assert_eq!(ChunkLoadState::Loading, ChunkLoadState::Loading);
        assert_eq!(ChunkLoadState::Pending, ChunkLoadState::Pending);
        assert_eq!(ChunkLoadState::Unloaded, ChunkLoadState::Unloaded);
        assert_ne!(ChunkLoadState::Loaded, ChunkLoadState::Loading);
    }

    #[test]
    fn test_chunk_load_state_clone() {
        let state = ChunkLoadState::Loading;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_chunk_load_state_serialization() {
        let state = ChunkLoadState::Loaded;
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: ChunkLoadState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(state, deserialized);
    }

    // HitchDetector tests
    #[test]
    fn test_hitch_detector_empty() {
        let detector = HitchDetector::new(100, 2.0);
        assert_eq!(detector.average_frame_time(), 0.0);
        assert_eq!(detector.p99_frame_time(), 0.0);
        assert_eq!(detector.hitch_rate(), 0.0);
        assert_eq!(detector.hitch_count(), 0);
    }

    #[test]
    fn test_hitch_detector_single_frame() {
        let mut detector = HitchDetector::new(100, 2.0);
        detector.record_frame(1.5);
        
        assert_eq!(detector.average_frame_time(), 1.5);
        assert_eq!(detector.p99_frame_time(), 1.5);
    }

    #[test]
    fn test_hitch_detector_p99_calculation() {
        let mut detector = HitchDetector::new(100, 50.0);
        
        // Add 99 normal frames and 1 slow frame
        for _ in 0..99 {
            detector.record_frame(10.0);
        }
        detector.record_frame(40.0); // Slow but not hitch
        
        // p99 should be 40.0
        assert_eq!(detector.p99_frame_time(), 40.0);
    }

    #[test]
    fn test_hitch_detector_history_eviction() {
        let mut detector = HitchDetector::new(10, 2.0);
        
        // Add a hitch first
        detector.record_frame(5.0);
        assert_eq!(detector.hitch_count(), 1);
        
        // Add 10 more normal frames (should evict the hitch)
        for _ in 0..10 {
            detector.record_frame(1.0);
        }
        
        // Hitch should have been evicted
        assert_eq!(detector.hitch_count(), 0);
    }

    #[test]
    fn test_hitch_detector_hitch_rate_calculation() {
        let mut detector = HitchDetector::new(100, 2.0);
        
        // Add 90 normal, 10 hitches
        for _ in 0..90 {
            detector.record_frame(1.0);
        }
        for _ in 0..10 {
            detector.record_frame(5.0);
        }
        
        // 10% hitch rate
        assert!((detector.hitch_rate() - 10.0).abs() < 0.1);
    }

    // MemoryStats tests
    #[test]
    fn test_memory_stats_default() {
        let stats = MemoryStats::default();
        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.bytes_per_chunk, 0);
        assert_eq!(stats.chunk_count, 0);
        assert_eq!(stats.peak_bytes, 0);
    }

    #[test]
    fn test_memory_stats_peak_tracking() {
        let mut stats = MemoryStats::default();
        
        // Initial update
        stats.update(100, 1024);
        assert_eq!(stats.peak_bytes, 100 * 1024);
        
        // Lower count
        stats.update(50, 1024);
        assert_eq!(stats.peak_bytes, 100 * 1024); // Peak unchanged
        
        // Higher count
        stats.update(150, 1024);
        assert_eq!(stats.peak_bytes, 150 * 1024); // New peak
    }

    #[test]
    fn test_memory_stats_total_mb() {
        let mut stats = MemoryStats::default();
        stats.update(1, 1024 * 1024); // 1 MB
        assert_eq!(stats.total_mb(), 1.0);
        
        stats.update(512, 2 * 1024); // 1 MB total
        assert_eq!(stats.total_mb(), 1.0);
    }

    #[test]
    fn test_memory_stats_delta_from_peak_zero() {
        let stats = MemoryStats::default();
        // Peak is 0, so delta should be 0
        assert_eq!(stats.delta_from_peak_percent(), 0.0);
    }

    #[test]
    fn test_memory_stats_delta_from_peak_at_peak() {
        let mut stats = MemoryStats::default();
        stats.update(100, 1024);
        // At peak, so delta should be 0
        assert_eq!(stats.delta_from_peak_percent(), 0.0);
    }

    #[test]
    fn test_memory_stats_clone() {
        let mut stats = MemoryStats::default();
        stats.update(100, 1024);
        
        let cloned = stats.clone();
        assert_eq!(stats.total_bytes, cloned.total_bytes);
        assert_eq!(stats.peak_bytes, cloned.peak_bytes);
    }

    #[test]
    fn test_memory_stats_serialization() {
        let mut stats = MemoryStats::default();
        stats.update(100, 1024);
        
        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: MemoryStats = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(stats.total_bytes, deserialized.total_bytes);
    }

    // StreamingDiagnostics tests
    #[test]
    fn test_streaming_diagnostics_new() {
        let diag = StreamingDiagnostics::new(16.67, 100);
        assert!(diag.get_all_chunk_states().is_empty());
        assert_eq!(diag.camera_pos(), Vec3::ZERO);
    }

    #[test]
    fn test_streaming_diagnostics_update_camera() {
        let mut diag = StreamingDiagnostics::new(16.67, 100);
        let pos = Vec3::new(100.0, 50.0, 200.0);
        diag.update_camera(pos);
        assert_eq!(diag.camera_pos(), pos);
    }

    #[test]
    fn test_streaming_diagnostics_record_frame() {
        let mut diag = StreamingDiagnostics::new(16.67, 100);
        
        // Normal frame
        assert!(!diag.record_frame(10.0));
        
        // Hitch frame
        assert!(diag.record_frame(50.0));
    }

    #[test]
    fn test_streaming_diagnostics_update_memory() {
        let mut diag = StreamingDiagnostics::new(16.67, 100);
        diag.update_memory(50, 1024 * 1024);
        
        assert_eq!(diag.memory_stats().chunk_count, 50);
        assert_eq!(diag.memory_stats().total_mb(), 50.0);
    }

    #[test]
    fn test_streaming_diagnostics_get_chunk_state_unloaded() {
        let diag = StreamingDiagnostics::new(16.67, 100);
        let state = diag.get_chunk_state(ChunkId::new(99, 99));
        assert_eq!(state, ChunkLoadState::Unloaded);
    }

    #[test]
    fn test_streaming_diagnostics_update_streaming_stats() {
        let mut diag = StreamingDiagnostics::new(16.67, 100);
        let stats = StreamingStats {
            loaded_chunk_count: 100,
            pending_load_count: 10,
            ..Default::default()
        };
        diag.update_streaming_stats(stats.clone());
        
        assert_eq!(diag.streaming_stats().loaded_chunk_count, 100);
        assert_eq!(diag.streaming_stats().pending_load_count, 10);
    }

    #[test]
    fn test_streaming_diagnostics_update_lod_stats() {
        let mut diag = StreamingDiagnostics::new(16.67, 100);
        let stats = LodStats {
            total_chunks: 50,
            full_count: 20,
            half_count: 15,
            quarter_count: 10,
            skybox_count: 5,
            transitioning_count: 0,
        };
        diag.update_lod_stats(stats.clone());
        
        assert_eq!(diag.lod_stats().total_chunks, 50);
        assert_eq!(diag.lod_stats().full_count, 20);
    }

    #[test]
    fn test_streaming_diagnostics_get_all_chunk_states() {
        let mut diag = StreamingDiagnostics::new(16.67, 100);
        
        let loaded = vec![ChunkId::new(0, 0)];
        let loading = vec![ChunkId::new(1, 1)];
        let pending = vec![];
        
        diag.update_chunk_states(&loaded, &loading, &pending);
        
        let states = diag.get_all_chunk_states();
        assert_eq!(states.len(), 2);
        assert_eq!(states.get(&ChunkId::new(0, 0)), Some(&ChunkLoadState::Loaded));
        assert_eq!(states.get(&ChunkId::new(1, 1)), Some(&ChunkLoadState::Loading));
    }

    #[test]
    fn test_streaming_diagnostics_hitch_detector() {
        let mut diag = StreamingDiagnostics::new(16.67, 100);
        
        for _ in 0..10 {
            diag.record_frame(10.0);
        }
        
        let detector = diag.hitch_detector();
        assert!(detector.average_frame_time() > 0.0);
    }

    // DiagnosticReport tests
    #[test]
    fn test_diagnostic_report_full() {
        let mut diag = StreamingDiagnostics::new(16.67, 100);
        
        // Set up some state
        let loaded = vec![ChunkId::new(0, 0), ChunkId::new(1, 0), ChunkId::new(2, 0)];
        let loading = vec![ChunkId::new(3, 0)];
        let pending = vec![ChunkId::new(4, 0), ChunkId::new(5, 0)];
        diag.update_chunk_states(&loaded, &loading, &pending);
        
        diag.update_memory(50, 1024 * 1024);
        
        for _ in 0..10 {
            diag.record_frame(12.0);
        }
        diag.record_frame(30.0); // 1 hitch
        
        let report = diag.generate_report();
        
        assert_eq!(report.chunk_counts.loaded, 3);
        assert_eq!(report.chunk_counts.loading, 1);
        assert_eq!(report.chunk_counts.pending, 2);
        assert_eq!(report.frame_stats.hitch_count, 1);
        assert_eq!(report.memory.chunk_count, 50);
    }

    #[test]
    fn test_diagnostic_report_serialization() {
        let mut diag = StreamingDiagnostics::new(16.67, 100);
        let report = diag.generate_report();
        
        let serialized = serde_json::to_string(&report).unwrap();
        let deserialized: DiagnosticReport = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(report.chunk_counts.loaded, deserialized.chunk_counts.loaded);
    }

    // FrameStats tests
    #[test]
    fn test_frame_stats_serialization() {
        let stats = FrameStats {
            average_ms: 16.0,
            p99_ms: 20.0,
            hitch_count: 5,
            hitch_rate: 2.5,
        };
        
        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: FrameStats = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(stats.average_ms, deserialized.average_ms);
        assert_eq!(stats.hitch_count, deserialized.hitch_count);
    }

    // LodStatsReport tests
    #[test]
    fn test_lod_stats_report_serialization() {
        let stats = LodStatsReport {
            total_chunks: 100,
            full_count: 50,
            half_count: 30,
            quarter_count: 15,
            skybox_count: 5,
            transitioning_count: 0,
        };
        
        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: LodStatsReport = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(stats.total_chunks, deserialized.total_chunks);
    }

    // ChunkCounts tests
    #[test]
    fn test_chunk_counts_serialization() {
        let counts = ChunkCounts {
            loaded: 100,
            loading: 10,
            pending: 20,
        };
        
        let serialized = serde_json::to_string(&counts).unwrap();
        let deserialized: ChunkCounts = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(counts.loaded, deserialized.loaded);
    }

    #[test]
    fn test_streaming_diagnostics_chunk_state_priority() {
        let mut diag = StreamingDiagnostics::new(16.67, 100);
        
        // Same chunk in multiple lists - last one wins
        let chunk = ChunkId::new(0, 0);
        let loaded = vec![chunk];
        let loading = vec![chunk]; // Same chunk
        let pending = vec![];
        
        diag.update_chunk_states(&loaded, &loading, &pending);
        
        // Loading should override loaded since it's inserted later
        assert_eq!(diag.get_chunk_state(chunk), ChunkLoadState::Loading);
    }
}

