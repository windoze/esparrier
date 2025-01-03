use core::cmp::min;

use heapless::Vec;

use super::{error::PacketError, packet_io::PacketReader};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ClipboardFormat {
    Text = 0,
    Html = 1,
    Bitmap = 2,
}

use crate::clipboard::MAX_CLIPBOARD_SIZE;

pub async fn parse_clipboard<T: PacketReader>(
    stream: &mut T,
) -> Result<(Option<Vec<u8, MAX_CLIPBOARD_SIZE>>, usize), PacketError> {
    let mut consumed: usize = 0;

    let _sz = stream.read_u32().await?;
    consumed += 4;
    let mut ret = Vec::new();
    let num_formats = stream.read_u32().await?;
    consumed += 4;

    for _ in 0..num_formats {
        let format = stream.read_u32().await?;
        consumed += 4;
        let mut length = stream.read_u32().await? as usize;
        consumed += 4;

        let format = match format {
            0 => ClipboardFormat::Text,
            1 => ClipboardFormat::Html,
            2 => ClipboardFormat::Bitmap,
            _ => Err(PacketError::FormatError)?,
        };

        if format == ClipboardFormat::Text {
            while length > 0 {
                let mut buf = [0; 16];
                let read_length = min(length, 16);
                stream
                    .read_exact(&mut buf[0..read_length])
                    .await
                    .map_err(|_| PacketError::IoError)?;
                consumed += read_length;
                length -= read_length;
                if ret.len() < (MAX_CLIPBOARD_SIZE - read_length) {
                    ret.extend_from_slice(&buf[0..read_length]).unwrap();
                }
            }
        }
    }
    if ret.is_empty() {
        Ok((None, consumed))
    } else {
        Ok((Some(ret), consumed))
    }
}
