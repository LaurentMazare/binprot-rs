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
