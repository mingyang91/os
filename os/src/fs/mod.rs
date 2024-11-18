pub mod exfat;
pub mod virtio_blk;

use core::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum StorageError {
    /// An error occurred while sending a command to the device
    CommandFailed { command: u8, error_code: u32 },

    /// The requested block was out of range
    OutOfBounds { block_num: u64 },

    /// Data could not be read or written due to hardware error
    HardwareFault,

    /// Timeout occurred during a read or write operation
    Timeout,

    /// Data read from the device was invalid or corrupted
    DataCorruption,

    /// An unknown error occurred
    Unknown,
}

impl Display for StorageError {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        match self {
            StorageError::CommandFailed {
                command,
                error_code,
            } => {
                write!(
                    f,
                    "Command 0x{:02X} failed with error code: {}",
                    command, error_code
                )
            }
            StorageError::OutOfBounds { block_num } => {
                write!(f, "Block number {} is out of bounds", block_num)
            }
            StorageError::HardwareFault => {
                write!(f, "Hardware fault occurred during storage operation")
            }
            StorageError::Timeout => {
                write!(f, "Timeout occurred during storage operation")
            }
            StorageError::DataCorruption => {
                write!(f, "Data corruption detected during storage operation")
            }
            StorageError::Unknown => {
                write!(f, "An unknown error occurred")
            }
        }
    }
}

pub trait StorageDevice {
    fn read_block(&self, block_num: u64, buffer: &mut [u8]) -> Result<(), StorageError>;
    fn write_block(&self, block_num: u64, buffer: &[u8]) -> Result<(), StorageError>;
}
