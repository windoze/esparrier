use embedded_io_async::Read as AsyncRead;

use super::{error::PacketError, packet::Packet, packet_io::PacketReader, packet_io::PacketWriter};

pub struct PacketStream<S: PacketReader + PacketWriter> {
    stream: S,
}

impl<S: PacketReader + PacketWriter> PacketStream<S> {
    pub fn new(stream: S) -> Self {
        Self { stream }
    }

    pub async fn read(&mut self) -> Result<Packet, PacketError> {
        let size = self.stream.read_packet_size().await?;
        if size < 4 {
            let mut buf = [0; 4];
            self.stream
                .read_exact(&mut buf[0..size as usize])
                .await
                .map_err(|_| PacketError::PacketTooSmall)?;
            return Err(PacketError::PacketTooSmall);
        }
        Self::do_read(&mut self.stream, size as usize).await
    }

    async fn do_read<T: AsyncRead + Unpin>(
        chunk: &mut T,
        mut limit: usize,
    ) -> Result<Packet, PacketError> {
        let code: [u8; 4] = chunk.read_bytes_fixed().await?;
        limit -= 4;

        let packet = match code.as_ref() {
            b"QINF" => Packet::QueryInfo,
            b"CIAK" => Packet::InfoAck,
            b"CALV" => Packet::KeepAlive,
            b"EUNK" => Packet::ErrorUnknownDevice,
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
