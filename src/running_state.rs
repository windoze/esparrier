use embassy_net::Ipv4Cidr;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::constants::*;

#[derive(Clone, Debug)]
pub struct RunningState {
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
    pub feature_flags: u8,
    pub ip_address: Option<Ipv4Cidr>,
    pub server_connected: bool,
    pub active: bool,
    pub keep_awake: bool,
    pub model_id: u8,
}

impl RunningState {
    pub const fn new() -> Self {
        Self {
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            version_patch: VERSION_PATCH,
            feature_flags: FEATURE_FLAGS,
            ip_address: None,
            server_connected: false,
            active: false,
            keep_awake: false,
            model_id: MODEL_ID,
        }
    }

    pub fn set_ip_address(&self, ip_address: Option<Ipv4Cidr>) -> Self {
        Self {
            ip_address,
            ..self.clone()
        }
    }

    pub fn to_bytes<'a>(&self, bytes: &'a mut [u8]) -> &'a [u8] {
        bytes[0] = self.version_major;
        bytes[1] = self.version_minor;
        bytes[2] = self.version_patch;
        bytes[3] = self.feature_flags;
        if let Some(ip) = self.ip_address {
            let octets = ip.address().octets();
            bytes[4] = octets[0];
            bytes[5] = octets[1];
            bytes[6] = octets[2];
            bytes[7] = octets[3];
            bytes[8] = ip.prefix_len();
        } else {
            bytes[4..9].fill(0);
        }
        bytes[9] = self.server_connected as u8;
        bytes[10] = self.active as u8;
        bytes[11] = self.keep_awake as u8;
        bytes[12] = self.model_id;

        &bytes[..13]
    }
}

impl Default for RunningState {
    fn default() -> Self {
        Self::new()
    }
}

static RUNNING_STATE: Mutex<CriticalSectionRawMutex, RunningState> =
    Mutex::new(RunningState::new());

pub async fn get_running_state() -> RunningState {
    RUNNING_STATE.lock().await.clone()
}

pub async fn get_running_state_mut() -> embassy_sync::mutex::MutexGuard<
    'static,
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    RunningState,
> {
    RUNNING_STATE.lock().await
}
