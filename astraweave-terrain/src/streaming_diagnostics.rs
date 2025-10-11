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
}
