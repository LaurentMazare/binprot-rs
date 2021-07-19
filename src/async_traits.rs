use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[async_trait]
pub trait BinProtWriteAsync {
    async fn binprot_write_async<W: AsyncWriteExt>(&self, w: &mut W) -> std::io::Result<()>;
}

#[async_trait]
pub trait BinProtReadAsync {
    async fn binprot_read_async<R: AsyncReadExt + ?Sized>(
        r: &mut R,
    ) -> Result<Self, crate::error::Error>
    where
        Self: Sized;
}
