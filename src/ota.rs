//! OTA (Over-The-Air) firmware update module
//!
//! This module handles receiving firmware updates over USB and writing them to flash.
//! The flash write operations are blocking, so HID reports are paused during OTA
//! to prevent timeout panics.
//!
//! This module uses esp-storage directly with esp-bootloader-esp-idf for partition
//! management and boot selection.

use core::sync::atomic::{AtomicBool, Ordering};

use embedded_storage::Storage;
use esp_bootloader_esp_idf::ota::OtaImageState;
use esp_bootloader_esp_idf::ota_updater::OtaUpdater;
use esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN;
use log::{error, info, warn};

use crate::config::with_flash_storage;

/// Global flag indicating OTA is in progress.
/// When true, HID report sending should be skipped to avoid timeout panics
/// during blocking flash write operations.
pub static OTA_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// OTA error types
#[derive(Debug, Clone, Copy)]
pub enum OtaError {
    /// OTA is already in progress
    AlreadyInProgress,
    /// OTA has not been started
    NotStarted,
    /// Failed to initialize OTA
    InitFailed,
    /// Failed to write chunk to flash
    WriteFailed,
    /// CRC verification failed
    CrcMismatch,
    /// Failed to finalize OTA
    FlushFailed,
    /// Invalid firmware size
    InvalidSize,
    /// Partition not found
    PartitionNotFound,
}

/// OTA state machine
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum OtaState {
    /// No OTA in progress
    Idle,
    /// Receiving firmware data
    Receiving {
        total_size: u32,
        received: u32,
        target_crc: u32,
    },
    /// OTA completed successfully, ready to reboot
    Complete,
    /// OTA failed with error
    Error(OtaError),
}

/// OTA Manager handles the firmware update process
pub struct OtaManager {
    state: OtaState,
    last_progress_log: u8,
    total_size: u32,
    received: u32,
    target_crc: u32,
    /// The absolute offset of the target OTA partition on flash
    target_partition_offset: u32,
    /// Size of the target partition
    target_partition_size: u32,
    /// Running CRC32 calculation
    running_crc: u32,
}

impl OtaManager {
    /// Create a new OTA manager
    pub fn new() -> Self {
        Self {
            state: OtaState::Idle,
            last_progress_log: 0,
            total_size: 0,
            received: 0,
            target_crc: 0,
            target_partition_offset: 0,
            target_partition_size: 0,
            running_crc: 0,
        }
    }

    /// Get current OTA state
    #[allow(dead_code)]
    pub fn state(&self) -> OtaState {
        self.state
    }

    /// Check if OTA is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(self.state, OtaState::Receiving { .. })
    }

    /// Begin OTA update
    ///
    /// # Arguments
    /// * `total_size` - Total firmware size in bytes
    /// * `target_crc` - Expected CRC32 of the firmware
    pub async fn begin(&mut self, total_size: u32, target_crc: u32) -> Result<(), OtaError> {
        if self.is_in_progress() {
            return Err(OtaError::AlreadyInProgress);
        }

        if total_size == 0 || total_size > 0x100000 {
            // Max OTA partition size is 1MB (0x100000)
            error!("Invalid firmware size: {}", total_size);
            return Err(OtaError::InvalidSize);
        }

        info!(
            "Starting OTA: size={}, crc=0x{:08x}",
            total_size, target_crc
        );

        // Set the global flag BEFORE initializing OTA
        // This pauses HID report sending
        OTA_IN_PROGRESS.store(true, Ordering::SeqCst);

        // Find the target partition using the shared flash storage
        let result = with_flash_storage(|flash| {
            // Create OTA updater - buffer must be exactly PARTITION_TABLE_MAX_LEN (0xC00 = 3072 bytes)
            let mut buffer = [0u8; PARTITION_TABLE_MAX_LEN];
            let mut ota = match OtaUpdater::new(flash, &mut buffer) {
                Ok(ota) => ota,
                Err(e) => {
                    error!("Failed to create OTA updater: {:?}", e);
                    return Err(OtaError::InitFailed);
                }
            };

            // Get the next OTA partition (the one we'll write to)
            // This returns a FlashRegion which wraps the partition
            let (next_partition, part_type) = match ota.next_partition() {
                Ok(p) => p,
                Err(e) => {
                    error!("No next OTA partition found: {:?}", e);
                    return Err(OtaError::PartitionNotFound);
                }
            };

            // Get partition size from the FlashRegion
            let size = next_partition.partition_size() as u32;

            // To get the offset, we need to find the partition in the partition table again
            // Read partition table
            let mut pt_buffer = [0u8; PARTITION_TABLE_MAX_LEN];
            let pt = match esp_bootloader_esp_idf::partitions::read_partition_table(
                flash,
                &mut pt_buffer,
            ) {
                Ok(pt) => pt,
                Err(e) => {
                    error!("Failed to read partition table: {:?}", e);
                    return Err(OtaError::InitFailed);
                }
            };

            // Find the partition with matching type
            let offset = pt
                .find_partition(esp_bootloader_esp_idf::partitions::PartitionType::App(
                    part_type,
                ))
                .map_err(|e| {
                    error!("Failed to find partition: {:?}", e);
                    OtaError::PartitionNotFound
                })?
                .ok_or_else(|| {
                    error!("Partition not found in table");
                    OtaError::PartitionNotFound
                })?
                .offset();

            info!(
                "Target OTA partition: offset=0x{:x}, size=0x{:x}",
                offset, size
            );

            // Verify the firmware will fit
            if total_size > size {
                error!("Firmware too large: {} > {}", total_size, size);
                return Err(OtaError::InvalidSize);
            }

            Ok((offset, size))
        })
        .await;

        match result {
            Ok((offset, size)) => {
                self.total_size = total_size;
                self.received = 0;
                self.target_crc = target_crc;
                self.target_partition_offset = offset;
                self.target_partition_size = size;
                self.running_crc = 0xFFFFFFFF; // CRC32 initial value
                self.state = OtaState::Receiving {
                    total_size,
                    received: 0,
                    target_crc,
                };
                self.last_progress_log = 0;
                info!("OTA initialized, ready to receive data");
                Ok(())
            }
            Err(e) => {
                OTA_IN_PROGRESS.store(false, Ordering::SeqCst);
                self.state = OtaState::Error(e);
                Err(e)
            }
        }
    }

    /// Write a chunk of firmware data
    ///
    /// # Arguments
    /// * `data` - Firmware data chunk (should be 4096 bytes except for the last chunk)
    ///
    /// # Returns
    /// * `Ok(true)` - All data received, ready to flush
    /// * `Ok(false)` - More data expected
    /// * `Err(OtaError)` - Write failed
    pub async fn write_chunk(&mut self, data: &[u8]) -> Result<bool, OtaError> {
        if !matches!(self.state, OtaState::Receiving { .. }) {
            return Err(OtaError::NotStarted);
        }

        let write_offset = self.target_partition_offset + self.received;

        // Update running CRC
        for &byte in data {
            self.running_crc = crc32_update(self.running_crc, byte);
        }

        // Write the chunk using shared flash storage (this is a blocking operation)
        let result = with_flash_storage(|flash| match flash.write(write_offset, data) {
            Ok(()) => Ok(()),
            Err(e) => {
                error!("Failed to write OTA chunk at 0x{:x}: {:?}", write_offset, e);
                Err(OtaError::WriteFailed)
            }
        })
        .await;

        match result {
            Ok(()) => {
                self.received += data.len() as u32;
                self.state = OtaState::Receiving {
                    total_size: self.total_size,
                    received: self.received,
                    target_crc: self.target_crc,
                };

                // Log progress every 10%
                let progress = ((self.received as u64 * 100) / self.total_size as u64) as u8;
                let progress_decile = progress / 10;
                if progress_decile > self.last_progress_log {
                    self.last_progress_log = progress_decile;
                    info!(
                        "OTA progress: {}% ({}/{} bytes)",
                        progress, self.received, self.total_size
                    );
                }

                // Check if all data received
                let complete = self.received >= self.total_size;
                Ok(complete)
            }
            Err(e) => {
                self.abort();
                Err(e)
            }
        }
    }

    /// Finalize the OTA update
    ///
    /// # Arguments
    /// * `verify_crc` - Whether to verify CRC32 of written data
    /// * `_enable_rollback` - Reserved for future use
    pub async fn flush(
        &mut self,
        verify_crc: bool,
        _enable_rollback: bool,
    ) -> Result<(), OtaError> {
        if !matches!(self.state, OtaState::Receiving { .. }) {
            return Err(OtaError::NotStarted);
        }

        info!("Finalizing OTA (verify_crc={})", verify_crc);

        // Finalize CRC calculation
        let final_crc = !self.running_crc;

        if verify_crc && final_crc != self.target_crc {
            error!(
                "CRC mismatch: calculated=0x{:08x}, expected=0x{:08x}",
                final_crc, self.target_crc
            );
            self.abort();
            return Err(OtaError::CrcMismatch);
        }

        info!("CRC verified: 0x{:08x}", final_crc);

        // Set the next boot partition
        let result = with_flash_storage(|flash| {
            // Create OTA updater - buffer must be exactly PARTITION_TABLE_MAX_LEN (0xC00 = 3072 bytes)
            let mut buffer = [0u8; PARTITION_TABLE_MAX_LEN];
            let mut ota = match OtaUpdater::new(flash, &mut buffer) {
                Ok(ota) => ota,
                Err(e) => {
                    error!("Failed to create OTA updater: {:?}", e);
                    return Err(OtaError::FlushFailed);
                }
            };

            // Activate the next partition
            if let Err(e) = ota.activate_next_partition() {
                error!("Failed to activate next partition: {:?}", e);
                return Err(OtaError::FlushFailed);
            }

            // Set the image state to New so bootloader will try it
            if let Err(e) = ota.set_current_ota_state(OtaImageState::New) {
                error!("Failed to set OTA state: {:?}", e);
                return Err(OtaError::FlushFailed);
            }

            Ok(())
        })
        .await;

        match result {
            Ok(()) => {
                info!("OTA completed successfully!");
                self.state = OtaState::Complete;
                // Keep OTA_IN_PROGRESS true until reboot
                Ok(())
            }
            Err(e) => {
                self.abort();
                Err(e)
            }
        }
    }

    /// Abort the current OTA update
    pub fn abort(&mut self) {
        if self.is_in_progress() {
            warn!("Aborting OTA update");
        }
        self.state = OtaState::Idle;
        self.last_progress_log = 0;
        self.total_size = 0;
        self.received = 0;
        self.target_crc = 0;
        self.target_partition_offset = 0;
        self.target_partition_size = 0;
        self.running_crc = 0;
        OTA_IN_PROGRESS.store(false, Ordering::SeqCst);
    }

    /// Get OTA progress as (received, total) bytes
    pub fn progress(&self) -> Option<(u32, u32)> {
        match self.state {
            OtaState::Receiving {
                total_size,
                received,
                ..
            } => Some((received, total_size)),
            _ => None,
        }
    }
}

impl Default for OtaManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Update CRC32 with a single byte (IEEE 802.3 polynomial)
#[inline]
fn crc32_update(crc: u32, byte: u8) -> u32 {
    let mut crc = crc ^ (byte as u32);
    for _ in 0..8 {
        if crc & 1 != 0 {
            crc = (crc >> 1) ^ 0xEDB88320;
        } else {
            crc >>= 1;
        }
    }
    crc
}
