use crate::error::Error;
use crate::int::{CODE_INT16, CODE_INT32, CODE_INT64, CODE_NEG_INT8};
use crate::{BinProtRead, BinProtWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct AsyncBuffer(Vec<u8>);

impl AsyncBuffer {
    pub fn new(buf_size: usize) -> Self {
        AsyncBuffer(Vec::with_capacity(buf_size))
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

    pub async fn write_with_size<T: BinProtWrite, W: AsyncWriteExt + Unpin>(
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

pub async fn write_nat0<W: AsyncWriteExt + Unpin>(w: &mut W, v: u64) -> std::io::Result<()> {
    if v < 0x000000080 {
        w.write_all(&[v as u8]).await?;
    } else if v < 0x000010000 {
        w.write_all(&[CODE_INT16]).await?;
        w.write_all(&(v as u16).to_le_bytes()).await?;
    } else if v < 0x100000000 {
        w.write_all(&[CODE_INT32]).await?;
        w.write_all(&(v as u32).to_le_bytes()).await?;
    } else {
        w.write_all(&[CODE_INT64]).await?;
        w.write_all(&v.to_le_bytes()).await?;
    }
    Ok(())
}

pub async fn write_i64<W: AsyncWriteExt + Unpin>(w: &mut W, v: i64) -> std::io::Result<()> {
    if 0 <= v {
        if v < 0x000000080 {
            w.write_all(&[v as u8]).await?;
        } else if v < 0x00008000 {
            w.write_all(&[CODE_INT16]).await?;
            w.write_all(&(v as u16).to_le_bytes()).await?;
        } else if v < 0x80000000 {
            w.write_all(&[CODE_INT32]).await?;
            w.write_all(&(v as u32).to_le_bytes()).await?;
        } else {
            w.write_all(&[CODE_INT64]).await?;
            w.write_all(&v.to_le_bytes()).await?;
        }
    } else if v >= -0x00000080 {
        w.write_all(&[CODE_NEG_INT8]).await?;
        w.write_all(&v.to_le_bytes()[..1]).await?;
    } else if v >= -0x00008000 {
        w.write_all(&[CODE_INT16]).await?;
        w.write_all(&v.to_le_bytes()[..2]).await?;
    } else if v >= -0x80000000 {
        w.write_all(&[CODE_INT32]).await?;
        w.write_all(&v.to_le_bytes()[..4]).await?;
    } else if v < -0x80000000 {
        w.write_all(&[CODE_INT64]).await?;
        w.write_all(&v.to_le_bytes()).await?;
    }
    Ok(())
}

pub async fn read_signed<R: AsyncReadExt + Unpin + ?Sized>(r: &mut R) -> std::io::Result<i64> {
    let c = r.read_u8().await?;
    let v = match c {
        CODE_NEG_INT8 => {
            let i = r.read_i8().await? as i64;
            if i >= 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Neg_int8"));
            }
            i
        }
        CODE_INT16 => r.read_i16_le().await? as i64,
        CODE_INT32 => r.read_i32_le().await? as i64,
        CODE_INT64 => r.read_i64_le().await?,
        c => c as i64,
    };
    Ok(v)
}

pub async fn read_nat0<R: AsyncReadExt + Unpin + ?Sized>(r: &mut R) -> std::io::Result<u64> {
    let c = r.read_u8().await?;
    let v = match c {
        CODE_INT16 => r.read_u16_le().await? as u64,
        CODE_INT32 => r.read_u32_le().await? as u64,
        CODE_INT64 => r.read_u64_le().await?,
        c => c as u64,
    };
    Ok(v)
}
