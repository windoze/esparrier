use embassy_net::{tcp::TcpSocket, IpEndpoint, Ipv4Address, Stack};
use embedded_io_async::Write;
use esp_hal::{peripheral::Peripheral, timer::timg::Wdt};
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use log::{debug, error, info, warn};

use super::{
    packet::Packet, packet_io::PacketReader, packet_io::PacketWriter, packet_stream::PacketStream,
    Actuator, BarrierError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardStage {
    None,
    Mark1,
    Mark2(usize),
    Mark3,
}

#[allow(unused_assignments)]
pub async fn start<A: Actuator, Ep: AsRef<str>>(
    endpoint: Ep,
    device_name: heapless::String<64>,
    stack: &'static Stack<WifiDevice<'_, WifiStaDevice>>,
    actor: &mut A,
    watchdog: &mut Wdt<<esp_hal::peripherals::TIMG1 as Peripheral>::P>,
) -> Result<(), BarrierError> {
    let screen_size: (u16, u16) = actor.get_screen_size().await?;

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    debug!("Connecting to {}", endpoint.as_ref());
    let mut stream = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    stream.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    stream
        .connect(parse_endpoint(endpoint))
        .await
        .inspect_err(|e| error!("Failed to connect: {:?}", e))
        .map_err(|_| BarrierError::Disconnected)?;
    debug!("Connected");

    let _size = stream.read_packet_size().await?;
    if stream.read_bytes_fixed::<7>().await? == [b'B', b'a', b'r', b'r', b'i', b'e', b'r'] {
        debug!("Got hello");
    } else {
        error!("Got invalid hello");
        return Err(BarrierError::ProtocolError(
            super::error::PacketError::FormatError,
        ));
    }
    let major = stream.read_u16().await?;
    let minor = stream.read_u16().await?;
    debug!("Got hello {}:{}", major, minor);

    stream
        .write_u32("Barrier".len() as u32 + 2 + 2 + 4 + device_name.bytes().len() as u32)
        .await?;
    stream
        .write_all(b"Barrier")
        .await
        .map_err(|_| BarrierError::ProtocolError(super::error::PacketError::IoError))?;
    stream.write_u16(1).await?;
    stream.write_u16(6).await?;
    stream.write_str(device_name.as_ref()).await?;

    actor.connected().await?;
    watchdog.feed();

    let mut last_seq_num: u32 = 0;
    let mut clipboard_stage = ClipboardStage::None;

    let mut packet_stream = PacketStream::new(stream);
    while let Ok(packet) = packet_stream.read(&mut clipboard_stage).await {
        match packet {
            Packet::QueryInfo => {
                match packet_stream
                    .write(Packet::DeviceInfo {
                        x: 0,
                        y: 0,
                        w: screen_size.0,
                        h: screen_size.1,
                        _dummy: 0,
                        mx: 0,
                        my: 0,
                    })
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        actor.disconnected().await?;
                        Err(e)
                    }
                }?;
            }
            Packet::KeepAlive => {
                match packet_stream.write(Packet::KeepAlive).await {
                    Ok(_) => {
                        info!("Feed watchdog on KeepAlive");
                        watchdog.feed();
                        Ok(())
                    }
                    Err(e) => {
                        actor.disconnected().await?;
                        Err(e)
                    }
                }?;
            }
            Packet::MouseMoveAbs { x, y } => {
                // There is no `ceil` function in `no_std` environment
                let abs_x = (((x as u32 * 0x7fff) + (screen_size.0 as u32) - 1)
                    / (screen_size.0 as u32)) as u16;
                let abs_y = (((y as u32 * 0x7fff) + (screen_size.1 as u32) - 1)
                    / (screen_size.1 as u32)) as u16;
                // let abs_x = ((x as f32) * (0x7fff as f32 / (screen_size.0 as f32))).ceil() as u16;
                // let abs_y = ((y as f32) * (0x7fff as f32 / (screen_size.1 as f32))).ceil() as u16;
                actor.set_cursor_position(abs_x, abs_y).await?;
            }
            Packet::MouseMove { x, y } => {
                actor.move_cursor(x, y).await?;
            }
            Packet::KeyUp { id, mask, button } => {
                actor.key_up(id, mask, button).await?;
            }
            Packet::KeyDown { id, mask, button } => {
                actor.key_down(id, mask, button).await?;
            }
            Packet::KeyRepeat {
                id,
                mask,
                button,
                count,
            } => {
                actor.key_repeat(id, mask, button, count).await?;
            }
            Packet::MouseDown { id } => {
                actor.mouse_down(id).await?;
            }
            Packet::MouseUp { id } => {
                actor.mouse_up(id).await?;
            }
            Packet::MouseWheel { x_delta, y_delta } => {
                actor.mouse_wheel(x_delta, y_delta).await?;
            }
            Packet::InfoAck => { //Ignore
            }
            Packet::CursorEnter { seq_num, .. } => {
                last_seq_num = seq_num;
                info!("Cursor enter: seq_num:{}", last_seq_num);
                actor.enter().await?;
            }
            Packet::CursorLeave => {
                actor.leave().await?;
            }
            Packet::GrabClipboard { id, seq_num } => {
                info!("Grab clipboard: id:{}, seq_num:{}", id, seq_num);
            }
            Packet::SetClipboard { id, seq_num, data } => {
                debug!(
                    "Set clipboard: id:{}, seq_num:{}, data:{:?}",
                    id, seq_num, data
                );
                if let Some(data) = data {
                    actor.set_clipboard(data).await?;
                }
            }
            Packet::DeviceInfo { .. } | Packet::ErrorUnknownDevice | Packet::ClientNoOp => {
                // Server only packets
            }
            Packet::ServerBusy => {
                warn!("Server is busy, disconnecting");
                break;
            }
            Packet::Unknown(cmd) => {
                debug!(
                    "Unknown packet: {}",
                    core::str::from_utf8(&cmd).unwrap_or("????")
                );
            }
        }
    }
    actor.disconnected().await?;
    Err(BarrierError::Disconnected)
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

fn parse_endpoint<Ep: AsRef<str>>(s: Ep) -> IpEndpoint {
    let s = s.as_ref();
    let mut parts = s.split(':');
    let ip = parts.next().expect("invalid ip endpoint");
    let port = parts.next().expect("invalid ip endpoint");
    let ip = parse_addr(ip);
    let port = port.parse().expect("invalid port");
    IpEndpoint::from((ip, port))
}
