use crate::error::Error;
use crate::{BinProtRead, BinProtWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Buffer(Vec<u8>);

impl Buffer {
    pub fn new(buf_size: usize) -> Self {
        Buffer(Vec::with_capacity(buf_size))
    }

    pub async fn read_with_size<T: BinProtRead, R: AsyncReadExt + Unpin>(
        &mut self,
        r: &mut R,
    ) -> Result<T, Error> {
        let buf = &mut self.0;
        let mut recv_bytes = [0u8; 8];
        r.read_exact(&mut recv_bytes).await?;
        let recv_len = i64::from_le_bytes(recv_bytes);
        buf.resize(recv_len as usize, 0u8);
        r.read_exact(buf).await?;
        let data = T::binprot_read(&mut buf.as_slice())?;
        Ok(data)
    }

    pub async fn write_bin_prot<T: BinProtWrite, W: AsyncWriteExt + Unpin>(
        &mut self,
        w: &mut W,
        v: &T,
    ) -> std::io::Result<()> {
        let buf = &mut self.0;
        buf.clear();
        v.binprot_write(buf)?;
        let len = buf.len() as i64;
        w.write_all(&len.to_le_bytes()).await?;
        w.write_all(buf).await?;
        Ok(())
    }
}
