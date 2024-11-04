use core::{
    alloc::{Layout, LayoutError},
    ptr::NonNull,
};

use buddy_system_allocator::LockedHeap;

use super::{AlignSize, PageTableSpec};

pub static FRAME_ALLOCATOR: FrameAllocator = FrameAllocator(LockedHeap::empty());

pub struct FrameAllocator(LockedHeap<32>);

#[derive(Debug)]
pub enum Error {
    LayoutError(LayoutError),
    OutOfMemory,
}

impl FrameAllocator {
    pub fn init(&self, start: usize, size: usize) {
        let mut heap = self.0.lock();
        unsafe { heap.init(start, size) }
    }

    pub fn alloc<S>(&self, size: usize) -> Result<Frame, Error>
    where
        S: PageTableSpec,
    {
        let align = Self::fit_align_from_size::<S>(size);
        let layout = Layout::from_size_align(size, align).map_err(Error::LayoutError)?;
        let mut heap = self.0.lock();
        heap.alloc(layout)
            .map(|ptr| Frame { ptr, layout })
            .map_err(|_| Error::OutOfMemory)
    }

    fn dealloc(&self, frame: &Frame) {
        let mut heap = self.0.lock();
        heap.dealloc(frame.ptr, frame.layout);
    }

    fn fit_align_from_size<S>(size: usize) -> usize
    where
        S: PageTableSpec,
    {
        if size <= AlignSize::Page2M as usize / 2 {
            AlignSize::Page4K as usize
        } else if size <= AlignSize::Page1G as usize / 2 {
            AlignSize::Page2M as usize
        } else {
            AlignSize::Page1G as usize
        }
    }
}

pub struct Frame {
    pub ptr: NonNull<u8>,
    pub layout: Layout,
}

impl Drop for Frame {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.dealloc(self);
    }
}
