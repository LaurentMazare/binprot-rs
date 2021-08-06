// Support for bin_prot_shape like digest computation.
// https://github.com/janestreet/bin_prot/tree/master/shape
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape<T> {
    Annotate(uuid::Uuid, T),
    Base(uuid::Uuid, Vec<T>),
    Tuple(Vec<T>),
    Record(Vec<(String, T)>),
    Variant(Vec<(String, Vec<T>)>),
    // Polymorphic variants are insensitive to the order the constructors are listed
    PolyVariant(BTreeMap<String, Option<T>>),
    // Left-hand-side of [Application] is a potentially recursive definition: it
    // can refer to itself using [RecApp (i, _)] where [i] is the depth of this
    // application node (how many application nodes are above it).
    // It also has its own scope of type variables so it can not refer to type variables
    // of the enclosing scope.
    Application(T, Vec<T>),
    RecApp(i64, Vec<T>),
    Var(i64),
}

trait Digestible {
    fn digest(&self) -> md5::Digest;
}

impl<T: Digestible> Digestible for (String, T) {
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
            }
            Some(t) => {
                context.consume("some");
                context.consume(<[u8; 16]>::from(t.digest()));
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

impl Digestible for String {
    fn digest(&self) -> md5::Digest {
        md5::compute(self)
    }
}

impl<T: Digestible> Digestible for Shape<T> {
    fn digest(&self) -> md5::Digest {
        let mut context = md5::Context::new();
        match self {
            Shape::Annotate(uuid, t) => {
                context.consume("annotate");
                context.consume(uuid.as_bytes());
                context.consume(<[u8; 16]>::from(t.digest()));
            }
            Shape::Base(uuid, vec) => {
                context.consume("base");
                context.consume(uuid.as_bytes());
                context.consume(<[u8; 16]>::from(vec.digest()))
            }
            Shape::Tuple(vec) => {
                context.consume("tuple");
                context.consume(<[u8; 16]>::from(vec.digest()))
            }
            Shape::Record(vec) => {
                context.consume("record");
                context.consume(<[u8; 16]>::from(vec.digest()))
            }
            Shape::Variant(vec) => {
                context.consume("variant");
                context.consume(<[u8; 16]>::from(vec.digest()))
            }
            Shape::PolyVariant(map) => {
                context.consume("poly_variant");
                context.consume(<[u8; 16]>::from(map.digest()))
            }
            Shape::RecApp(n, vec) => {
                context.consume("rec_app");
                context.consume(<[u8; 16]>::from(md5::compute(n.to_string())));
                context.consume(<[u8; 16]>::from(vec.digest()))
            }
            Shape::Application(t, vec) => {
                context.consume("application");
                context.consume(<[u8; 16]>::from(t.digest()));
                context.consume(<[u8; 16]>::from(vec.digest()))
            }
            Shape::Var(v) => {
                context.consume("var");
                context.consume(<[u8; 16]>::from(md5::compute(v.to_string())));
            }
        }
        context.compute()
    }
}
