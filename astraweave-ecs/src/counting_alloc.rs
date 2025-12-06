//! Counting allocator for zero-alloc hot path validation.
//!
//! This module provides a global allocator wrapper that counts heap allocations,
//! allowing tests and benchmarks to assert zero allocations in hot paths.
//!
//! # Usage
//!
//! In test files, enable the counting allocator with `--features alloc-counter`:
//!
//! ```rust,ignore
//! use astraweave_ecs::counting_alloc::{reset_allocs, allocs};
//!
//! reset_allocs();
//! // ... run hot path code ...
//! assert_eq!(allocs(), 0, "Hot path should not allocate");
//! ```
//!
//! # Note
//!
//! This is only enabled with the `alloc-counter` feature to avoid overhead in
//! production builds.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global allocation counter.
static ALLOCS: AtomicUsize = AtomicUsize::new(0);

/// Global deallocation counter.
static DEALLOCS: AtomicUsize = AtomicUsize::new(0);

/// Counting allocator that wraps the system allocator.
///
/// When registered as `#[global_allocator]`, this tracks all heap allocations
/// and deallocations, enabling zero-alloc assertions in tests.
pub struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCS.fetch_add(1, Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        System.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // realloc counts as both a dealloc and an alloc
        DEALLOCS.fetch_add(1, Ordering::Relaxed);
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        System.realloc(ptr, layout, new_size)
    }
}

/// Get the current allocation count since last reset.
#[inline]
pub fn allocs() -> usize {
    ALLOCS.load(Ordering::Relaxed)
}

/// Get the current deallocation count since last reset.
#[inline]
pub fn deallocs() -> usize {
    DEALLOCS.load(Ordering::Relaxed)
}

/// Reset both allocation and deallocation counters to zero.
#[inline]
pub fn reset_allocs() {
    ALLOCS.store(0, Ordering::Relaxed);
    DEALLOCS.store(0, Ordering::Relaxed);
}

/// Get net allocations (allocs - deallocs). Useful for leak detection.
#[inline]
pub fn net_allocs() -> isize {
    allocs() as isize - deallocs() as isize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counting_functions() {
        // Just verify the functions work (actual counting requires global allocator)
        let _ = allocs();
        let _ = deallocs();
        let _ = net_allocs();
        reset_allocs();
    }
}
