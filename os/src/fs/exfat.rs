#[repr(C, packed)]
pub struct ExFatBootSector {
    pub jump_boot: [u8; 3],            // Jump instruction to boot code
    pub fs_name: [u8; 8],              // File system name ("EXFAT   ")
    pub must_be_zero: [u8; 53],        // Reserved, must be zero
    pub partition_offset: u64,         // Offset of the partition on the disk
    pub volume_length: u64,            // Total number of sectors in the volume
    pub fat_offset: u32,               // Sector offset of the FAT
    pub fat_length: u32,               // Length of the FAT in sectors
    pub cluster_heap_offset: u32,      // Sector offset of the Cluster Heap
    pub cluster_count: u32,            // Total number of clusters
    pub root_dir_cluster: u32,         // Cluster of the root directory
    pub volume_serial_number: u32,     // Unique serial number
    pub fs_revision: u16,              // File system version
    pub volume_flags: u16,             // Flags (dirty, etc.)
    pub bytes_per_sector_shift: u8,    // Sector size (2^n bytes per sector)
    pub sectors_per_cluster_shift: u8, // Cluster size (2^n sectors per cluster)
    pub number_of_fats: u8,            // Number of FATs
    pub drive_select: u8,              // Drive select
    pub percent_in_use: u8,            // Percent of volume in use
    pub reserved: [u8; 7],             // Reserved, must be zero
    pub boot_code: [u8; 390],          // Boot code (not used in exFAT)
    pub boot_signature: u16,           // Boot sector signature (0xAA55)
}

#[repr(C, packed)]
pub struct ExFatFileEntry {
    entry_type: u8,      // Entry type (e.g., file, directory)
    secondary_count: u8, // Number of secondary entries
    name_length: u8,
    name_hash: u16,
    first_cluster: u32, // Start cluster of file data
    data_length: u64,   // File size
}

pub type Cluster = u32;
pub const EXFAT_EOF: Cluster = 0xFFFFFFFF; // End of File marker

pub struct ExFatFAT<'a> {
    pub fat_data: &'a [u8], // Raw FAT data
}

impl<'a> ExFatFAT<'a> {
    pub fn next_cluster(&self, cluster: Cluster) -> Option<Cluster> {
        let entry_offset = cluster as usize * 4;
        let next_cluster = u32::from_le_bytes(
            self.fat_data[entry_offset..entry_offset + 4]
                .try_into()
                .unwrap(),
        );
        if next_cluster == EXFAT_EOF {
            None
        } else {
            Some(next_cluster)
        }
    }
}

pub struct AllocationBitmap<'a> {
    pub bitmap_data: &'a mut [u8], // Raw bitmap data
}

impl<'a> AllocationBitmap<'a> {
    pub fn is_allocated(&self, cluster: Cluster) -> bool {
        let byte_index = (cluster / 8) as usize;
        let bit_index = (cluster % 8) as u8;
        (self.bitmap_data[byte_index] & (1 << bit_index)) != 0
    }

    pub fn allocate_cluster(&mut self, cluster: Cluster) {
        let byte_index = (cluster / 8) as usize;
        let bit_index = (cluster % 8) as u8;
        self.bitmap_data[byte_index] |= 1 << bit_index;
    }

    pub fn free_cluster(&mut self, cluster: Cluster) {
        let byte_index = (cluster / 8) as usize;
        let bit_index = (cluster % 8) as u8;
        self.bitmap_data[byte_index] &= !(1 << bit_index);
    }
}
