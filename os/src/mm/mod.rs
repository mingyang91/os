use bitflags::bitflags;

bitflags! {
    pub struct PageTableEntryFlags: usize {
        const V = 0b00000001;
        const R = 0b00000010;
        const W = 0b00000100;
        const X = 0b00001000;
        const U = 0b00010000;
        const G = 0b00100000;
        const A = 0b01000000;
        const D = 0b10000000;
    }
}

pub const KERNEL_PTE_FLAGS: usize = PageTableEntryFlags::V.bits()
    | PageTableEntryFlags::R.bits()
    | PageTableEntryFlags::W.bits()
    | PageTableEntryFlags::X.bits()
    | PageTableEntryFlags::A.bits()
    | PageTableEntryFlags::D.bits();
