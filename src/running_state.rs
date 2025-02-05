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
        }
    }

    pub fn set_ip_address(&self, ip_address: Option<Ipv4Cidr>) -> Self {
        Self {
            ip_address,
            ..self.clone()
        }
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
