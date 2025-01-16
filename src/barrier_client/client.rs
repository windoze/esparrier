use embassy_net::{tcp::TcpSocket, IpEndpoint, Stack};
use embassy_time::Duration;
use embedded_io_async::Write;
use log::{debug, error, info, warn};

use super::{
    packet::Packet, packet_io::PacketReader, packet_io::PacketWriter, packet_stream::PacketStream,
    Actuator, BarrierError,
};

#[cfg(feature = "clipboard")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardStage {
    None,
    Mark1,
    Mark2(usize),
    Mark3,
}

#[allow(unused_assignments)]
pub async fn start_barrier_client<Actor: Actuator>(
    endpoint: IpEndpoint,
    device_name: &'static str,
    stack: Stack<'_>,
    mut actor: Actor,
) -> Result<(), BarrierError> {
    let screen_size: (u16, u16) = actor.get_screen_size().await?;

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    debug!("Connecting to {}", endpoint);
    let mut stream = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    stream.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    stream
        .connect(endpoint)
        .await
        .inspect_err(|e| error!("Failed to connect: {:?}", e))
        .map_err(|_| BarrierError::Disconnected)?;
    debug!("Connected");
    stream.set_keep_alive(Some(Duration::from_secs(1)));
    stream.set_timeout(Some(Duration::from_secs(3)));

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
    stream.write_str(device_name).await?;

    actor.connected().await?;
    // watchdog.feed();

    #[cfg(feature = "clipboard")]
    let mut clipboard_stage = ClipboardStage::None;

    let mut packet_stream = PacketStream::new(stream);
    while let Ok(packet) = packet_stream
        .read(
            #[cfg(feature = "clipboard")]
            &mut clipboard_stage,
        )
        .await
    {
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
                        debug!("Feed watchdog on KeepAlive");
                        // watchdog.feed();
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
                let abs_x = (x as u32 * 0x7fff).div_ceil(screen_size.0 as u32) as u16;
                let abs_y = (y as u32 * 0x7fff).div_ceil(screen_size.1 as u32) as u16;
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
            Packet::CursorEnter {
                x,
                y,
                seq_num: _dummy,
                mask,
            } => {
                actor.enter(x, y, mask).await?;
            }
            Packet::CursorLeave => {
                actor.leave().await?;
            }
            Packet::GrabClipboard { id, seq_num } => {
                debug!("Grab clipboard: id:{}, seq_num:{}", id, seq_num);
            }
            #[cfg(feature = "clipboard")]
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
            Packet::ResetOptions => {
                info!("Reset options");
            }
            Packet::GoodBye => {
                info!("Goodbye");
                break;
            }
            Packet::BadProtocol => {
                error!("Bad protocol");
                break;
            }
            Packet::IncompatibleVersion { major, minor } => {
                error!("Incompatible version: {}:{}", major, minor);
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
