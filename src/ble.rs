#![cfg(feature = "ble")]

use embassy_executor::Spawner;
use embassy_futures::{
    join::join,
    select::{select, Either},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use esp_radio::{ble::controller::BleConnector, Controller as EspController};
use heapless::{String, Vec};
use log::{info, warn};
use trouble_host::{
    gatt::{GattConnection, GattConnectionEvent},
    prelude::*,
};

use crate::{
    AppConfig, mk_static,
    synergy_hid::{ReportType, SynergyHid},
};

const BLE_CMD_SLOTS: usize = 20;
const BLE_CONNECTIONS_MAX: usize = 1;
const BLE_L2CAP_CHANNELS: usize = 2;
const BLE_REPORT_QUEUE_DEPTH: usize = 32;
const BLE_ATTRIBUTE_TABLE_SIZE: usize = 32;

const HID_INFORMATION: [u8; 4] = [0x11, 0x01, 0x00, 0x02];
const HID_REPORT_REFERENCE_KEYBOARD: [u8; 2] = [ReportType::Keyboard as u8, 0x01];
const HID_REPORT_REFERENCE_MOUSE: [u8; 2] = [ReportType::Mouse as u8, 0x01];
const HID_REPORT_REFERENCE_CONSUMER: [u8; 2] = [ReportType::Consumer as u8, 0x01];
const INPUT_REPORT_MAX: usize = ReportType::get_max_report_size();
const HID_REPORT_MAP_CAP: usize = 256;

const HID_SERVICE_UUID: u16 = 0x1812;
const HID_INFORMATION_UUID: u16 = 0x2A4A;
const HID_CONTROL_POINT_UUID: u16 = 0x2A4C;
const HID_PROTOCOL_MODE_UUID: u16 = 0x2A4E;
const HID_REPORT_MAP_UUID: u16 = 0x2A4B;
const HID_REPORT_UUID: u16 = 0x2A4D;
const HID_REPORT_REFERENCE_UUID: u16 = 0x2908;
const HID_SERVICE_UUID_LE: [u8; 2] = HID_SERVICE_UUID.to_le_bytes();

static BLE_REPORT_CHANNEL: Channel<CriticalSectionRawMutex, BleReport, BLE_REPORT_QUEUE_DEPTH> =
    Channel::new();

#[gatt_server(
    connections_max = BLE_CONNECTIONS_MAX,
    mutex_type = CriticalSectionRawMutex,
    attribute_table_size = BLE_ATTRIBUTE_TABLE_SIZE
)]
struct HidServer {
    hid_service: HidService,
}

#[gatt_service(uuid = HID_SERVICE_UUID)]
struct HidService {
    #[characteristic(uuid = HID_INFORMATION_UUID, read)]
    hid_information: [u8; 4],
    #[characteristic(uuid = HID_CONTROL_POINT_UUID, write)]
    control_point: u8,
    #[characteristic(uuid = HID_PROTOCOL_MODE_UUID, read, write)]
    protocol_mode: u8,
    #[characteristic(uuid = HID_REPORT_MAP_UUID, read)]
    report_map: Vec<u8, HID_REPORT_MAP_CAP>,
    #[descriptor(uuid = HID_REPORT_REFERENCE_UUID, read, value = HID_REPORT_REFERENCE_KEYBOARD)]
    #[characteristic(uuid = HID_REPORT_UUID, read, notify)]
    keyboard_input: [u8; 9],
    #[descriptor(uuid = HID_REPORT_REFERENCE_UUID, read, value = HID_REPORT_REFERENCE_MOUSE)]
    #[characteristic(uuid = HID_REPORT_UUID, read, notify)]
    mouse_input: [u8; 8],
    #[descriptor(uuid = HID_REPORT_REFERENCE_UUID, read, value = HID_REPORT_REFERENCE_CONSUMER)]
    #[characteristic(uuid = HID_REPORT_UUID, read, notify)]
    consumer_input: [u8; 3],
}

const HID_REPORT_MAP: &[u8] = SynergyHid::get_report_descriptor().1;

#[derive(Clone, Copy)]
struct BleReport {
    report_type: ReportType,
    len: usize,
    data: [u8; INPUT_REPORT_MAX],
}

impl BleReport {
    fn new(report_type: ReportType, payload: &[u8]) -> Option<Self> {
        if payload.len() > INPUT_REPORT_MAX {
            return None;
        }
        let mut data = [0u8; INPUT_REPORT_MAX];
        data[..payload.len()].copy_from_slice(payload);
        Some(Self {
            report_type,
            len: payload.len(),
            data,
        })
    }

    fn as_array<const N: usize>(&self) -> Option<[u8; N]> {
        if self.len < N {
            return None;
        }
        let mut buf = [0u8; N];
        buf.copy_from_slice(&self.data[..N]);
        Some(buf)
    }
}

pub fn publish_report(report: (ReportType, &[u8])) {
    if let Some(packet) = BleReport::new(report.0, report.1) {
        let _ = BLE_REPORT_CHANNEL.try_send(packet);
    }
}

pub fn start_ble(
    spawner: Spawner,
    radio: &'static EspController<'static>,
    bt: esp_hal::peripherals::BT<'static>,
) {
    let connector = match BleConnector::new(radio, bt, esp_radio::ble::Config::default()) {
        Ok(connector) => connector,
        Err(err) => {
            warn!("Failed to init BLE connector: {err:?}");
            return;
        }
    };

    let ble_name: &'static str = {
        let name = mk_static!(String<32>, AppConfig::get().ble_device_name.clone());
        name.as_str()
    };

    if let Err(err) = spawner.spawn(ble_task(connector, ble_name)) {
        warn!("Failed to spawn BLE task: {err:?}");
    }
}

#[embassy_executor::task]
async fn ble_task(connector: BleConnector<'static>, name: &'static str) {
    let controller: ExternalController<_, BLE_CMD_SLOTS> = ExternalController::new(connector);
    let resources = mk_static!(
        HostResources<DefaultPacketPool, BLE_CONNECTIONS_MAX, BLE_L2CAP_CHANNELS>,
        HostResources::new()
    );
    let stack = mk_static!(
        Stack<
            'static,
            ExternalController<BleConnector<'static>, BLE_CMD_SLOTS>,
            DefaultPacketPool
        >,
        trouble_host::new(controller, resources)
    );
    let stack: &'static Stack<
        'static,
        ExternalController<BleConnector<'static>, BLE_CMD_SLOTS>,
        DefaultPacketPool,
    > = stack;
    let Host {
        peripheral,
        runner,
        ..
    } = stack.build();

    let server_ref = mk_static!(
        HidServer<'static>,
        HidServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
            name,
            appearance: &appearance::human_interface_device::MOUSE,
        }))
        .unwrap()
    );
    let server: &'static HidServer<'static> = server_ref;
    init_hid_service(server);

    let _ = join(
        ble_runner(runner),
        ble_peripheral_loop(peripheral, server, name),
    )
    .await;
}

async fn ble_runner<C: Controller, P: PacketPool>(mut runner: Runner<'static, C, P>) {
    loop {
        if let Err(err) = runner.run().await {
            warn!("BLE runner stopped: {err:?}");
        }
    }
}

async fn ble_peripheral_loop<C: Controller>(
    mut peripheral: Peripheral<'static, C, DefaultPacketPool>,
    server: &'static HidServer<'static>,
    device_name: &'static str,
) {
    loop {
        match advertise(device_name, &mut peripheral, server).await {
            Ok(conn) => {
                info!("BLE central connected");
                connection_loop(server, conn).await;
                info!("BLE central disconnected");
            }
            Err(err) => warn!("BLE advertise error: {err:?}"),
        }
    }
}

async fn advertise<C: Controller>(
    name: &'static str,
    peripheral: &mut Peripheral<'static, C, DefaultPacketPool>,
    server: &'static HidServer<'static>,
) -> Result<GattConnection<'static, 'static, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut adv_data = [0u8; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[HID_SERVICE_UUID_LE]),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut adv_data[..],
    )?;

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &adv_data[..len],
                scan_data: &[],
            },
        )
        .await?;

    Ok(
        advertiser
            .accept()
            .await?
            .with_attribute_server(server)
            .map_err(BleHostError::BleHost)?,
    )
}

async fn connection_loop(
    server: &'static HidServer<'static>,
    conn: GattConnection<'_, 'static, DefaultPacketPool>,
) {
    loop {
        match select(conn.next(), BLE_REPORT_CHANNEL.receive()).await {
            Either::First(event) => {
                if handle_gatt_event(event).await {
                    break;
                }
            }
            Either::Second(report) => {
                if send_notification(server, &conn, report).await.is_err() {
                    break;
                }
            }
        }
    }
}

async fn handle_gatt_event(
    event: GattConnectionEvent<'_, 'static, DefaultPacketPool>,
) -> bool {
    match event {
        GattConnectionEvent::Disconnected { .. } => true,
        GattConnectionEvent::Gatt { event } => {
            if let Ok(reply) = event.accept() {
                reply.send().await;
            }
            false
        }
        _ => false,
    }
}

fn init_hid_service(server: &'static HidServer<'static>) {
    if let Err(err) = server.set(&server.hid_service.hid_information, &HID_INFORMATION) {
        warn!("Failed to set HID information: {err:?}");
    }
    let boot_protocol: u8 = 1;
    if let Err(err) = server.set(&server.hid_service.protocol_mode, &boot_protocol) {
        warn!("Failed to set HID protocol mode: {err:?}");
    }
    let mut report_map = Vec::<u8, HID_REPORT_MAP_CAP>::new();
    if report_map.extend_from_slice(HID_REPORT_MAP).is_err() {
        warn!("HID report map exceeds storage capacity");
        return;
    }
    if let Err(err) = server.set(&server.hid_service.report_map, &report_map) {
        warn!("Failed to set HID report map: {err:?}");
    }
}

async fn send_notification(
    server: &'static HidServer<'static>,
    conn: &GattConnection<'_, 'static, DefaultPacketPool>,
    report: BleReport,
) -> Result<(), ()> {
    match report.report_type {
        ReportType::Keyboard => {
            let payload = report.as_array::<9>().ok_or(())?;
            server
                .hid_service
                .keyboard_input
                .notify(conn, &payload)
                .await
                .map_err(|_| ())
        }
        ReportType::Mouse => {
            let payload = report.as_array::<8>().ok_or(())?;
            server
                .hid_service
                .mouse_input
                .notify(conn, &payload)
                .await
                .map_err(|_| ())
        }
        ReportType::Consumer => {
            let payload = report.as_array::<3>().ok_or(())?;
            server
                .hid_service
                .consumer_input
                .notify(conn, &payload)
                .await
                .map_err(|_| ())
        }
    }
}
