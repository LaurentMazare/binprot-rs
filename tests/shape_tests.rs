use binprot::macros::BinProtShape;
use binprot::{BinProtShape, Digestible};

fn assert_digest<T: BinProtShape>(digest: &'static str) {
    assert_eq!(format!("{:x}", T::binprot_shape().digest()), digest)
}

#[allow(dead_code)]
#[derive(BinProtShape)]
struct Test1 {
    t: i64,
}

#[allow(dead_code)]
#[derive(BinProtShape)]
struct Test2 {
    t: i64,
    u: f64,
}

#[allow(dead_code)]
#[derive(BinProtShape)]
struct Test3 {
    t: Test1,
    u: Test2,
}

#[allow(dead_code)]
#[derive(BinProtShape)]
struct Test4 {
    t: Test1,
    u: (Test2, Test2),
    v: (Test3, Test3, Test3),
}

#[allow(dead_code)]
#[derive(BinProtShape)]
enum TestVariant {
    Foo,
}

#[allow(dead_code)]
#[derive(BinProtShape)]
enum TestVariant2 {
    Foo,
    Bar(i64),
    Bar2(i64, f64),
    Baz { x: i64, y: f64 },
}

#[allow(dead_code)]
#[derive(BinProtShape)]
#[polymorphic_variant]
enum TestPolyVariant {
    A,
}

#[allow(dead_code)]
#[derive(BinProtShape)]
#[polymorphic_variant]
enum TestPolyVariant2 {
    A(i64),
}

#[allow(dead_code)]
#[derive(BinProtShape)]
#[polymorphic_variant]
enum TestPolyVariant3 {
    A(i64),
    B,
    C(i64, f64),
    D(String),
}

#[allow(dead_code)]
#[derive(BinProtShape)]
#[polymorphic_variant]
enum TestPolyVariant4 {
    B,
    D(String),
    C(i64, f64),
    A(i64),
}

#[allow(dead_code)]
#[derive(BinProtShape)]
struct TestRec(Vec<TestRec>);

#[allow(dead_code)]
#[derive(BinProtShape)]
struct TestRec2 {
    foo: Option<Box<TestRec2>>,
}

#[allow(dead_code)]
#[derive(BinProtShape)]
enum TestRec3 {
    Empty,
    Cons(i64, Box<TestRec3>),
}

#[allow(dead_code)]
#[derive(BinProtShape)]
enum TestRec4 {
    Empty,
    Cons((i64, Box<TestRec4>)),
}

#[test]
fn test_shapes() {
    assert_digest::<i64>("698cfa4093fe5e51523842d37b92aeac");
    assert_digest::<f64>("1fd923acb2dd9c5d401ad5b08b1d40cd");
    assert_digest::<String>("d9a8da25d5656b016fb4dbdc2e4197fb");
    assert_digest::<Test1>("43fa87a0bac7a0bb295f67cdc685aa26");
    assert_digest::<(Test1, Test1)>("d9aa33e00d47eb8eeb7f489b17d78d11");
    assert_digest::<(i64, Test1)>("4455e4c2995a2db383c16d4e99093686");
    assert_digest::<Test2>("485a864ae3ab9d4e12534fd17f64a7c4");
    assert_digest::<Test3>("3a9e779c28768361e904e90f37728927");
    assert_digest::<Test4>("7a412f4ba96d992a85db1d498721b752");
    assert_digest::<Vec<i64>>("4c138035aa69ec9dd8b7a7119090f84a");
    assert_digest::<()>("86ba5df747eec837f0b391dd49f33f9e");
    assert_digest::<Option<i64>>("33fd4ff7bde530bddf13dfa739207fae");
    assert_digest::<Result<i64, String>>("d90ddb29b1dc8ae4416867c01634f2de");
    assert_eq!(format!("{:?}", TestVariant::binprot_shape()), "Variant([(\"Foo\", [])])");
    assert_digest::<TestVariant>("81253431711eb0c9d669d0cf1c5ffea7");
    assert_digest::<TestVariant2>("6b5a9ecfe97b786f98c8b9e502c3d6db");
    assert_eq!(format!("{:?}", TestPolyVariant::binprot_shape()), "PolyVariant({\"A\": None})",);
    assert_digest::<TestPolyVariant>("37dab657a1bd138599a678980804d513");
    assert_digest::<TestPolyVariant2>("d82abba442a26f15f25d121e20b45083");
    assert_digest::<TestPolyVariant3>("534bd89034090512512955f635735d46");
    assert_digest::<TestPolyVariant4>("534bd89034090512512955f635735d46");
    // Recursive types are not handled properly yet if there are multiple
    // recursive types. However for a simple recursive type this works well.
    assert_digest::<TestRec>("4526f4c156fe4f6acde769fcb6262b23");
    assert_eq!(
        format!("{:?}", TestRec2::binprot_shape()),
        "Application(Record([(\"foo\", Base(Uuid(\"option\"), [RecApp(0, [])]))]), [])"
    );
    assert_digest::<TestRec2>("2e92d51efb901fcf492f243fc1c3601d");
    assert_digest::<TestRec4>("a0627068b62aa4530d1891cbe7f5d51e");
    assert_digest::<TestRec3>("2ac39052755cfe456342e727b104f34a");
}
