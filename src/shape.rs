// Support for bin_prot_shape like digest computation.
// https://github.com/janestreet/bin_prot/tree/master/shape
// TODO: handle recursive types!
use crate::traits::ShapeContext;
use crate::BinProtShape;
use std::collections::{BTreeMap, HashMap};

// In the OCaml version, uuids are used as strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uuid(&'static str);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape {
    Annotate(Uuid, Box<Shape>),
    Base(Uuid, Vec<Shape>),
    Tuple(Vec<Shape>),
    Record(Vec<(&'static str, Shape)>),
    Variant(Vec<(&'static str, Vec<Shape>)>),
    // Polymorphic variants are insensitive to the order the constructors are listed
    PolyVariant(BTreeMap<&'static str, Option<Shape>>),
    Application(Box<Shape>, Vec<Shape>),
    RecApp(i64, Vec<Shape>),
    Var(i64),
}

pub trait Digestible {
    fn digest(&self) -> md5::Digest;
}

impl Digestible for Uuid {
    fn digest(&self) -> md5::Digest {
        md5::compute(self.0)
    }
}

impl<T: Digestible> Digestible for (&'static str, T) {
    fn digest(&self) -> md5::Digest {
        let mut context = md5::Context::new();
        context.consume(<[u8; 16]>::from(self.0.digest()));
        context.consume(<[u8; 16]>::from(self.1.digest()));
        context.compute()
    }
}

impl<T1: Digestible, T2: Digestible> Digestible for (&T1, &T2) {
    fn digest(&self) -> md5::Digest {
        let mut context = md5::Context::new();
        context.consume(<[u8; 16]>::from(self.0.digest()));
        context.consume(<[u8; 16]>::from(self.1.digest()));
        context.compute()
    }
}

impl<T: Digestible> Digestible for [T] {
    fn digest(&self) -> md5::Digest {
        let mut context = md5::Context::new();
        for elem in self.iter() {
            context.consume(<[u8; 16]>::from(elem.digest()));
        }
        context.compute()
    }
}

impl<T1: Digestible, T2: Digestible> Digestible for BTreeMap<T1, T2> {
    fn digest(&self) -> md5::Digest {
        let mut context = md5::Context::new();
        for key_value in self.iter() {
            context.consume(<[u8; 16]>::from(key_value.digest()));
        }
        context.compute()
    }
}

impl<T: Digestible> Digestible for Option<T> {
    fn digest(&self) -> md5::Digest {
        let mut context = md5::Context::new();
        match self {
            None => {
                context.consume("none");
                context.consume(<[u8; 16]>::from("".digest()));
            }
            Some(t) => {
                context.consume("some");
                let mut inner = md5::Context::new();
                inner.consume(<[u8; 16]>::from(t.digest()));
                context.consume(<[u8; 16]>::from(inner.compute()));
            }
        }
        context.compute()
    }
}

impl<T: Digestible, E: Digestible> Digestible for Result<T, E> {
    fn digest(&self) -> md5::Digest {
        let mut context = md5::Context::new();
        match self {
            Ok(t) => {
                context.consume("ok");
                let mut inner = md5::Context::new();
                inner.consume(<[u8; 16]>::from(t.digest()));
                context.consume(<[u8; 16]>::from(inner.compute()));
            }
            Err(t) => {
                context.consume("err");
                let mut inner = md5::Context::new();
                inner.consume(<[u8; 16]>::from(t.digest()));
                context.consume(<[u8; 16]>::from(inner.compute()));
            }
        }
        context.compute()
    }
}

impl<T: Digestible> Digestible for Vec<T> {
    fn digest(&self) -> md5::Digest {
        self.as_slice().digest()
    }
}

impl Digestible for &str {
    fn digest(&self) -> md5::Digest {
        md5::compute(self)
    }
}

impl Digestible for String {
    fn digest(&self) -> md5::Digest {
        md5::compute(self)
    }
}

struct Constructor {
    context: md5::Context,
    inner: md5::Context,
}

impl Constructor {
    fn new(cons_name: &str) -> Constructor {
        let mut context = md5::Context::new();
        let inner = md5::Context::new();
        context.consume(cons_name);
        Constructor { context, inner }
    }

    fn add_digest<T: Digestible>(mut self, t: &T) -> Self {
        self.inner.consume(<[u8; 16]>::from(t.digest()));
        self
    }

    fn finish(mut self) -> md5::Digest {
        self.context.consume(<[u8; 16]>::from(self.inner.compute()));
        self.context.compute()
    }
}

impl Digestible for Shape {
    fn digest(&self) -> md5::Digest {
        match self {
            Shape::Annotate(uuid, t) => {
                Constructor::new("annotate").add_digest(uuid).add_digest(&**t).finish()
            }
            Shape::Base(uuid, vec) => {
                Constructor::new("base").add_digest(uuid).add_digest(vec).finish()
            }
            Shape::Tuple(vec) => Constructor::new("tuple").add_digest(vec).finish(),
            Shape::Record(vec) => Constructor::new("record").add_digest(vec).finish(),
            Shape::Variant(vec) => Constructor::new("variant").add_digest(vec).finish(),
            Shape::PolyVariant(map) => Constructor::new("poly_variant").add_digest(map).finish(),
            Shape::RecApp(n, vec) => {
                Constructor::new("rec_app").add_digest(&n.to_string()).add_digest(vec).finish()
            }
            Shape::Application(t, vec) => {
                Constructor::new("application").add_digest(&**t).add_digest(vec).finish()
            }
            Shape::Var(v) => Constructor::new("var").add_digest(&v.to_string()).finish(),
        }
    }
}

impl From<&'static str> for Uuid {
    fn from(s: &'static str) -> Self {
        Uuid(s)
    }
}

fn base(s: &'static str) -> Shape {
    Shape::Base(Uuid::from(s), vec![])
}

impl BinProtShape for i64 {
    fn binprot_shape_impl(_: &mut ShapeContext) -> Shape {
        base("int")
    }
}

impl BinProtShape for f64 {
    fn binprot_shape_impl(_: &mut ShapeContext) -> Shape {
        base("float")
    }
}

impl BinProtShape for String {
    fn binprot_shape_impl(_: &mut ShapeContext) -> Shape {
        base("string")
    }
}

impl BinProtShape for bool {
    fn binprot_shape_impl(_: &mut ShapeContext) -> Shape {
        base("bool")
    }
}

impl BinProtShape for char {
    fn binprot_shape_impl(_: &mut ShapeContext) -> Shape {
        base("char")
    }
}

impl BinProtShape for i32 {
    fn binprot_shape_impl(_: &mut ShapeContext) -> Shape {
        base("i32")
    }
}

impl BinProtShape for () {
    fn binprot_shape_impl(_: &mut ShapeContext) -> Shape {
        base("unit")
    }
}

impl<T: BinProtShape> BinProtShape for Vec<T> {
    fn binprot_shape_impl(c: &mut ShapeContext) -> Shape {
        Shape::Base(Uuid::from("array"), vec![T::binprot_shape_loop(c)])
    }
}

fn iterable_binable1_shape(caller_identity: Uuid, bin_shape_el: Shape) -> Shape {
    Shape::Base(
        caller_identity,
        vec![Shape::Base(Uuid::from("ac8a9ff4-4994-11e6-9a1b-9fb4e933bd9d"), vec![bin_shape_el])],
    )
}

impl<K: BinProtShape, V: BinProtShape, S> BinProtShape for HashMap<K, V, S>
where
    S: 'static,
{
    fn binprot_shape_impl(c: &mut ShapeContext) -> Shape {
        let caller_identity = Uuid::from("8fabab0a-4992-11e6-8cca-9ba2c4686d9e");
        let bin_shape_el = Shape::Tuple(vec![K::binprot_shape_loop(c), V::binprot_shape_loop(c)]);
        iterable_binable1_shape(caller_identity, bin_shape_el)
    }
}

impl<K: BinProtShape, V: BinProtShape> BinProtShape for BTreeMap<K, V> {
    fn binprot_shape_impl(c: &mut ShapeContext) -> Shape {
        let caller_identity = Uuid::from("dfb300f8-4992-11e6-9c15-73a2ac6b815c");
        let bin_shape_el = Shape::Tuple(vec![K::binprot_shape_loop(c), V::binprot_shape_loop(c)]);
        iterable_binable1_shape(caller_identity, bin_shape_el)
    }
}

impl<T: BinProtShape> BinProtShape for Option<T> {
    fn binprot_shape_impl(c: &mut ShapeContext) -> Shape {
        Shape::Base(Uuid::from("option"), vec![T::binprot_shape_loop(c)])
    }
}

impl<T: BinProtShape, E: BinProtShape> BinProtShape for Result<T, E> {
    fn binprot_shape_impl(c: &mut ShapeContext) -> Shape {
        Shape::Base(Uuid::from("result"), vec![T::binprot_shape_loop(c), E::binprot_shape_loop(c)])
    }
}

impl<T: BinProtShape> BinProtShape for Box<T> {
    fn binprot_shape_impl(c: &mut ShapeContext) -> Shape {
        T::binprot_shape_loop(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest_str(s: &Shape) -> String {
        format!("{:x}", s.digest())
    }

    #[test]
    fn shape_digest() {
        assert_eq!(digest_str(&base("int")), "698cfa4093fe5e51523842d37b92aeac");
        assert_eq!(digest_str(&base("int32")), "0892f5f3797659e9ecf8a0faa5f76829");
        assert_eq!(digest_str(&base("int64")), "0078f5c24ad346a7066cb6673cd5c3cb");
        assert_eq!(digest_str(&base("string")), "d9a8da25d5656b016fb4dbdc2e4197fb");
        assert_eq!(digest_str(&base("float")), "1fd923acb2dd9c5d401ad5b08b1d40cd");
        assert_eq!(digest_str(&base("bool")), "a25306e4c5d30d35adbb5b0462a6b1b3");
        assert_eq!(digest_str(&base("char")), "84610d32d63dcff5c93f1033ec8cb1d5");
        let shape_t = Shape::Record(vec![("t", base("int"))]);
        assert_eq!(digest_str(&shape_t), "43fa87a0bac7a0bb295f67cdc685aa26");
        let shape_u = Shape::Record(vec![("t", base("int")), ("u", base("float"))]);
        assert_eq!(digest_str(&shape_u), "485a864ae3ab9d4e12534fd17f64a7c4");
        let shape_v = Shape::Record(vec![("t", shape_t), ("u", shape_u)]);
        assert_eq!(digest_str(&shape_v), "3a9e779c28768361e904e90f37728927");
        // Shape used for some recursive type, see tests/shape_tests.ml
        //   type int_list =
        //     | Empty
        //     | Cons of (int * int_list)
        let shape_rec = {
            let inner = Shape::Variant(vec![
                ("Empty", vec![]),
                ("Cons", vec![Shape::Tuple(vec![base("int"), Shape::RecApp(0, vec![])])]),
            ]);
            Shape::Application(Box::new(inner), vec![])
        };
        assert_eq!(digest_str(&shape_rec), "a0627068b62aa4530d1891cbe7f5d51e");
        // type simple_rec = { foo : simple_rec option } [@@deriving bin_io]
        let shape_rec = {
            let inner = Shape::Record(vec![(
                "foo",
                Shape::Base("option".into(), vec![Shape::RecApp(0, vec![])]),
            )]);
            Shape::Application(Box::new(inner), vec![])
        };
        assert_eq!(digest_str(&shape_rec), "2e92d51efb901fcf492f243fc1c3601d");
        let shape_i64_i64_hashtbl = {
            std::collections::HashMap::<i64, i64>::binprot_shape_impl(&mut ShapeContext::default())
        };
        assert_eq!(digest_str(&shape_i64_i64_hashtbl), "1fd943a5d8026fbd3e6746c972ab2127");
        let shape_i64_i64_btreemap = {
            std::collections::BTreeMap::<i64, i64>::binprot_shape_impl(&mut ShapeContext::default())
        };
        assert_eq!(digest_str(&shape_i64_i64_btreemap), "ed73a010af8ffc32cab7411d6be2d676");
    }
}
