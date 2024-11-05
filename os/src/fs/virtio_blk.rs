use crate::mm;
use spin::Mutex;
use virtio_drivers::{device::blk::VirtIOBlk, transport::mmio::MmioTransport, Hal};

pub struct VirtIOBlock(Mutex<VirtIOBlk<VirtIOBlockHal, MmioTransport>>);

struct VirtIOBlockHal;
unsafe impl Hal for VirtIOBlockHal {
    fn dma_alloc(
        pages: usize,
        direction: virtio_drivers::BufferDirection,
    ) -> (virtio_drivers::PhysAddr, core::ptr::NonNull<u8>) {
        let frame = mm::allocator::FRAME_ALLOCATOR
            .alloc(pages)
            .expect("Failed to allocate DMA frame");
        let paddr = frame.ptr.as_ptr() as usize;
        let vaddr = frame.ptr;
        (paddr, vaddr)
    }

    unsafe fn dma_dealloc(
        paddr: virtio_drivers::PhysAddr,
        vaddr: core::ptr::NonNull<u8>,
        pages: usize,
    ) -> i32 {
        todo!()
    }

    unsafe fn mmio_phys_to_virt(
        paddr: virtio_drivers::PhysAddr,
        size: usize,
    ) -> core::ptr::NonNull<u8> {
        todo!()
    }

    unsafe fn share(
        buffer: core::ptr::NonNull<[u8]>,
        direction: virtio_drivers::BufferDirection,
    ) -> virtio_drivers::PhysAddr {
        todo!()
    }

    unsafe fn unshare(
        paddr: virtio_drivers::PhysAddr,
        buffer: core::ptr::NonNull<[u8]>,
        direction: virtio_drivers::BufferDirection,
    ) {
        todo!()
    }
}
