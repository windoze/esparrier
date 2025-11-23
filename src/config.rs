use core::{
    cmp::{max, min},
    str::FromStr,
};

use embassy_net::{Config, IpEndpoint, Ipv4Address, Ipv4Cidr, StaticConfigV4};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
};
use embedded_storage::{ReadStorage, Storage};
use esp_storage::FlashStorage;
use heapless::{String, Vec};
use log::{debug, warn};
use serde::{Deserialize, Serialize};

use crate::constants::*;

// Flash has a sector size of 4KB
const MAX_CONFIG_SIZE: usize = 4096;
// Default NVS partition address, must be the same as the one in the partition table
// @see partition_single_app.csv
const NVS_PARTITION_ADDRESS: u32 = 0x9000;

#[derive(Clone, Deserialize, Serialize)]
pub struct Secret<const N: usize>(pub String<N>);

impl<const N: usize> core::fmt::Debug for Secret<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<Redacted>")
    }
}

impl<const N: usize> Default for Secret<N> {
    fn default() -> Self {
        Self(String::default())
    }
}

impl<const N: usize> From<Secret<N>> for String<N> {
    fn from(s: Secret<N>) -> Self {
        s.0
    }
}

impl<const N: usize> FromStr for Secret<N> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(String::from_str(s).map_err(|_| ())?))
    }
}

impl<const N: usize> AsRef<str> for Secret<N> {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    // These fields must be set
    pub ssid: String<32>,
    pub password: Secret<64>,
    pub server: String<64>,
    pub screen_name: String<64>,

    // Screen configuration
    #[serde(default = "get_default_screen_width")]
    pub screen_width: u16,
    #[serde(default = "get_default_screen_height")]
    pub screen_height: u16,
    #[serde(default)]
    pub flip_wheel: bool,
    #[serde(default = "get_default_polling_rate")]
    pub polling_rate: u16,
    #[serde(default = "get_default_jiggle_interval")]
    pub jiggle_interval: u16,

    // Indicator brightness, used by both SmartLED and graphical indicators
    #[serde(default = "get_default_brightness")]
    pub brightness: u8,

    // Network configuration

    // Static IP configuration, CIDR notation, optional
    #[serde(default)]
    ip_addr: Option<String<20>>,
    // DNS server address, optional
    #[serde(default)]
    dns_server: Vec<String<16>, 3>,
    // Gateway IP address, optional
    #[serde(default)]
    gateway: Option<String<16>>,

    // USB HID configuration
    #[serde(default = "get_default_vid")]
    pub vid: u16,
    #[serde(default = "get_default_pid")]
    pub pid: u16,
    #[serde(default = "get_default_manufacturer")]
    pub manufacturer: String<64>,
    #[serde(default = "get_default_product")]
    pub product: String<64>,
    #[serde(default = "get_default_serial_number")]
    pub serial_number: String<64>,
    #[serde(default = "get_default_ble_device_name")]
    pub ble_device_name: String<32>,
}

// Kinda stupid
fn get_default_screen_width() -> u16 {
    SCREEN_WIDTH
}

fn get_default_screen_height() -> u16 {
    SCREEN_HEIGHT
}

fn get_default_polling_rate() -> u16 {
    POLLING_RATE
}

fn get_default_jiggle_interval() -> u16 {
    JIGGLE_INTERVAL
}

fn get_default_brightness() -> u8 {
    BRIGHTNESS
}

fn get_default_vid() -> u16 {
    USB_VID
}

fn get_default_pid() -> u16 {
    USB_PID
}

fn get_default_manufacturer() -> String<64> {
    String::from_str(USB_MANUFACTURER).unwrap()
}

fn get_default_product() -> String<64> {
    String::from_str(USB_PRODUCT).unwrap()
}

fn get_default_serial_number() -> String<64> {
    String::from_str(USB_SERIAL_NUMBER).unwrap()
}

fn get_default_ble_device_name() -> String<32> {
    String::from_str(USB_PRODUCT).unwrap()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ssid: String::from_str(WIFI_SSID).unwrap(),
            password: Secret::from_str(WIFI_PASSWORD).unwrap(),
            server: String::from_str(BARRIER_SERVER).unwrap(),
            screen_name: String::from_str(SCREEN_NAME).unwrap(),
            screen_width: SCREEN_WIDTH,
            screen_height: SCREEN_HEIGHT,
            polling_rate: POLLING_RATE,
            jiggle_interval: JIGGLE_INTERVAL,
            flip_wheel: REVERSED_WHEEL,
            brightness: BRIGHTNESS,
            ip_addr: None,
            dns_server: Vec::new(),
            gateway: None,
            vid: USB_VID,
            pid: USB_PID,
            manufacturer: String::from_str(USB_MANUFACTURER).unwrap(),
            product: String::from_str(USB_PRODUCT).unwrap(),
            serial_number: String::from_str(USB_SERIAL_NUMBER).unwrap(),
            ble_device_name: String::from_str(USB_PRODUCT).unwrap(),
        }
    }
}

static CONFIG: OnceLock<AppConfig> = OnceLock::new();
static FLASH_STORAGE: OnceLock<RwLock<CriticalSectionRawMutex, FlashStorage>> = OnceLock::new();

impl AppConfig {
    pub async fn init(flash: esp_hal::peripherals::FLASH<'static>) {
        FLASH_STORAGE.get_or_init(|| RwLock::new(FlashStorage::new(flash)));
        let config = Self::load().await;
        CONFIG.init(config).expect("Config already initialized");
        debug!("Config initialized: {:?}", Self::get());
    }

    pub fn get() -> &'static Self {
        // CONFIG.get_or_init(Self::load)
        CONFIG.try_get().expect("Config not initialized")
    }

    async fn load() -> Self {
        // TODO: Use proper way to read the config
        let mut bytes = [0u8; MAX_CONFIG_SIZE];
        let mut flash = FLASH_STORAGE.get().await.write().await;
        // Default NVS partition address
        // @see partition_single_app.csv
        let flash_addr = NVS_PARTITION_ADDRESS;
        flash.read(flash_addr, &mut bytes).unwrap();
        // Find the valid JSON range
        let bytes = json_range(&bytes);
        serde_json_core::from_slice(bytes)
            .map(|(c, _)| c)
            .inspect_err(|e| {
                warn!("Failed to load config, using default, error: {e:?}");
                debug!(
                    "Config content (first 16 bytes): {:?}",
                    &bytes[0..min(16, bytes.len())]
                );
            })
            .unwrap_or_default()
    }

    pub fn get_server_endpoint(&self) -> IpEndpoint {
        parse_endpoint(&self.server)
    }

    pub fn get_ip_config(&self) -> Config {
        match self.ip_addr.as_ref().map(|s| parse_cidr(s)) {
            Some(addr) => Config::ipv4_static(StaticConfigV4 {
                address: addr,
                dns_servers: self.dns_server.iter().map(|s| parse_addr(s)).collect(),
                gateway: self.gateway.as_ref().map(|s| parse_addr(s)), // Gateway is optional if server is on the same subnet
            }),
            None => Config::dhcpv4(Default::default()),
        }
    }

    pub fn get_polling_interval(&self) -> u8 {
        let polling_interval = 1000 / self.polling_rate;
        if polling_interval < 1 {
            1
        } else if polling_interval > 255 {
            255
        } else {
            polling_interval as u8
        }
    }
}

fn json_range(buf: &[u8]) -> &[u8] {
    let end = buf
        .iter()
        .position(|&c| c == 0 || c > 0xF4)
        .unwrap_or(buf.len());
    &buf[..end]
}

fn parse_addr(s: &str) -> Ipv4Address {
    let mut parts = s.split('.');
    let a = parts.next().expect("invalid ip address");
    let b = parts.next().expect("invalid ip address");
    let c = parts.next().expect("invalid ip address");
    let d = parts.next().expect("invalid ip address");
    let a = a.parse().expect("invalid ip address");
    let b = b.parse().expect("invalid ip address");
    let c = c.parse().expect("invalid ip address");
    let d = d.parse().expect("invalid ip address");
    Ipv4Address::new(a, b, c, d)
}

fn parse_cidr(s: &str) -> Ipv4Cidr {
    let mut parts = s.split('/');
    let ip = parts.next().expect("invalid cidr address");
    let port = parts.next().expect("invalid cidr address");
    let ip = parse_addr(ip);
    let prefix_len = port.parse().expect("invalid prefix length");
    Ipv4Cidr::new(ip, prefix_len)
}

fn parse_endpoint<Ep: AsRef<str>>(s: Ep) -> IpEndpoint {
    let s = s.as_ref();
    let mut parts = s.split(':');
    let ip = parts.next().expect("invalid ip endpoint");
    let port = parts.next().expect("invalid ip endpoint");
    let ip = parse_addr(ip);
    let port = port.parse().expect("invalid port");
    IpEndpoint::from((ip, port))
}

pub struct ConfigStore {
    pub data: [u8; MAX_CONFIG_SIZE],
    pub size: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum ConfigStoreError {
    FlashStorageError,
    RangeTooLarge,
    SerdeError,
    UnknownCommand,
}

impl From<serde_json_core::de::Error> for ConfigStoreError {
    fn from(_: serde_json_core::de::Error) -> Self {
        Self::SerdeError
    }
}

impl From<serde_json_core::ser::Error> for ConfigStoreError {
    fn from(_: serde_json_core::ser::Error) -> Self {
        Self::SerdeError
    }
}

impl From<esp_storage::FlashStorageError> for ConfigStoreError {
    fn from(_: esp_storage::FlashStorageError) -> Self {
        Self::FlashStorageError
    }
}

impl ConfigStore {
    pub fn new() -> Self {
        Self {
            data: [0; MAX_CONFIG_SIZE],
            size: 0,
        }
    }

    pub fn current() -> Self {
        Self::from(AppConfig::get())
    }

    pub fn len(&self) -> usize {
        self.size
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn write_block(&mut self, offset: usize, buf: &[u8]) {
        let len = buf.len();
        self.data[offset..offset + len].copy_from_slice(buf);
        self.size = max(self.size, offset + len);
    }

    pub fn read_block<'a>(&self, offset: usize, buf: &'a mut [u8]) -> &'a [u8] {
        let end = min(offset + buf.len(), self.size);
        buf[0..(end - offset)].copy_from_slice(&self.data[offset..end]);
        &buf[0..(end - offset)]
    }

    pub fn validate(&self) -> Result<(), ConfigStoreError> {
        serde_json_core::from_slice::<AppConfig>(json_range(&self.data))?;
        Ok(())
    }

    pub async fn commit(&mut self) -> Result<(), ConfigStoreError> {
        self.size = json_range(&self.data).len();
        // Fill the rest with 0
        for i in self.size..self.data.len() {
            self.data[i] = 0;
        }
        // Write to flash
        warn!("Writing config to flash...");
        let mut flash = FLASH_STORAGE.get().await.write().await;
        // Default NVS partition address
        // @see partition_single_app.csv
        let flash_addr = NVS_PARTITION_ADDRESS;
        flash.write(flash_addr, &self.data)?;
        warn!("Config written to flash");
        Ok(())
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&AppConfig> for ConfigStore {
    fn from(config: &AppConfig) -> Self {
        let mut ret = Self {
            data: [0; MAX_CONFIG_SIZE],
            size: 0,
        };
        let mut config = config.clone();
        config.password = Default::default();
        serde_json_core::to_slice(&config, &mut ret.data).unwrap();
        ret.size = json_range(&ret.data).len();
        ret
    }
}
