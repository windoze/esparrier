use embassy_usb_driver::{Endpoint, EndpointError, EndpointIn, EndpointOut};
use esp_hal::otg_fs::asynch::Driver;
use log::info;

use crate::{get_running_state, ConfigStore, RunningState};

type EpOut = <Driver<'static> as embassy_usb_driver::Driver<'static>>::EndpointOut;
type EpIn = <Driver<'static> as embassy_usb_driver::Driver<'static>>::EndpointIn;

#[derive(Debug, Clone, Copy)]
enum ControlCommand {
    GetState,
    ReadConfig,
    WriteConfig(u16),
    CommitConfig,
}

impl ControlCommand {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes[0] {
            b's' => Some(Self::GetState),
            b'r' => Some(Self::ReadConfig),
            b'w' => Some(Self::WriteConfig(u16::from_le_bytes([bytes[1], bytes[2]]))),
            b'c' => Some(Self::CommitConfig),
            _ => None,
        }
    }
}

enum ControlCommandResponse {
    State(RunningState),
    Config(u8),
    Ok,
    Error,
}

impl ControlCommandResponse {
    fn to_bytes<'a>(&self, bytes: &'a mut [u8]) -> &'a [u8] {
        match &self {
            Self::State(state) => {
                bytes[0] = b's';
                bytes[1] = state.version_major;
                bytes[2] = state.version_minor;
                bytes[3] = state.version_patch;
                let octets = state.ip_address.map_or([0, 0, 0, 0], |ip| ip.address().0);
                bytes[4] = octets[0];
                bytes[5] = octets[1];
                bytes[6] = octets[2];
                bytes[7] = octets[3];
                bytes[8] = state.ip_address.map_or(0, |ip| ip.prefix_len());
                bytes[9] = state.server_connected as u8;
                bytes[10] = state.active as u8;
                &bytes[..11]
            }
            Self::Config(value) => {
                bytes[0] = b'r';
                bytes[1] = *value;
                &bytes[..3]
            }
            Self::Ok => {
                bytes[0] = b'o';
                &bytes[..1]
            }
            Self::Error => {
                bytes[0] = b'e';
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
        loop {
            let mut data = [0; 64];
            match read_ep.read(&mut data).await {
                Ok(n) => {
                    info!("Got bulk: {:?}", &data[..n]);
                    match ControlCommand::from_bytes(&data) {
                        Some(ControlCommand::GetState) => {
                            let response = ControlCommandResponse::State(get_running_state().await);
                            let response_bytes = response.to_bytes(&mut data);
                            write_ep
                                .write(response_bytes)
                                .await
                                .inspect_err(|e| {
                                    info!("Error writing: {:?}", e);
                                })
                                .ok();
                        }
                        Some(ControlCommand::ReadConfig) => {
                            send_config(&mut write_ep).await.ok();
                        }
                        Some(ControlCommand::WriteConfig(_value)) => {
                            // TODO: Receive the config stream
                            let response = ControlCommandResponse::Ok;
                            let response_bytes = response.to_bytes(&mut data);
                            write_ep
                                .write(response_bytes)
                                .await
                                .inspect_err(|e| {
                                    info!("Error writing: {:?}", e);
                                })
                                .ok();
                        }
                        Some(ControlCommand::CommitConfig) => {
                            // TODO: Save config and reset
                            let response = ControlCommandResponse::Ok;
                            let response_bytes = response.to_bytes(&mut data);
                            write_ep
                                .write(response_bytes)
                                .await
                                .inspect_err(|e| {
                                    info!("Error writing: {:?}", e);
                                })
                                .ok();
                        }
                        None => {
                            let response = ControlCommandResponse::Error;
                            let response_bytes = response.to_bytes(&mut data);
                            write_ep
                                .write(response_bytes)
                                .await
                                .inspect_err(|e| {
                                    info!("Error writing: {:?}", e);
                                })
                                .ok();
                        }
                    }
                }
                Err(_) => break,
            }
        }
        info!("Control interface disconnected");
    }
}

async fn send_config(write_ep: &mut EpIn) -> Result<(), EndpointError> {
    let mut data = [0; 64];
    let store = ConfigStore::current();
    info!("Config len: {}", store.len());
    let blocks = (store.len() + 63) / 64;
    let response = ControlCommandResponse::Config(blocks as u8);
    write_ep
        .write(response.to_bytes(&mut data))
        .await
        .inspect_err(|e| {
            info!("Error writing: {:?}", e);
        })?;
    for i in 0..blocks {
        data.fill(0);
        store.read_block(i * 64, &mut data);
        write_ep.write(&data).await.inspect_err(|e| {
            info!("Error writing: {:?}", e);
        })?;
    }
    Ok(())
}
