use std::io::{Read, Write};

pub trait BinProtSize {
    fn binprot_size(&self) -> usize;
}

pub trait BinProtWrite {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
}

pub trait BinProtRead {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, crate::error::Error>
    where
        Self: Sized;
}

struct SizeWrite(usize);

impl Write for SizeWrite {
    fn write(&mut self, data: &[u8]) -> std::result::Result<usize, std::io::Error> {
        let len = data.len();
        self.0 += len;
        Ok(len)
    }
    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        Ok(())
    }
}

impl SizeWrite {
    fn new() -> Self {
        SizeWrite(0)
    }
}

impl<T: BinProtWrite> BinProtSize for T {
    fn binprot_size(&self) -> usize {
        let mut w = SizeWrite::new();
        self.binprot_write(&mut w).unwrap();
        w.0
    }
}
