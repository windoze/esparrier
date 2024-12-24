use embedded_io_async::Read as AsyncRead;
use embedded_io_async::Write as AsyncWrite;

use super::error::PacketError;

#[allow(dead_code)]
pub trait PacketReader: AsyncRead + Unpin {
    async fn discard_exact(&mut self, len: usize) -> Result<(), PacketError> {
        let mut buf = [0; 16];
        let mut len = len;
        while len > 0 {
            let to_read = core::cmp::min(len, buf.len());
            self.read_exact(&mut buf[..to_read])
                .await
                .map_err(|_| PacketError::InsufficientDataError)?;
            len -= to_read;
        }
        Ok(())
    }

    async fn read_packet_size(&mut self) -> Result<u32, PacketError> {
        self.read_u32().await
    }

    async fn read_u8(&mut self) -> Result<u8, PacketError> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)
            .await
            .map_err(|_| PacketError::InsufficientDataError)?;
        Ok(buf[0])
    }

    async fn read_i8(&mut self) -> Result<i8, PacketError> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)
            .await
            .map_err(|_| PacketError::InsufficientDataError)?;
        Ok(buf[0] as i8)
    }

    async fn read_u16(&mut self) -> Result<u16, PacketError> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)
            .await
            .map_err(|_| PacketError::InsufficientDataError)?;
        Ok(u16::from_be_bytes(buf))
    }

    async fn read_i16(&mut self) -> Result<i16, PacketError> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)
            .await
            .map_err(|_| PacketError::InsufficientDataError)?;
        Ok(i16::from_be_bytes(buf))
    }

    async fn read_u32(&mut self) -> Result<u32, PacketError> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)
            .await
            .map_err(|_| PacketError::InsufficientDataError)?;
        Ok(u32::from_be_bytes(buf))
    }

    async fn read_bytes_fixed<const N: usize>(&mut self) -> Result<[u8; N], PacketError> {
        let mut res = [0; N];
        self.read_exact(&mut res)
            .await
            .map_err(|_| PacketError::InsufficientDataError)?;
        Ok(res)
    }
}

impl<T: AsyncRead + Unpin> PacketReader for T {}

pub trait PacketWriter: AsyncWrite + Unpin {
    async fn write_str(&mut self, data: &str) -> Result<(), PacketError> {
        self.write_u32(data.len() as u32).await?;
        self.write_all(data.as_bytes())
            .await
            .map_err(|_| PacketError::IoError)?;
        Ok(())
    }

    async fn write_u16(&mut self, data: u16) -> Result<(), PacketError> {
        self.write_all(&data.to_be_bytes())
            .await
            .map_err(|_| PacketError::IoError)?;
        Ok(())
    }

    async fn write_u32(&mut self, data: u32) -> Result<(), PacketError> {
        self.write_all(&data.to_be_bytes())
            .await
            .map_err(|_| PacketError::IoError)?;
        Ok(())
    }
}

impl<T: AsyncWrite + Unpin> PacketWriter for T {}
