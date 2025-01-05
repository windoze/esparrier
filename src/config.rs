use core::{cmp::min, fmt, str::FromStr};

use embassy_net::{IpEndpoint, Ipv4Address, Ipv4Cidr};
use embedded_storage::ReadStorage;
use esp_storage::FlashStorage;
use heapless::String;
use log::{debug, warn};
use serde::Deserialize;

use crate::constants::*;

#[derive(Clone, Deserialize)]
pub struct Secret<const N: usize>(pub String<N>);

impl<const N: usize> fmt::Debug for Secret<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Redacted>")
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
        Ok(Self(String::from_str(s)?))
    }
}

impl<const N: usize> AsRef<str> for Secret<N> {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    // These fields must be set
    pub ssid: Secret<32>,
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

    // Network configuration

    // Static IP configuration, CIDR notation, optional
    #[serde(default)]
    ip_addr: Option<String<20>>,
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

    // Misc internal fields
    #[serde(default = "get_default_watchdog_timeout")]
    pub watchdog_timeout: u32,
}

// Kinda stupid
fn get_default_screen_width() -> u16 {
    SCREEN_WIDTH
}

fn get_default_screen_height() -> u16 {
    SCREEN_HEIGHT
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

fn get_default_watchdog_timeout() -> u32 {
    WATCHDOG_TIMEOUT
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ssid: Secret::from_str(WIFI_SSID).unwrap(),
            password: Secret::from_str(WIFI_PASSWORD).unwrap(),
            server: String::from_str(BARRIER_SERVER).unwrap(),
            screen_name: String::from_str(SCREEN_NAME).unwrap(),
            screen_width: SCREEN_WIDTH,
            screen_height: SCREEN_HEIGHT,
            flip_wheel: REVERSED_WHEEL,
            ip_addr: None,
            gateway: None,
            vid: USB_VID,
            pid: USB_PID,
            manufacturer: String::from_str(USB_MANUFACTURER).unwrap(),
            product: String::from_str(USB_PRODUCT).unwrap(),
            serial_number: String::from_str(USB_SERIAL_NUMBER).unwrap(),
            watchdog_timeout: WATCHDOG_TIMEOUT,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        // TODO: Use proper way to read the config
        let mut bytes = [0u8; 4096];
        let mut flash = FlashStorage::new();
        // Default NVS partition address
        // @see partition_single_app.csv
        let flash_addr = 0x9000;
        flash.read(flash_addr, &mut bytes).unwrap();
        // Find the valid JSON range
        let bytes = json_range(&bytes);
        serde_json_core::from_slice(bytes)
            .map(|(c, _)| c)
            .inspect_err(|e| {
                warn!("Failed to load config, using default, error: {:?}", e);
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

    pub fn get_ip_addr(&self) -> Option<Ipv4Cidr> {
        self.ip_addr.as_ref().map(|s| parse_cidr(s))
    }

    pub fn get_gateway(&self) -> Option<Ipv4Address> {
        self.gateway.as_ref().map(|s| parse_addr(s))
    }
}

fn json_range(buf: &[u8]) -> &[u8] {
    // HACK: Naive JSON range finder, looking for the first '{' and '}' pair
    // It only works in this case, where the JSON doesn't contain any '{' or '}' in the string,
    // and the JSON doesn't contain any nested object.
    let start = buf.iter().position(|&b| b == b'{').unwrap_or_default();
    let end = buf[start..]
        .iter()
        .position(|&b| b == b'}')
        .unwrap_or_default();
    if end > 0 {
        &buf[start..start + end + 1]
    } else {
        &buf[start..]
    }
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
