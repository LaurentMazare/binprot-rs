// Support for bin_prot_shape like digest computation.
// https://github.com/janestreet/bin_prot/tree/master/shape
// TODO: handle recursive types!
use std::collections::BTreeMap;

// In the OCaml version, uuids are used as strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uuid(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape<T> {
    Annotate(Uuid, T),
    Base(Uuid, Vec<T>),
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

impl Digestible for Uuid {
    fn digest(&self) -> md5::Digest {
        md5::compute(&self.0)
    }
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

impl<T: Digestible> Digestible for Shape<T> {
    fn digest(&self) -> md5::Digest {
        match self {
            Shape::Annotate(uuid, t) => {
                Constructor::new("annotate").add_digest(uuid).add_digest(t).finish()
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
                Constructor::new("application").add_digest(t).add_digest(vec).finish()
            }
            Shape::Var(v) => Constructor::new("var").add_digest(&v.to_string()).finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl From<&str> for Uuid {
        fn from(s: &str) -> Self {
            Uuid(s.to_string())
        }
    }

    fn base(s: &str) -> Shape<String> {
        Shape::<String>::Base(Uuid::from(s), vec![])
    }

    #[test]
    fn shape_digest() {
        let digest = format!("{:x}", base("int").digest());
        assert_eq!(digest, "698cfa4093fe5e51523842d37b92aeac");
        let digest = format!("{:x}", base("int64").digest());
        assert_eq!(digest, "0078f5c24ad346a7066cb6673cd5c3cb");
        let digest = format!("{:x}", Shape::Record(vec![("t".to_string(), base("int"))]).digest());
        assert_eq!(digest, "43fa87a0bac7a0bb295f67cdc685aa26");
    }
}
