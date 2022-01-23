use crate::Shape;
use std::collections::HashMap;
use std::io::{Read, Write};

pub type ShapeContext = HashMap<std::any::TypeId, bool>;

pub trait BinProtShape: 'static {
    fn binprot_shape_impl(_: &mut ShapeContext) -> Shape;

    fn binprot_shape_loop(typeids: &mut ShapeContext) -> Shape {
        let typeid = std::any::TypeId::of::<Self>();
        if typeids.contains_key(&typeid) {
            // TODO: Adjust the parameters.
            typeids.insert(typeid, true);
            Shape::RecApp(0, vec![])
        } else {
            typeids.insert(typeid, false);
            let shape = Self::binprot_shape_impl(typeids);
            match typeids.remove(&typeid) {
                None | Some(false) => shape,
                Some(true) => Shape::Application(Box::new(shape), vec![]),
            }
        }
    }

    fn binprot_shape() -> Shape {
        let mut typeids = HashMap::new();
        Self::binprot_shape_loop(&mut typeids)
    }
}

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
