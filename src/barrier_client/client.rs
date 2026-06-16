use embassy_net::{IpEndpoint, Stack, tcp::TcpSocket};
use embassy_time::{Duration, Instant, TimeoutError, with_timeout};
use embedded_io_async::Write;
use log::{debug, error, info, warn};

use crate::get_running_state;

use super::{
    Actuator, BarrierError, packet::Packet, packet_io::PacketReader, packet_io::PacketWriter,
    packet_stream::PacketStream,
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
    jiggle_interval: u16,
    stack: Stack<'_>,
    mut actor: Actor,
) -> Result<(), BarrierError> {
    let screen_size: (u16, u16) = actor.get_screen_size().await?;

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    debug!("Connecting to {endpoint}");
    let mut stream = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    stream.set_keep_alive(Some(Duration::from_secs(1)));
    stream.set_timeout(Some(Duration::from_secs(3)));

    stream
        .connect(endpoint)
        .await
        .inspect_err(|e| error!("Failed to connect: {e:?}"))
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
    debug!("Got hello {major}:{minor}");

    stream
        .write_u32("Barrier".len() as u32 + 2 + 2 + 4 + device_name.len() as u32)
        .await?;
    stream
        .write_all(b"Barrier")
        .await
        .map_err(|_| BarrierError::ProtocolError(super::error::PacketError::IoError))?;
    stream.write_u16(1).await?;
    stream.write_u16(6).await?;
    stream.write_str(device_name).await?;

    actor.connected().await?;

    #[cfg(feature = "clipboard")]
    let mut clipboard_stage = ClipboardStage::None;

    // Track last user action time for jiggle control
    let mut last_action_time = Instant::now();

    let mut packet_stream = PacketStream::new(stream);
    loop {
        match with_timeout(
            Duration::from_secs(jiggle_interval as u64),
            packet_stream.read(
                #[cfg(feature = "clipboard")]
                &mut clipboard_stage,
            ),
        )
        .await
        {
            Err(TimeoutError) => {
                // Periodical tasks
                if get_running_state().await.keep_awake {
                    // Only jiggle if there has been no user action for the jiggle_interval
                    if last_action_time.elapsed() >= Duration::from_secs(jiggle_interval as u64) {
                        actor.jiggle().await?;
                    }
                }
            }
            Ok(Err(e)) => {
                error!("Error: {e:?}");
                break;
            }
            Ok(Ok(packet)) => {
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
                                if get_running_state().await.keep_awake {
                                    // Only jiggle if there has been no user action for the jiggle_interval
                                    if last_action_time.elapsed()
                                        >= Duration::from_secs(jiggle_interval as u64)
                                    {
                                        actor.jiggle().await?;
                                    }
                                }
                                Ok(())
                            }
                            Err(e) => {
                                actor.disconnected().await?;
                                Err(e)
                            }
                        }?;
                    }
                    Packet::MouseMoveAbs { x, y } => {
                        actor.set_cursor_position(x, y).await?;
                        last_action_time = Instant::now();
                    }
                    Packet::MouseMove { x, y } => {
                        actor.move_cursor(x, y).await?;
                        last_action_time = Instant::now();
                    }
                    Packet::KeyUp { id, mask, button } => {
                        actor.key_up(id, mask, button).await?;
                        last_action_time = Instant::now();
                    }
                    Packet::KeyDown { id, mask, button } => {
                        actor.key_down(id, mask, button).await?;
                        last_action_time = Instant::now();
                    }
                    Packet::KeyRepeat {
                        id,
                        mask,
                        button,
                        count,
                    } => {
                        actor.key_repeat(id, mask, button, count).await?;
                        last_action_time = Instant::now();
                    }
                    Packet::MouseDown { id } => {
                        actor.mouse_down(id).await?;
                        last_action_time = Instant::now();
                    }
                    Packet::MouseUp { id } => {
                        actor.mouse_up(id).await?;
                        last_action_time = Instant::now();
                    }
                    Packet::MouseWheel { x_delta, y_delta } => {
                        actor.mouse_wheel(x_delta, y_delta).await?;
                        last_action_time = Instant::now();
                    }
                    Packet::CursorEnter {
                        x,
                        y,
                        seq_num: _dummy,
                        mask,
                    } => {
                        actor.enter(x, y, mask).await?;
                        last_action_time = Instant::now();
                    }
                    Packet::CursorLeave => {
                        actor.leave().await?;
                    }
                    Packet::GrabClipboard { id, seq_num } => {
                        debug!("Grab clipboard: id:{id}, seq_num:{seq_num}");
                    }
                    #[cfg(feature = "clipboard")]
                    Packet::SetClipboard { id, seq_num, data } => {
                        debug!("Set clipboard: id:{id}, seq_num:{seq_num}, data:{data:?}");
                        if let Some(data) = data {
                            actor.set_clipboard(data).await?;
                        }
                    }
                    Packet::DeviceInfo { .. }
                    | Packet::ClientNoOp
                    | Packet::InfoAck
                    | Packet::ResetOptions => {
                        // Do nothing
                    }
                    Packet::ServerBusy => {
                        warn!("Server is busy, disconnecting");
                        break;
                    }
                    Packet::GoodBye => {
                        info!("Goodbye");
                        break;
                    }
                    Packet::BadProtocol => {
                        error!("Bad protocol");
                        break;
                    }
                    Packet::UnknownDevice => {
                        error!("Unknown device");
                        break;
                    }
                    Packet::IncompatibleVersion { major, minor } => {
                        error!("Incompatible version: {major}:{minor}");
                        break;
                    }
                    Packet::Unknown(cmd) => {
                        log::info!(
                            "Unknown packet code: '{}' ({:02X} {:02X} {:02X} {:02X})",
                            core::str::from_utf8(&cmd).unwrap_or("????"),
                            cmd[0],
                            cmd[1],
                            cmd[2],
                            cmd[3]
                        );
                    }
                }
            }
        }
    }
    actor.disconnected().await?;
    Err(BarrierError::Disconnected)
}
