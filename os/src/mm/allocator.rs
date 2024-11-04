use slab;

static mut SLAB: slab::Slab<usize> = slab::Slab::<usize>::new();

pub unsafe fn test() {
    SLAB.insert(114514);
}
