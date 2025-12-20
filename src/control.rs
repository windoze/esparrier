use embassy_time::{Duration, TimeoutError, with_timeout};
use embassy_usb_driver::{Endpoint, EndpointError, EndpointIn, EndpointOut};
use esp_hal::otg_fs::asynch::Driver;
use log::{info, warn};

use crate::{
    ConfigStore, RunningState, config::ConfigStoreError, get_running_state,
    running_state::get_running_state_mut,
};

#[cfg(feature = "ota")]
use crate::ota::{OtaError, OtaManager};

type EpOut = <Driver<'static> as embassy_usb_driver::Driver<'static>>::EndpointOut;
type EpIn = <Driver<'static> as embassy_usb_driver::Driver<'static>>::EndpointIn;

#[derive(Debug)]
enum Error {
    Endpoint,
    Timeout,
    InvalidConfig,
    UnknownCommand,
    #[cfg(feature = "ota")]
    Ota(OtaError),
}

impl From<EndpointError> for Error {
    fn from(_: EndpointError) -> Self {
        Self::Endpoint
    }
}

impl From<TimeoutError> for Error {
    fn from(_: TimeoutError) -> Self {
        Self::Timeout
    }
}

impl From<ConfigStoreError> for Error {
    fn from(_: ConfigStoreError) -> Self {
        Self::InvalidConfig
    }
}

#[cfg(feature = "ota")]
impl From<OtaError> for Error {
    fn from(e: OtaError) -> Self {
        Self::Ota(e)
    }
}

impl From<Error> for ControlCommandResponse {
    fn from(e: Error) -> Self {
        Self::Error(e)
    }
}

#[derive(Debug, Clone, Copy)]
enum ControlCommand {
    GetState,
    ReadConfig,
    WriteConfig(u8),
    CommitConfig,
    KeepAwake(bool),
    Reboot,
    /// Start OTA update with total size (4 bytes LE) and CRC32 (4 bytes LE)
    #[cfg(feature = "ota")]
    OtaStart {
        size: u32,
        crc: u32,
    },
    /// OTA data chunk - number of 64-byte USB packets and actual data length
    /// Format: 'D' + packets (1 byte) + length (2 bytes LE)
    #[cfg(feature = "ota")]
    OtaData {
        packets: u8,
        length: u16,
    },
    /// Abort OTA update
    #[cfg(feature = "ota")]
    OtaAbort,
    /// Query OTA progress
    #[cfg(feature = "ota")]
    OtaStatus,
}

impl ControlCommand {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes[0] {
            b's' => Some(Self::GetState),
            b'r' => Some(Self::ReadConfig),
            b'w' => Some(Self::WriteConfig(bytes[1])),
            b'c' => Some(Self::CommitConfig),
            b'k' => Some(Self::KeepAwake(bytes[1] != 0)),
            b'b' => Some(Self::Reboot),
            #[cfg(feature = "ota")]
            b'O' if bytes.len() >= 9 => {
                // OtaStart: 'O' + size (4 bytes LE) + crc (4 bytes LE)
                let size = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                let crc = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
                Some(Self::OtaStart { size, crc })
            }
            #[cfg(feature = "ota")]
            b'D' if bytes.len() >= 4 => {
                // OtaData: 'D' + packets (1 byte) + length (2 bytes LE)
                let packets = bytes[1];
                let length = u16::from_le_bytes([bytes[2], bytes[3]]);
                Some(Self::OtaData { packets, length })
            }
            #[cfg(feature = "ota")]
            b'A' => Some(Self::OtaAbort),
            #[cfg(feature = "ota")]
            b'P' => Some(Self::OtaStatus),
            _ => None,
        }
    }
}

enum ControlCommandResponse {
    State(RunningState),
    Config(u8),
    Ok,
    Error(Error),
    /// OTA progress: received bytes (4 bytes LE), total bytes (4 bytes LE)
    #[cfg(feature = "ota")]
    OtaProgress {
        received: u32,
        total: u32,
    },
    /// OTA completed successfully
    #[cfg(feature = "ota")]
    OtaComplete,
}

impl ControlCommandResponse {
    fn to_bytes<'a>(&self, bytes: &'a mut [u8]) -> &'a [u8] {
        match &self {
            Self::State(state) => {
                bytes[0] = b's';
                let len = state.to_bytes(&mut bytes[1..]).len();
                &bytes[..len + 1]
            }
            Self::Config(value) => {
                bytes[0] = b'r';
                bytes[1] = *value;
                &bytes[..2]
            }
            Self::Ok => {
                bytes[0] = b'o';
                &bytes[..1]
            }
            Self::Error(e) => {
                bytes[0] = b'e';
                match e {
                    Error::Endpoint => bytes[1] = b'e',
                    Error::Timeout => bytes[1] = b't',
                    Error::InvalidConfig => bytes[1] = b'i',
                    Error::UnknownCommand => bytes[1] = b'u',
                    #[cfg(feature = "ota")]
                    Error::Ota(ota_err) => {
                        bytes[1] = b'O';
                        bytes[2] = match ota_err {
                            OtaError::AlreadyInProgress => b'a',
                            OtaError::NotStarted => b'n',
                            OtaError::InitFailed => b'i',
                            OtaError::WriteFailed => b'w',
                            OtaError::CrcMismatch => b'c',
                            OtaError::FlushFailed => b'f',
                            OtaError::InvalidSize => b's',
                            OtaError::PartitionNotFound => b'p',
                        };
                        return &bytes[..3];
                    }
                }
                &bytes[..2]
            }
            #[cfg(feature = "ota")]
            Self::OtaProgress { received, total } => {
                bytes[0] = b'P';
                bytes[1..5].copy_from_slice(&received.to_le_bytes());
                bytes[5..9].copy_from_slice(&total.to_le_bytes());
                &bytes[..9]
            }
            #[cfg(feature = "ota")]
            Self::OtaComplete => {
                bytes[0] = b'C';
                &bytes[..1]
            }
        }
    }
}

#[embassy_executor::task]
pub async fn control_task(mut read_ep: EpOut, mut write_ep: EpIn) {
    loop {
        read_ep.wait_enabled().await;
        info!("Control interface connected");
        let mut data = [0; 64];
        let mut new_config = None;
        #[cfg(feature = "ota")]
        let mut ota_manager = OtaManager::new();

        while let Ok(n) = read_ep.read(&mut data).await {
            let cmd = ControlCommand::from_bytes(&data[0..n]);
            info!("Got command: {cmd:?}");
            match cmd {
                Some(ControlCommand::GetState) => {
                    write_response(
                        &mut write_ep,
                        ControlCommandResponse::State(get_running_state().await),
                    )
                    .await
                    .ok();
                }
                Some(ControlCommand::KeepAwake(keep_awake)) => {
                    get_running_state_mut().await.keep_awake = keep_awake;
                    write_response(&mut write_ep, ControlCommandResponse::Ok)
                        .await
                        .ok();
                }
                Some(ControlCommand::ReadConfig) => {
                    send_config(&mut write_ep).await.ok();
                }
                Some(ControlCommand::WriteConfig(blocks)) => {
                    new_config = receive_config(&mut read_ep, blocks as usize).await.ok();
                    if new_config
                        .as_ref()
                        .map(|c| c.validate().is_ok())
                        .unwrap_or_default()
                    {
                        write_response(&mut write_ep, ControlCommandResponse::Ok)
                            .await
                            .ok();
                    } else {
                        warn!("Invalid config received");
                        write_response(&mut write_ep, Error::InvalidConfig.into())
                            .await
                            .ok();
                    }
                }
                Some(ControlCommand::CommitConfig) => match &mut new_config {
                    Some(config) => match config.commit().await {
                        Ok(_) => {
                            write_response(&mut write_ep, ControlCommandResponse::Ok)
                                .await
                                .ok();
                            info!("Config committed, resetting in 1 second...");
                            embassy_time::Timer::after(Duration::from_millis(1000)).await;
                            esp_hal::system::software_reset();
                        }
                        Err(e) => {
                            warn!("Error committing config: {e:?}");
                            write_response(&mut write_ep, ControlCommandResponse::Error(e.into()))
                                .await
                                .ok();
                        }
                    },
                    None => {
                        write_response(&mut write_ep, Error::InvalidConfig.into())
                            .await
                            .ok();
                    }
                },
                Some(ControlCommand::Reboot) => {
                    write_response(&mut write_ep, ControlCommandResponse::Ok)
                        .await
                        .ok();
                    // Wait for a short while to send the Ok response back
                    embassy_time::Timer::after(Duration::from_millis(100)).await;
                    info!("Rebooting...");
                    esp_hal::system::software_reset()
                }
                #[cfg(feature = "ota")]
                Some(ControlCommand::OtaStart { size, crc }) => {
                    match ota_manager.begin(size, crc).await {
                        Ok(()) => {
                            write_response(&mut write_ep, ControlCommandResponse::Ok)
                                .await
                                .ok();
                        }
                        Err(e) => {
                            write_response(&mut write_ep, Error::Ota(e).into())
                                .await
                                .ok();
                        }
                    }
                }
                #[cfg(feature = "ota")]
                Some(ControlCommand::OtaData { packets, length }) => {
                    // Receive OTA data: 'packets' number of 64-byte USB packets
                    // Then write 'length' bytes to flash (blocking operation)
                    match receive_ota_chunk(
                        &mut read_ep,
                        &mut write_ep,
                        &mut ota_manager,
                        packets,
                        length,
                    )
                    .await
                    {
                        Ok(complete) => {
                            if complete {
                                // All data received, finalize OTA
                                match ota_manager.flush(true, true).await {
                                    Ok(()) => {
                                        write_response(
                                            &mut write_ep,
                                            ControlCommandResponse::OtaComplete,
                                        )
                                        .await
                                        .ok();
                                        info!("OTA complete, rebooting in 100ms...");
                                        embassy_time::Timer::after(Duration::from_millis(100))
                                            .await;
                                        esp_hal::system::software_reset();
                                    }
                                    Err(e) => {
                                        write_response(&mut write_ep, Error::Ota(e).into())
                                            .await
                                            .ok();
                                    }
                                }
                            } else {
                                // More data expected, send progress
                                if let Some((received, total)) = ota_manager.progress() {
                                    write_response(
                                        &mut write_ep,
                                        ControlCommandResponse::OtaProgress { received, total },
                                    )
                                    .await
                                    .ok();
                                } else {
                                    write_response(&mut write_ep, ControlCommandResponse::Ok)
                                        .await
                                        .ok();
                                }
                            }
                        }
                        Err(e) => {
                            write_response(&mut write_ep, e.into()).await.ok();
                        }
                    }
                }
                #[cfg(feature = "ota")]
                Some(ControlCommand::OtaAbort) => {
                    ota_manager.abort();
                    write_response(&mut write_ep, ControlCommandResponse::Ok)
                        .await
                        .ok();
                }
                #[cfg(feature = "ota")]
                Some(ControlCommand::OtaStatus) => {
                    if let Some((received, total)) = ota_manager.progress() {
                        write_response(
                            &mut write_ep,
                            ControlCommandResponse::OtaProgress { received, total },
                        )
                        .await
                        .ok();
                    } else {
                        // Not in OTA mode, just respond Ok
                        write_response(&mut write_ep, ControlCommandResponse::Ok)
                            .await
                            .ok();
                    }
                }
                None => {
                    write_response(&mut write_ep, Error::UnknownCommand.into())
                        .await
                        .ok();
                }
            }
        }
        info!("Control interface disconnected");
        #[cfg(feature = "ota")]
        {
            // If OTA was in progress when disconnected, abort it
            if ota_manager.is_in_progress() {
                warn!("USB disconnected during OTA, aborting");
                ota_manager.abort();
            }
        }
    }
}

async fn write_response(
    write_ep: &mut EpIn,
    response: ControlCommandResponse,
) -> Result<(), Error> {
    let mut data = [0; 64];
    let response_bytes = response.to_bytes(&mut data);
    with_timeout(Duration::from_millis(500), write_ep.write(response_bytes))
        .await
        .inspect_err(|e| {
            info!("Error sending config: {e:?}");
        })??;
    Ok(())
}

async fn send_config(write_ep: &mut EpIn) -> Result<(), Error> {
    with_timeout(Duration::from_millis(1000), async {
        let mut data = [0; 64];
        let store = ConfigStore::current();
        info!("Config len: {}", store.len());
        let blocks = store.len().div_ceil(64);
        write_response(write_ep, ControlCommandResponse::Config(blocks as u8)).await?;

        for i in 0..blocks {
            data.fill(0);
            store.read_block(i * 64, &mut data);
            write_ep
                .write(&data)
                .await
                .map_err(|_| EndpointError::Disabled)?;
        }
        Result::<(), Error>::Ok(())
    })
    .await?
}

async fn receive_config(read_ep: &mut EpOut, blocks: usize) -> Result<ConfigStore, Error> {
    with_timeout(Duration::from_millis(1000), async {
        let mut store = ConfigStore::new();
        let mut data = [0; 64];
        let mut offset = 0;
        for _ in 0..blocks {
            data.fill(0);
            let block_len = read_ep
                .read(&mut data)
                .await
                .map_err(|_| EndpointError::Disabled)?;
            store.write_block(offset, &data);
            offset += block_len;
        }
        Result::<ConfigStore, Error>::Ok(store)
    })
    .await?
}

/// Receive OTA data chunk from USB and write to flash.
///
/// This function receives `packets` number of 64-byte USB packets (up to 4096 bytes total),
/// then performs a blocking flash write operation. The flash write is blocking but we
/// yield to the async runtime between USB reads and after the flash write.
///
/// # Arguments
/// * `read_ep` - USB bulk OUT endpoint for reading data
/// * `write_ep` - USB bulk IN endpoint for sending responses (unused but needed for error handling)
/// * `ota_manager` - OTA manager instance
/// * `packets` - Number of 64-byte USB packets to receive (max 64 = 4096 bytes)
///
/// # Returns
/// * `Ok(true)` - All firmware data received, OTA should be finalized
/// * `Ok(false)` - Chunk written successfully, more data expected
/// * `Err(Error)` - Failed to receive or write chunk
#[cfg(feature = "ota")]
async fn receive_ota_chunk(
    read_ep: &mut EpOut,
    _write_ep: &mut EpIn,
    ota_manager: &mut OtaManager,
    packets: u8,
    length: u16,
) -> Result<bool, Error> {
    // Buffer for one OTA chunk (up to 4096 bytes)
    // We use a static buffer to avoid stack overflow
    const MAX_CHUNK_SIZE: usize = 4096;
    let mut chunk_buf = [0u8; MAX_CHUNK_SIZE];
    let mut chunk_offset = 0usize;

    let packets = packets as usize;
    let length = length as usize;
    if packets == 0 || packets > 64 || length == 0 || length > MAX_CHUNK_SIZE {
        warn!("Invalid OTA packet count {} or length {}", packets, length);
        return Err(Error::Ota(OtaError::InvalidSize));
    }

    // Receive USB packets into the chunk buffer
    // Use a longer timeout for OTA data (5 seconds per chunk)
    let receive_result = with_timeout(Duration::from_millis(5000), async {
        let mut packet_buf = [0u8; 64];
        for i in 0..packets {
            packet_buf.fill(0);
            let n = read_ep
                .read(&mut packet_buf)
                .await
                .map_err(|_| EndpointError::Disabled)?;

            // Copy received data to chunk buffer
            let copy_len = n.min(MAX_CHUNK_SIZE - chunk_offset);
            chunk_buf[chunk_offset..chunk_offset + copy_len]
                .copy_from_slice(&packet_buf[..copy_len]);
            chunk_offset += copy_len;

            // Yield every 16 packets to let other tasks run
            if (i + 1) % 16 == 0 {
                embassy_futures::yield_now().await;
            }
        }
        Result::<(), Error>::Ok(())
    })
    .await;

    if let Err(e) = receive_result {
        warn!("Timeout receiving OTA data: {:?}", e);
        ota_manager.abort();
        return Err(Error::Timeout);
    }
    if let Err(e) = receive_result.unwrap() {
        warn!("Error receiving OTA data: {:?}", e);
        ota_manager.abort();
        return Err(e);
    }

    // Now write the chunk to flash (this is a BLOCKING operation internally)
    // Use 'length' to exclude padding bytes from the last packet
    // The HID report writer should skip sending reports while OTA_IN_PROGRESS is true
    let write_result = ota_manager.write_chunk(&chunk_buf[..length]).await;

    // Yield after flash write to let other tasks run
    embassy_futures::yield_now().await;

    match write_result {
        Ok(complete) => Ok(complete),
        Err(e) => {
            // OTA manager already aborted on error
            Err(Error::Ota(e))
        }
    }
}
