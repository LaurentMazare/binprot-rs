use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Write};

const CODE_NEG_INT8: u8 = 0xff;
const CODE_INT16: u8 = 0xfe;
const CODE_INT32: u8 = 0xfd;
const CODE_INT64: u8 = 0xfc;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

pub trait BinProtSize {
    fn binprot_size(&self) -> usize;
}

pub trait BinProtWrite {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
}

pub trait BinProtRead {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized;
}

fn write_nat0<W: Write>(w: &mut W, v: u64) -> std::io::Result<()> {
    if v < 0x000000080 {
        w.write_all(&[v as u8])?;
    } else if v < 0x000010000 {
        w.write_all(&[CODE_INT16])?;
        w.write_all(&(v as u16).to_le_bytes())?;
    } else if v < 0x100000000 {
        w.write_all(&[CODE_INT32])?;
        w.write_all(&(v as u32).to_le_bytes())?;
    } else {
        w.write_all(&[CODE_INT64])?;
        w.write_all(&v.to_le_bytes())?;
    }
    Ok(())
}

fn write_i64<W: Write>(w: &mut W, v: i64) -> std::io::Result<()> {
    if 0 <= v {
        if v < 0x000000080 {
            w.write_all(&[v as u8])?;
        } else if v < 0x00008000 {
            w.write_all(&[CODE_INT16])?;
            w.write_all(&(v as u16).to_le_bytes())?;
        } else if v < 0x80000000 {
            w.write_all(&[CODE_INT32])?;
            w.write_all(&(v as u32).to_le_bytes())?;
        } else {
            w.write_all(&[CODE_INT64])?;
            w.write_all(&v.to_le_bytes())?;
        }
    } else if v >= -0x00000080 {
        w.write_all(&[CODE_NEG_INT8])?;
        w.write_all(&v.to_le_bytes()[..1])?;
    } else if v >= -0x00008000 {
        w.write_all(&[CODE_INT16])?;
        w.write_all(&v.to_le_bytes()[..2])?;
    } else if v >= -0x80000000 {
        w.write_all(&[CODE_INT32])?;
        w.write_all(&v.to_le_bytes()[..4])?;
    } else if v < -0x80000000 {
        w.write_all(&[CODE_INT64])?;
        w.write_all(&v.to_le_bytes())?;
    }
    Ok(())
}

fn write_f64<W: Write>(w: &mut W, v: f64) -> std::io::Result<()> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}

fn read_signed<R: Read + ?Sized>(r: &mut R) -> std::io::Result<i64> {
    let c = r.read_u8()?;
    let v = match c {
        CODE_NEG_INT8 => r.read_i8()? as i64,
        CODE_INT16 => r.read_i16::<LittleEndian>()? as i64,
        CODE_INT32 => r.read_i32::<LittleEndian>()? as i64,
        CODE_INT64 => r.read_i64::<LittleEndian>()?,
        c => c as i64,
    };
    Ok(v)
}

fn read_nat0<R: Read + ?Sized>(r: &mut R) -> std::io::Result<u64> {
    let c = r.read_u8()?;
    let v = match c {
        CODE_INT16 => r.read_u16::<LittleEndian>()? as u64,
        CODE_INT32 => r.read_u32::<LittleEndian>()? as u64,
        CODE_INT64 => r.read_u64::<LittleEndian>()?,
        c => c as u64,
    };
    Ok(v)
}

fn read_float<R: Read + ?Sized>(r: &mut R) -> std::io::Result<f64> {
    let f = r.read_f64::<LittleEndian>()?;
    Ok(f)
}

pub struct Nat0(u64);

impl BinProtWrite for Nat0 {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        write_nat0(w, self.0)
    }
}

impl BinProtWrite for i64 {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        write_i64(w, *self)
    }
}

impl BinProtWrite for f64 {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        write_f64(w, *self)
    }
}

impl<A: BinProtWrite, B: BinProtWrite> BinProtWrite for (A, B) {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.0.binprot_write(w)?;
        self.1.binprot_write(w)?;
        Ok(())
    }
}

impl BinProtRead for Nat0 {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let u64 = read_nat0(r)?;
        Ok(Nat0(u64))
    }
}

impl BinProtRead for i64 {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let i64 = read_signed(r)?;
        Ok(i64)
    }
}

impl BinProtRead for f64 {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let f64 = read_float(r)?;
        Ok(f64)
    }
}

impl<A: BinProtRead, B: BinProtRead> BinProtRead for (A, B) {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let a = A::binprot_read(r)?;
        let b = B::binprot_read(r)?;
        Ok((a, b))
    }
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
