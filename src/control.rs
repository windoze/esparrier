use embassy_time::{with_timeout, Duration, TimeoutError};
use embassy_usb_driver::{Endpoint, EndpointError, EndpointIn, EndpointOut};
use esp_hal::otg_fs::asynch::Driver;
use log::{info, warn};

use crate::{
    config::ConfigStoreError, get_running_state, running_state::get_running_state_mut, ConfigStore,
    RunningState,
};

type EpOut = <Driver<'static> as embassy_usb_driver::Driver<'static>>::EndpointOut;
type EpIn = <Driver<'static> as embassy_usb_driver::Driver<'static>>::EndpointIn;

enum Error {
    Endpoint,
    Timeout,
    InvalidConfig,
    UnknownCommand,
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
            _ => None,
        }
    }
}

enum ControlCommandResponse {
    State(RunningState),
    Config(u8),
    Ok,
    Error(Error),
}

impl ControlCommandResponse {
    fn to_bytes<'a>(&self, bytes: &'a mut [u8]) -> &'a [u8] {
        match &self {
            Self::State(state) => {
                bytes[0] = b's';
                bytes[1] = state.version_major;
                bytes[2] = state.version_minor;
                bytes[3] = state.version_patch;
                bytes[4] = state.feature_flags;
                let octets = state
                    .ip_address
                    .map_or([0, 0, 0, 0], |ip| ip.address().octets());
                bytes[5] = octets[0];
                bytes[6] = octets[1];
                bytes[7] = octets[2];
                bytes[8] = octets[3];
                bytes[9] = state.ip_address.map_or(0, |ip| ip.prefix_len());
                bytes[10] = state.server_connected as u8;
                bytes[11] = state.active as u8;
                bytes[12] = state.keep_awake as u8;
                &bytes[..13]
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
                }
                &bytes[..2]
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
        while let Ok(n) = read_ep.read(&mut data).await {
            let cmd = ControlCommand::from_bytes(&data[0..n]);
            info!("Got command: {:?}", cmd);
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
                    Some(config) => match config.commit() {
                        Ok(_) => {
                            write_response(&mut write_ep, ControlCommandResponse::Ok)
                                .await
                                .ok();
                            info!("Config committed, resetting in 1 second...");
                            embassy_time::Timer::after(Duration::from_millis(1000)).await;
                            esp_hal::system::software_reset();
                        }
                        Err(e) => {
                            warn!("Error committing config: {:?}", e);
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
                None => {
                    write_response(&mut write_ep, Error::UnknownCommand.into())
                        .await
                        .ok();
                }
            }
        }
        info!("Control interface disconnected");
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
            info!("Error sending config: {:?}", e);
        })??;
    Ok(())
}

async fn send_config(write_ep: &mut EpIn) -> Result<(), Error> {
    let mut data = [0; 64];
    let store = ConfigStore::current();
    info!("Config len: {}", store.len());
    let blocks = store.len().div_ceil(64);
    write_response(write_ep, ControlCommandResponse::Config(blocks as u8)).await?;

    for i in 0..blocks {
        data.fill(0);
        store.read_block(i * 64, &mut data);
        with_timeout(Duration::from_millis(500), write_ep.write(&data))
            .await
            .map_err(|_| EndpointError::Disabled)??;
    }
    Ok(())
}

async fn receive_config(read_ep: &mut EpOut, blocks: usize) -> Result<ConfigStore, Error> {
    let mut store = ConfigStore::new();
    let mut data = [0; 64];
    let mut offset = 0;
    for _ in 0..blocks {
        data.fill(0);
        let block_len = with_timeout(Duration::from_millis(500), read_ep.read(&mut data))
            .await
            .map_err(|_| EndpointError::Disabled)??;
        store.write_block(offset, &data);
        offset += block_len;
    }
    Ok(store)
}
