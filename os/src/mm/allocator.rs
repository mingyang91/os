use core::{
    alloc::{Layout, LayoutError},
    marker::PhantomData,
    ptr::NonNull,
};

use buddy_system_allocator::LockedHeap;

use super::{AlignSize, Mode, PageTableSpec};

pub static FRAME_ALLOCATOR: FrameAllocator<Mode> = FrameAllocator(LockedHeap::empty(), PhantomData);

pub struct FrameAllocator<M>(LockedHeap<32>, PhantomData<M>);

#[derive(Debug)]
pub enum Error {
    LayoutError(LayoutError),
    OutOfMemory,
}

impl<M> FrameAllocator<M>
where
    M: PageTableSpec,
{
    pub fn init(&self, start: usize, size: usize) {
        let mut heap = self.0.lock();
        unsafe { heap.init(start, size) }
    }

    pub fn alloc(&self, size: usize) -> Result<Frame, Error> {
        let align = Self::fit_align_from_size(size);
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

    fn fit_align_from_size(size: usize) -> usize {
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
