pub mod allocator;

use core::{marker::PhantomData, ptr::write_volatile};

use bitflags::bitflags;

const PAGE_SIZE: usize = 4096;
const PN_BITS: usize = 9;
const PAGE_OFFSET_BITS: usize = 12;
const RSW_BITS: usize = 2;
const PTE_FLAGS_BITS: usize = 8;

mod pte_mask {
    #![allow(dead_code)]
    pub const FLAGS_MASK: usize =
        0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111;
    pub const RSW_MASK: usize =
        0b00000000_00000000_00000000_00000000_00000000_00000000_00000011_00000000;
    pub const PPN_0_MASK: usize =
        0b00000000_00000000_00000000_00000000_00000000_00000111_11111100_00000000;
    pub const PPN_1_MASK: usize =
        0b00000000_00000000_00000000_00000000_00001111_11111000_00000000_00000000;
    pub const PPN_2_MASK: usize =
        0b00000000_00000000_00000000_00011111_11110000_00000000_00000000_00000000;
    pub const PPN_3_MASK: usize =
        0b00000000_00000000_00111111_11100000_00000000_00000000_00000000_00000000;
    pub const PPN_4_MASK: usize =
        0b00000000_01111111_11000000_00000000_00000000_00000000_00000000_00000000;
}

mod addr_mask {
    #![allow(dead_code)]
    pub const OFFSET_MASK: usize =
        0b00000000_00000000_00000000_00000000_00000000_00000000_00001111_11111111;
    pub const PN_0_MASK: usize =
        0b00000000_00000000_00000000_00000000_00000000_00011111_11110000_00000000;
    pub const PN_1_MASK: usize =
        0b00000000_00000000_00000000_00000000_00111111_11100000_00000000_00000000;
    pub const PN_2_MASK: usize =
        0b00000000_00000000_00000000_01111111_11000000_00000000_00000000_00000000;
    pub const PN_3_MASK: usize =
        0b00000000_00000000_11111111_10000000_00000000_00000000_00000000_00000000;
    pub const PN_4_MASK: usize =
        0b00000001_11111111_00000000_00000000_00000000_00000000_00000000_00000000;
}

mod satp_mask {
    #![allow(dead_code)]
    pub const PPN_BITS: usize = 44;
    pub const ASID_BITS: usize = 16;
    pub const MODE_BITS: usize = 4;
    pub const PPN_MASK:  usize = (1 << PPN_BITS) - 1;
    pub const ASID_MASK: usize = ((1 << ASID_BITS) - 1) << PPN_BITS;
    pub const MODE_MASK: usize = ((1 << MODE_BITS) - 1) << (PPN_BITS + ASID_BITS);
}

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

pub const KERNEL_PTE_FLAGS: PageTableEntryFlags = PageTableEntryFlags::from_bits_truncate(
    PageTableEntryFlags::V.bits()
        | PageTableEntryFlags::R.bits()
        | PageTableEntryFlags::W.bits()
        | PageTableEntryFlags::X.bits()
        | PageTableEntryFlags::A.bits()
        | PageTableEntryFlags::D.bits(),
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlignSize {
    Page4K = 8 << 9,
    Page2M = 8 << (9 * 2),
    Page1G = 8 << (9 * 3),
    Page512G = 8 << (9 * 4),
}

#[repr(C, align(4096))]
pub struct PageTable([PageTableEntry; 512]);

impl PageTable {
    pub const fn zero() -> PageTable {
        PageTable([PageTableEntry::zero(); 512])
    }

    #[inline]
    pub fn ppn(&self) -> usize {
        core::ptr::addr_of!(self) as *const _ as usize >> 12
    }
}

#[repr(C, align(4096))]
pub struct RootPageTable<S: PageTableSpec>(PageTable, core::marker::PhantomData<S>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    AddressNotAligned,
    OutOfMemory,
}

impl<S: PageTableSpec> RootPageTable<S> {
    pub const fn zero() -> RootPageTable<S> {
        RootPageTable(PageTable::zero(), PhantomData)
    }

    #[inline]
    pub fn satp(&self, asid: usize) -> usize {
        const { assert!(S::MODE < 1 << 4); }
        (S::MODE << 60) | ((asid << 44) & satp_mask::ASID_MASK) | self.0.ppn()
    }

    #[inline]
    pub fn active(&self, asid: usize) {
        riscv::register::satp::write(self.satp(asid));
        riscv::asm::sfence_vma_all();
    }

    #[inline]
    pub unsafe fn map(
        &mut self,
        virt_addr: Address,
        phy_addr: Address,
        align_size: AlignSize,
        flags: PageTableEntryFlags,
    ) -> Result<(), Error> {
        if S::LEVEL < 4 && align_size >= AlignSize::Page512G {
            return Err(Error::OutOfMemory);
        }

        if S::LEVEL < 3 && align_size >= AlignSize::Page1G {
            return Err(Error::OutOfMemory);
        }

        if S::LEVEL < 2 && align_size >= AlignSize::Page2M {
            return Err(Error::OutOfMemory);
        }

        if align_size == AlignSize::Page1G && S::LEVEL == 3 {
            if !virt_addr.is_aligned::<Align1G>() || !phy_addr.is_aligned::<Align1G>() {
                return Err(Error::AddressNotAligned);
            }
            let vpn_2 = virt_addr.pn_2();
            let mut pte = PageTableEntry::zero();
            pte.set_ppn_2(phy_addr.pn_2());
            pte.set_flags(flags);
            write_volatile(&mut self.0 .0[vpn_2], pte);
            return Ok(());
        }

        todo!()
    }
}

#[derive(Clone, Copy)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub const fn zero() -> PageTableEntry {
        PageTableEntry(0)
    }

    #[inline]
    pub fn new(ppn: usize, flags: PageTableEntryFlags) -> PageTableEntry {
        PageTableEntry(ppn << 10 | flags.bits())
    }

    #[inline]
    pub const fn flags(&self) -> PageTableEntryFlags {
        PageTableEntryFlags::from_bits_truncate(self.0)
    }

    #[inline]
    pub const fn set_flags(&mut self, flags: PageTableEntryFlags) {
        self.0 = (self.0 & !pte_mask::FLAGS_MASK) | flags.bits();
    }

    #[inline]
    pub const fn ppn_0(&self) -> usize {
        self.0 & pte_mask::PPN_0_MASK >> (PTE_FLAGS_BITS + RSW_BITS)
    }

    #[inline]
    pub const fn set_ppn_0(&mut self, ppn: usize) {
        self.0 = (self.0 & !pte_mask::PPN_0_MASK)
            | (ppn << (PTE_FLAGS_BITS + RSW_BITS) & pte_mask::PPN_0_MASK);
    }

    #[inline]
    pub const fn ppn_1(&self) -> usize {
        self.0 & pte_mask::PPN_1_MASK >> (PTE_FLAGS_BITS + RSW_BITS + PN_BITS)
    }

    #[inline]
    pub const fn set_ppn_1(&mut self, ppn: usize) {
        self.0 = (self.0 & !pte_mask::PPN_1_MASK)
            | (ppn << (PTE_FLAGS_BITS + RSW_BITS + PN_BITS) & pte_mask::PPN_1_MASK);
    }

    #[inline]
    pub const fn ppn_2(&self) -> usize {
        self.0 & pte_mask::PPN_2_MASK >> (PTE_FLAGS_BITS + RSW_BITS + PN_BITS * 2)
    }

    #[inline]
    pub const fn set_ppn_2(&mut self, ppn: usize) {
        self.0 = (self.0 & !pte_mask::PPN_2_MASK)
            | (ppn << (PTE_FLAGS_BITS + RSW_BITS + PN_BITS * 2) & pte_mask::PPN_2_MASK);
    }

    #[inline]
    pub const fn ppn_3(&self) -> usize {
        self.0 & pte_mask::PPN_3_MASK >> (PTE_FLAGS_BITS + RSW_BITS + PN_BITS * 3)
    }

    #[inline]
    pub const fn set_ppn_3(&mut self, ppn: usize) {
        self.0 = (self.0 & !pte_mask::PPN_3_MASK)
            | (ppn << (PTE_FLAGS_BITS + RSW_BITS + PN_BITS * 3) & pte_mask::PPN_3_MASK);
    }

    #[inline]
    pub const fn ppn_4(&self) -> usize {
        self.0 & pte_mask::PPN_4_MASK >> (PTE_FLAGS_BITS + RSW_BITS + PN_BITS * 4)
    }

    #[inline]
    pub const fn set_ppn_4(&mut self, ppn: usize) {
        self.0 = (self.0 & !pte_mask::PPN_4_MASK)
            | (ppn << (PTE_FLAGS_BITS + RSW_BITS + PN_BITS * 4) & pte_mask::PPN_4_MASK);
    }

    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.flags().contains(PageTableEntryFlags::V)
    }

    #[inline]
    pub const fn is_leaf(&self) -> bool {
        const NO_LEAF: usize = PageTableEntryFlags::R.bits()
            | PageTableEntryFlags::W.bits()
            | PageTableEntryFlags::X.bits();
        !self
            .flags()
            .contains(PageTableEntryFlags::from_bits_truncate(NO_LEAF))
    }
}

pub trait AlignCheck {
    const ALIGN_SIZE: usize;
}

pub struct Unaligned;

impl AlignCheck for Unaligned {
    const ALIGN_SIZE: usize = 0;
}

pub struct Align4K;

impl AlignCheck for Align4K {
    const ALIGN_SIZE: usize = PAGE_SIZE;
}

pub struct Align2M;
impl AlignCheck for Align2M {
    const ALIGN_SIZE: usize = PAGE_SIZE << PN_BITS;
}

pub struct Align1G;
impl AlignCheck for Align1G {
    const ALIGN_SIZE: usize = PAGE_SIZE << (PN_BITS * 2);
}

pub struct Align512G;
impl AlignCheck for Align512G {
    const ALIGN_SIZE: usize = PAGE_SIZE << (PN_BITS * 3);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address<A: AlignCheck = Unaligned>(usize, core::marker::PhantomData<A>);

impl Address {
    #[inline]
    pub const fn new(addr: usize) -> Address<Unaligned> {
        Address(addr, PhantomData)
    }
}

impl<A: AlignCheck> Address<A> {
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.0 & addr_mask::OFFSET_MASK
    }

    #[inline]
    pub fn pn_0(&self) -> usize {
        (self.0 & addr_mask::PN_0_MASK) >> PAGE_OFFSET_BITS
    }

    #[inline]
    pub fn pn_1(&self) -> usize {
        (self.0 & addr_mask::PN_1_MASK) >> (PAGE_OFFSET_BITS + PN_BITS)
    }

    #[inline]
    pub fn pn_2(&self) -> usize {
        (self.0 & addr_mask::PN_2_MASK) >> (PAGE_OFFSET_BITS + 2 * PN_BITS)
    }

    #[inline]
    pub fn pn_3(&self) -> usize {
        (self.0 & addr_mask::PN_3_MASK) >> (PAGE_OFFSET_BITS + 3 * PN_BITS)
    }

    #[inline]
    pub fn pn_4(&self) -> usize {
        (self.0 & addr_mask::PN_4_MASK) >> (PAGE_OFFSET_BITS + 4 * PN_BITS)
    }

    #[inline]
    pub fn is_aligned<T: AlignCheck>(&self) -> bool {
        self.0 & (T::ALIGN_SIZE - 1) == 0
    }

    #[inline]
    pub fn check_alignment<T: AlignCheck>(self) -> Result<Address<A>, Error> {
        if self.is_aligned::<T>() {
            Ok(Address(self.0, PhantomData))
        } else {
            Err(Error::AddressNotAligned)
        }
    }
}

pub trait PageTableSpec {
    const MODE: usize;
    const LEVEL: usize;
}

pub struct RV39;

impl PageTableSpec for RV39 {
    const MODE: usize = 8;
    const LEVEL: usize = 3;
}
