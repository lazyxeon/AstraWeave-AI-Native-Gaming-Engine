// SPDX-License-Identifier: MIT
//! Type-erased contiguous storage for components
//! 
//! Inspired by Bevy's BlobVec, this provides cache-friendly storage without
//! the overhead of Box<dyn Any> indirection. Components are stored as raw bytes
//! with proper alignment and drop handling.

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ptr::{self, NonNull};

/// Type-erased vector of components stored contiguously in memory.
/// 
/// This is the foundation for high-performance ECS storage, providing:
/// - Zero heap indirection (vs Box<dyn Any>)
/// - SIMD-friendly contiguous memory
/// - Cache-friendly iteration
/// - Proper drop handling via function pointer
pub struct BlobVec {
    /// Raw pointer to the start of component data
    data: NonNull<u8>,
    /// Number of components currently stored
    len: usize,
    /// Allocated capacity (in number of components)
    capacity: usize,
    /// Memory layout of a single component
    item_layout: Layout,
    /// Function to drop a single component
    /// SAFETY: Must be called with a valid pointer to T
    drop_fn: Option<unsafe fn(*mut u8)>,
}

impl BlobVec {
    /// Create a new BlobVec for components of type T
    pub fn new<T: 'static>() -> Self {
        let item_layout = Layout::new::<T>();
        let drop_fn = if std::mem::needs_drop::<T>() {
            Some(Self::drop_fn_of::<T>() as unsafe fn(*mut u8))
        } else {
            None
        };
        
        Self {
            data: NonNull::dangling(),
            len: 0,
            capacity: 0,
            item_layout,
            drop_fn,
        }
    }
    
    /// Create a new BlobVec with specified capacity
    pub fn with_capacity<T: 'static>(capacity: usize) -> Self {
        let mut blob = Self::new::<T>();
        if capacity > 0 {
            blob.reserve(capacity);
        }
        blob
    }
    
    /// Get the drop function for type T
    fn drop_fn_of<T>() -> unsafe fn(*mut u8) {
        |ptr| unsafe {
            ptr.cast::<T>().drop_in_place();
        }
    }
    
    /// Reserve space for at least `additional` more components
    pub fn reserve(&mut self, additional: usize) {
        let required_cap = self.len.checked_add(additional).expect("capacity overflow");
        if required_cap <= self.capacity {
            return;
        }
        
        let new_capacity = required_cap.max(self.capacity * 2).max(4);
        
        let new_layout = Layout::from_size_align(
            self.item_layout.size() * new_capacity,
            self.item_layout.align(),
        ).expect("invalid layout");
        
        let new_data = if self.capacity == 0 {
            // First allocation
            unsafe { NonNull::new(alloc(new_layout)).expect("allocation failed") }
        } else {
            // Reallocate existing memory
            let old_layout = Layout::from_size_align(
                self.item_layout.size() * self.capacity,
                self.item_layout.align(),
            ).expect("invalid layout");
            
            unsafe {
                let new_ptr = realloc(
                    self.data.as_ptr(),
                    old_layout,
                    new_layout.size(),
                );
                NonNull::new(new_ptr).expect("reallocation failed")
            }
        };
        
        self.data = new_data;
        self.capacity = new_capacity;
    }
    
    /// Push a component onto the end of the vector
    /// 
    /// # Safety
    /// Type T must match the type this BlobVec was created for
    pub unsafe fn push<T>(&mut self, value: T) {
        if self.len == self.capacity {
            self.reserve(1);
        }
        
        let ptr = self.data.as_ptr().add(self.len * self.item_layout.size());
        ptr.cast::<T>().write(value);
        self.len += 1;
    }
    
    /// Get a reference to a component at the specified index
    /// 
    /// # Safety
    /// - Index must be in bounds
    /// - Type T must match the type this BlobVec was created for
    pub unsafe fn get<T>(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        
        let ptr = self.data.as_ptr().add(index * self.item_layout.size());
        Some(&*ptr.cast::<T>())
    }
    
    /// Get a mutable reference to a component at the specified index
    /// 
    /// # Safety
    /// - Index must be in bounds
    /// - Type T must match the type this BlobVec was created for
    pub unsafe fn get_mut<T>(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        
        let ptr = self.data.as_ptr().add(index * self.item_layout.size());
        Some(&mut *ptr.cast::<T>())
    }
    
    /// Get the component data as a contiguous slice
    /// 
    /// This is the key to high-performance iteration - direct memory access!
    /// 
    /// # Safety
    /// Type T must match the type this BlobVec was created for
    pub unsafe fn as_slice<T>(&self) -> &[T] {
        if self.len == 0 {
            return &[];
        }
        std::slice::from_raw_parts(self.data.as_ptr().cast::<T>(), self.len)
    }
    
    /// Get the component data as a mutable contiguous slice
    /// 
    /// # Safety
    /// Type T must match the type this BlobVec was created for
    pub unsafe fn as_slice_mut<T>(&mut self) -> &mut [T] {
        if self.len == 0 {
            return &mut [];
        }
        std::slice::from_raw_parts_mut(self.data.as_ptr().cast::<T>(), self.len)
    }
    
    /// Remove and return the component at the specified index
    /// Uses swap_remove for O(1) performance (order not preserved)
    /// 
    /// # Safety
    /// - Index must be in bounds
    /// - Type T must match the type this BlobVec was created for
    pub unsafe fn swap_remove<T>(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        
        let last_index = self.len - 1;
        let ptr = self.data.as_ptr().add(index * self.item_layout.size()).cast::<T>();
        
        if index != last_index {
            // Swap with last element
            let last_ptr = self.data.as_ptr().add(last_index * self.item_layout.size()).cast::<T>();
            ptr::swap(ptr, last_ptr);
        }
        
        self.len -= 1;
        ptr::read(self.data.as_ptr().add(self.len * self.item_layout.size()).cast::<T>())
    }
    
    /// Get the number of components stored
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// Check if the BlobVec is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// Get the current capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Clear all components, calling their drop functions
    pub fn clear(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            for i in 0..self.len {
                unsafe {
                    let ptr = self.data.as_ptr().add(i * self.item_layout.size());
                    drop_fn(ptr);
                }
            }
        }
        self.len = 0;
    }
}

impl Drop for BlobVec {
    fn drop(&mut self) {
        self.clear();
        
        if self.capacity > 0 {
            let layout = Layout::from_size_align(
                self.item_layout.size() * self.capacity,
                self.item_layout.align(),
            ).expect("invalid layout");
            
            unsafe {
                dealloc(self.data.as_ptr(), layout);
            }
        }
    }
}

unsafe impl Send for BlobVec {}
unsafe impl Sync for BlobVec {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }
    
    #[derive(Debug, PartialEq)]
    struct DropTest {
        value: i32,
        dropped: std::rc::Rc<std::cell::Cell<bool>>,
    }
    
    impl Drop for DropTest {
        fn drop(&mut self) {
            self.dropped.set(true);
        }
    }
    
    #[test]
    fn test_push_and_get() {
        let mut blob = BlobVec::new::<Position>();
        
        unsafe {
            blob.push(Position { x: 1.0, y: 2.0, z: 3.0 });
            blob.push(Position { x: 4.0, y: 5.0, z: 6.0 });
        }
        
        assert_eq!(blob.len(), 2);
        
        unsafe {
            let pos1 = blob.get::<Position>(0).unwrap();
            assert_eq!(pos1.x, 1.0);
            assert_eq!(pos1.y, 2.0);
            
            let pos2 = blob.get::<Position>(1).unwrap();
            assert_eq!(pos2.x, 4.0);
            assert_eq!(pos2.y, 5.0);
        }
    }
    
    #[test]
    fn test_as_slice() {
        let mut blob = BlobVec::new::<Position>();
        
        unsafe {
            blob.push(Position { x: 1.0, y: 2.0, z: 3.0 });
            blob.push(Position { x: 4.0, y: 5.0, z: 6.0 });
            blob.push(Position { x: 7.0, y: 8.0, z: 9.0 });
            
            let slice = blob.as_slice::<Position>();
            assert_eq!(slice.len(), 3);
            assert_eq!(slice[0].x, 1.0);
            assert_eq!(slice[1].x, 4.0);
            assert_eq!(slice[2].x, 7.0);
        }
    }
    
    #[test]
    fn test_as_slice_mut() {
        let mut blob = BlobVec::new::<Position>();
        
        unsafe {
            blob.push(Position { x: 1.0, y: 2.0, z: 3.0 });
            blob.push(Position { x: 4.0, y: 5.0, z: 6.0 });
            
            let slice = blob.as_slice_mut::<Position>();
            slice[0].x = 100.0;
            slice[1].y = 200.0;
            
            let slice = blob.as_slice::<Position>();
            assert_eq!(slice[0].x, 100.0);
            assert_eq!(slice[1].y, 200.0);
        }
    }
    
    #[test]
    fn test_swap_remove() {
        let mut blob = BlobVec::new::<Position>();
        
        unsafe {
            blob.push(Position { x: 1.0, y: 2.0, z: 3.0 });
            blob.push(Position { x: 4.0, y: 5.0, z: 6.0 });
            blob.push(Position { x: 7.0, y: 8.0, z: 9.0 });
            
            let removed = blob.swap_remove::<Position>(1);
            assert_eq!(removed.x, 4.0);
            
            assert_eq!(blob.len(), 2);
            
            // Element at index 1 should now be the former last element
            let pos = blob.get::<Position>(1).unwrap();
            assert_eq!(pos.x, 7.0);
        }
    }
    
    #[test]
    fn test_drop_handling() {
        let dropped = std::rc::Rc::new(std::cell::Cell::new(false));
        
        {
            let mut blob = BlobVec::new::<DropTest>();
            
            unsafe {
                blob.push(DropTest {
                    value: 42,
                    dropped: dropped.clone(),
                });
            }
            
            assert!(!dropped.get());
        } // BlobVec dropped here
        
        assert!(dropped.get());
    }
    
    #[test]
    fn test_clear() {
        let dropped1 = std::rc::Rc::new(std::cell::Cell::new(false));
        let dropped2 = std::rc::Rc::new(std::cell::Cell::new(false));
        
        let mut blob = BlobVec::new::<DropTest>();
        
        unsafe {
            blob.push(DropTest { value: 1, dropped: dropped1.clone() });
            blob.push(DropTest { value: 2, dropped: dropped2.clone() });
        }
        
        assert!(!dropped1.get());
        assert!(!dropped2.get());
        
        blob.clear();
        
        assert!(dropped1.get());
        assert!(dropped2.get());
        assert_eq!(blob.len(), 0);
    }
    
    #[test]
    fn test_reserve() {
        let mut blob = BlobVec::new::<Position>();
        assert_eq!(blob.capacity(), 0);
        
        blob.reserve(10);
        assert!(blob.capacity() >= 10);
        
        let old_capacity = blob.capacity();
        blob.reserve(5); // Should not reallocate
        assert_eq!(blob.capacity(), old_capacity);
    }
}
