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
//#[derive(BinProtShape)]
//#[polymorphic_variant]
enum TestPolyVariant {
    A,
    B(i64),
}

#[allow(dead_code)]
#[derive(BinProtShape)]
struct TestRec(Vec<TestRec>);

#[allow(dead_code)]
#[derive(BinProtShape)]
struct TestRec2 {
    foo: Option<Box<TestRec2>>,
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
    assert_eq!(format!("{:?}", TestVariant::binprot_shape()), "Variant([(\"Foo\", [])])");
    assert_digest::<TestVariant>("81253431711eb0c9d669d0cf1c5ffea7");
    assert_digest::<TestVariant2>("6b5a9ecfe97b786f98c8b9e502c3d6db");
    // assert_digest::<TestPolyVariant>("81253431711eb0c9d669d0cf1c5ffea7");
    // Recursive types are not handled properly yet, the following
    // result in a stack overflow.
    // assert_digest::<TestRec>("86ba5df747eec837f0b391dd49f33f9e");
    // assert_digest::<TestRec2>("2e92d51efb901fcf492f243fc1c3601d");
}