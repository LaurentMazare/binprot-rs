use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

const CODE_NEG_INT8: u8 = 0xff;
const CODE_INT16: u8 = 0xfe;
const CODE_INT32: u8 = 0xfd;
const CODE_INT64: u8 = 0xfc;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    UnexpectedVariantIndex { index: u8, ident: &'static str },
    UnexpectedPolymorphicVariantIndex { index: i32, ident: &'static str },
    UnexpectedValueForUnit(u8),
    UnexpectedValueForBool(u8),
    UnexpectedValueForOption(u8),
    Utf8Error(std::str::Utf8Error),
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

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::Utf8Error(e)
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

/// This uses the "size-prefixed binary protocol".
/// https://ocaml.janestreet.com/ocaml-core/v0.13/doc/async_unix/Async_unix/Writer/index.html#val-write_bin_prot
pub fn binprot_write_with_size<W: Write, B: BinProtWrite>(b: &B, w: &mut W) -> std::io::Result<()> {
    let len = b.binprot_size();
    w.write_i64::<byteorder::LittleEndian>(len as i64)?;
    b.binprot_write(w)
}

/// This also uses the "size-prefixed binary protocol".
pub fn binprot_read_with_size<R: Read, B: BinProtRead>(r: &mut R) -> Result<B, Error> {
    // TODO: use the length value to avoid reading more that the specified number of bytes.
    let _len = r.read_i64::<byteorder::LittleEndian>()?;
    B::binprot_read(r)
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

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
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

impl BinProtWrite for () {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&[0u8])
    }
}

impl BinProtWrite for bool {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let b = if *self { 1 } else { 0 };
        w.write_all(&[b])
    }
}

impl<T: BinProtWrite> BinProtWrite for Option<T> {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        match &*self {
            None => w.write_all(&[0u8]),
            Some(v) => {
                w.write_all(&[1u8])?;
                v.binprot_write(w)
            }
        }
    }
}

impl<T: BinProtWrite> BinProtWrite for Vec<T> {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        write_nat0(w, self.len() as u64)?;
        for v in self.iter() {
            v.binprot_write(w)?
        }
        Ok(())
    }
}

impl<T: BinProtWrite> BinProtWrite for &[T] {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        write_nat0(w, self.len() as u64)?;
        for v in self.iter() {
            v.binprot_write(w)?
        }
        Ok(())
    }
}

impl BinProtWrite for String {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let bytes = self.as_bytes();
        write_nat0(w, bytes.len() as u64)?;
        w.write_all(&bytes)
    }
}

macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<$($name: BinProtWrite),+> BinProtWrite for ($($name,)+)
        {
            #[allow(non_snake_case)]
            fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
                let ($($name,)+) = self;
                $($name.binprot_write(w)?;)+
                Ok(())
            }
        }

        impl<$($name: BinProtRead),+> BinProtRead for ($($name,)+)
        {
            #[allow(non_snake_case)]
            fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
            where
                Self: Sized,
            {
                $(let $name = $name::binprot_read(r)?;)+
                Ok(($($name,)+))
            }
        }
    };
}

tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
tuple_impls! { A B C D E F G H I }

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

impl BinProtRead for () {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let c = r.read_u8()?;
        if c == 0 {
            Ok(())
        } else {
            Err(Error::UnexpectedValueForUnit(c))
        }
    }
}

impl BinProtRead for bool {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let c = r.read_u8()?;
        if c == 0 {
            Ok(false)
        } else if c == 1 {
            Ok(true)
        } else {
            Err(Error::UnexpectedValueForBool(c))
        }
    }
}

impl<T: BinProtRead> BinProtRead for Option<T> {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let c = r.read_u8()?;
        if c == 0 {
            Ok(None)
        } else if c == 1 {
            let v = T::binprot_read(r)?;
            Ok(Some(v))
        } else {
            Err(Error::UnexpectedValueForOption(c))
        }
    }
}

impl<T: BinProtRead> BinProtRead for Vec<T> {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let len = read_nat0(r)?;
        let mut v: Vec<T> = Vec::new();
        for _i in 0..len {
            let item = T::binprot_read(r)?;
            v.push(item)
        }
        Ok(v)
    }
}

impl BinProtRead for String {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let len = read_nat0(r)?;
        let mut buf: Vec<u8> = vec![0u8; len as usize];
        r.read_exact(&mut buf)?;
        let str = std::str::from_utf8(&buf)?;
        Ok(str.to_string())
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithLen<T>(pub T);

impl<T: BinProtWrite + BinProtSize> BinProtWrite for WithLen<T> {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let len = self.0.binprot_size();
        write_nat0(w, len as u64)?;
        self.0.binprot_write(w)
    }
}

impl<T: BinProtRead> BinProtRead for WithLen<T> {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        // TODO: stop reading past this length
        let _len = read_nat0(r)?;
        let t = T::binprot_read(r)?;
        Ok(WithLen(t))
    }
}
