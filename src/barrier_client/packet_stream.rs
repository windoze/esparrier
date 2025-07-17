use embedded_io_async::Read as AsyncRead;
use log::debug;

#[cfg(feature = "clipboard")]
use crate::barrier_client::{client::ClipboardStage, clipboard::parse_clipboard};

use super::{error::PacketError, packet::Packet, packet_io::PacketReader, packet_io::PacketWriter};

pub struct PacketStream<S: PacketReader + PacketWriter> {
    stream: S,
}

impl<S: PacketReader + PacketWriter> PacketStream<S> {
    pub fn new(stream: S) -> Self {
        Self { stream }
    }

    pub async fn read(
        &mut self,
        #[cfg(feature = "clipboard")] clipboard_stage: &mut ClipboardStage,
    ) -> Result<Packet, PacketError> {
        let size = self.stream.read_packet_size().await?;
        if size < 4 {
            let mut buf = [0; 4];
            self.stream
                .read_exact(&mut buf[0..size as usize])
                .await
                .map_err(|_| PacketError::PacketTooSmall)?;
            return Err(PacketError::PacketTooSmall);
        }
        Self::do_read(
            &mut self.stream,
            size as usize,
            #[cfg(feature = "clipboard")]
            clipboard_stage,
        )
        .await
    }

    async fn do_read<T: AsyncRead + Unpin>(
        chunk: &mut T,
        mut limit: usize,
        #[cfg(feature = "clipboard")] clipboard_stage: &mut ClipboardStage,
    ) -> Result<Packet, PacketError> {
        let code: [u8; 4] = chunk.read_bytes_fixed().await?;
        limit -= 4;

        let packet = match code.as_ref() {
            b"QINF" => Packet::QueryInfo,
            b"CIAK" => Packet::InfoAck,
            b"CALV" => Packet::KeepAlive,
            b"EUNK" => Packet::UnknownDevice,
            b"EBSY" => Packet::ServerBusy,
            b"DMMV" => {
                let x = chunk.read_u16().await?;
                limit -= 2;
                let y = chunk.read_u16().await?;
                limit -= 2;
                Packet::MouseMoveAbs { x, y }
            }
            b"DMRM" => {
                let x = chunk.read_i16().await?;
                limit -= 2;
                let y = chunk.read_i16().await?;
                limit -= 2;
                Packet::MouseMove { x, y }
            }
            b"CINN" => {
                let x = chunk.read_u16().await?;
                limit -= 2;
                let y = chunk.read_u16().await?;
                limit -= 2;
                let seq_num = chunk.read_u32().await?;
                limit -= 4;
                let mask = chunk.read_u16().await?;
                limit -= 2;
                Packet::CursorEnter {
                    x,
                    y,
                    seq_num,
                    mask,
                }
            }
            b"COUT" => Packet::CursorLeave,
            b"CCLP" => {
                let id = chunk.read_u8().await?;
                limit -= 1;
                let seq_num = chunk.read_u32().await?;
                limit -= 4;
                Packet::GrabClipboard { id, seq_num }
            }
            #[cfg(feature = "clipboard")]
            b"DCLP" => {
                use log::{debug, warn};
                let id = chunk.read_u8().await?;
                let seq_num = chunk.read_u32().await?;
                let mark = chunk.read_u8().await?;
                limit -= 6;
                debug!("Clipboard id: {id}, seq: {seq_num}, mark: {mark}, payload size: {limit}");

                // mark 1 is the total length string in ASCII
                // mark 2 is the actual data and is split into chunks
                // mark 3 is an empty chunk
                debug!("Current Clipboard stage: {clipboard_stage:?}");
                *clipboard_stage = match mark {
                    1 => match clipboard_stage {
                        ClipboardStage::None => ClipboardStage::Mark1,
                        ClipboardStage::Mark3 => ClipboardStage::Mark1,
                        _ => {
                            warn!("Unexpected clipboard stage: {clipboard_stage:?}");
                            ClipboardStage::None
                        }
                    },
                    2 => match clipboard_stage {
                        // 1st mark 2 chunk
                        ClipboardStage::Mark1 => ClipboardStage::Mark2(0),
                        ClipboardStage::Mark2(idx) => ClipboardStage::Mark2(*idx + 1),
                        _ => {
                            warn!("Unexpected clipboard stage: {clipboard_stage:?}");
                            ClipboardStage::None
                        }
                    },
                    3 => match clipboard_stage {
                        ClipboardStage::Mark2(_) => ClipboardStage::Mark3,
                        _ => {
                            warn!("Unexpected clipboard stage: {clipboard_stage:?}");
                            ClipboardStage::None
                        }
                    },
                    _ => {
                        warn!("Unexpected clipboard mark: {mark}");
                        ClipboardStage::None
                    }
                };
                // We only process the 1st mark 2 chunk
                if *clipboard_stage == ClipboardStage::Mark2(0) {
                    let (data, consumed) = parse_clipboard(chunk).await?;
                    limit -= consumed;
                    Packet::SetClipboard { id, seq_num, data }
                } else {
                    Packet::SetClipboard {
                        id,
                        seq_num,
                        data: None,
                    }
                }
            }
            b"DMUP" => {
                let id = chunk.read_i8().await?;
                limit -= 1;
                Packet::MouseUp { id }
            }
            b"DMDN" => {
                let id = chunk.read_i8().await?;
                limit -= 1;
                Packet::MouseDown { id }
            }
            b"DKUP" => {
                let id = chunk.read_u16().await?;
                limit -= 2;
                let mask = chunk.read_u16().await?;
                limit -= 2;
                let button = chunk.read_u16().await?;
                limit -= 2;
                Packet::KeyUp { id, mask, button }
            }
            b"DKDN" => {
                let id = chunk.read_u16().await?;
                limit -= 2;
                let mask = chunk.read_u16().await?;
                limit -= 2;
                let button = chunk.read_u16().await?;
                limit -= 2;
                Packet::KeyDown { id, mask, button }
            }
            b"DKRP" => {
                let id = chunk.read_u16().await?;
                limit -= 2;
                let mask = chunk.read_u16().await?;
                limit -= 2;
                let count = chunk.read_u16().await?;
                limit -= 2;
                let button = chunk.read_u16().await?;
                limit -= 2;
                Packet::KeyRepeat {
                    id,
                    mask,
                    button,
                    count,
                }
            }
            b"DMWM" => {
                let x_delta = chunk.read_i16().await?;
                limit -= 2;
                let y_delta = chunk.read_i16().await?;
                limit -= 2;
                Packet::MouseWheel { x_delta, y_delta }
            }
            b"EICV" => {
                let major = chunk.read_u16().await?;
                limit -= 2;
                let minor = chunk.read_u16().await?;
                limit -= 2;
                Packet::IncompatibleVersion { major, minor }
            }
            b"CROP" => Packet::ResetOptions,
            b"EBAD" => Packet::BadProtocol,
            b"CBYE" => Packet::GoodBye,
            b"DSOP" => {
                // TODO: Maybe there is any option we should care about?
                let num_options = chunk.read_u32().await? / 2;
                limit -= 4;
                for _ in 0..num_options {
                    let mut buf = [0; 4];
                    chunk
                        .read_exact(&mut buf)
                        .await
                        .map_err(|_| PacketError::InsufficientDataError)?;
                    limit -= 4;
                    let value = chunk.read_u32().await?;
                    limit -= 4;
                    debug!("Option: {buf:?}, value: {value}");
                }
                Packet::ClientNoOp
            }
            _ => Packet::Unknown(code),
        };

        // Discard the rest of the packet
        chunk.discard_exact(limit).await?;

        Ok(packet)
    }

    pub async fn write(&mut self, packet: Packet) -> Result<(), PacketError> {
        packet.write_wire(&mut self.stream).await
    }
}
