use binprot::{BinProtRead, BinProtSize, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};

fn test_roundtrip<T>(t: T, sz: usize, vs: Option<&[u8]>)
where
    T: BinProtRead + BinProtWrite + PartialEq + std::fmt::Debug,
{
    assert_eq!(t.binprot_size(), sz);
    let mut data: Vec<u8> = Vec::new();
    t.binprot_write(&mut data).unwrap();
    let mut slice = data.as_slice();
    if let Some(vs) = vs {
        assert_eq!(slice, vs);
    }
    let flipped = T::binprot_read(&mut slice).unwrap();
    assert_eq!(t, flipped)
}

#[derive(BinProtRead, BinProtWrite, Debug, PartialEq)]
struct Pancakes(i64);

#[test]
fn breakfast1() {
    test_roundtrip(Pancakes(12), 1, Some(&[12]));
    test_roundtrip(Pancakes(0), 1, Some(&[0]));
    test_roundtrip(Pancakes(-1), 2, Some(&[255, 255]));
    test_roundtrip(
        Pancakes(12345678910111213),
        9,
        Some(&[252, 237, 189, 242, 93, 84, 220, 43, 0]),
    );
}

#[derive(BinProtRead, BinProtWrite, Debug, PartialEq)]
struct MorePancakes(i64, f64, i64);

#[test]
fn breakfast2() {
    test_roundtrip(
        MorePancakes(12, 3.141592, 1234567890123),
        18,
        Some(&[
            12, 122, 0, 139, 252, 250, 33, 9, 64, 252, 203, 4, 251, 113, 31, 1, 0, 0,
        ]),
    );
}

#[derive(BinProtRead, BinProtWrite, Debug, PartialEq)]
struct Breakfasts {
    pancakes: Pancakes,
    more_pancakes: MorePancakes,
    value1: i64,
    value2: (f64, f64),
}

#[test]
fn breakfast3() {
    let breakfasts = Breakfasts {
        pancakes: Pancakes(12),
        more_pancakes: MorePancakes(-123, 2.71828182846, 0),
        value1: -1234567890123456,
        value2: (3.141592, 6535.8979),
    };
    // Generated in ocaml.
    let expected = [
        12, 255, 133, 207, 95, 20, 139, 10, 191, 5, 64, 0, 252, 64, 69, 117, 195, 42, 157, 251,
        255, 122, 0, 139, 252, 250, 33, 9, 64, 20, 63, 198, 220, 229, 135, 185, 64,
    ];
    test_roundtrip(breakfasts, 37, Some(&expected))
}

#[derive(BinProtWrite, BinProtRead, Debug, PartialEq)]
enum BreakfastMenu<T> {
    Any(T),
    Eggs(i64),
    Pancakes(Pancakes),
    MorePancakes(MorePancakes),
    LotsOfPancakes(Pancakes, MorePancakes),
    Everything { eggs: i64, pancakes: i64 },
    Nothing,
}

#[test]
fn breakfast4() {
    let breakfast: BreakfastMenu<BreakfastMenu<i64>> =
        BreakfastMenu::Any(BreakfastMenu::Everything {
            eggs: 123,
            pancakes: 456,
        });
    let expected = [0, 5, 123, 254, 200, 1];
    test_roundtrip(breakfast, 6, Some(&expected));
    test_roundtrip(BreakfastMenu::<i64>::Nothing, 1, None);
    let expected = [1, 42];
    test_roundtrip(BreakfastMenu::<i64>::Eggs(42), 2, Some(&expected));
    test_roundtrip(
        binprot::WithLen(BreakfastMenu::<i64>::Eggs(42)),
        3,
        Some(&[2, 1, 42]),
    );
}

#[derive(BinProtWrite, BinProtRead, Debug, PartialEq)]
struct BreakfastItem {
    name: String,
    quantity: f64,
    large: bool,
}

#[test]
fn breakfast5() {
    let expected = [3, 101, 103, 103, 111, 18, 131, 192, 202, 33, 9, 64, 1];
    test_roundtrip(
        BreakfastItem {
            name: "egg".to_string(),
            quantity: 3.1415,
            large: true,
        },
        13,
        Some(&expected),
    );
    let expected = [
        9, 99, 114, 111, 105, 115, 115, 97, 110, 116, 0, 0, 0, 0, 128, 28, 200, 192, 0,
    ];
    test_roundtrip(
        BreakfastItem {
            name: "croissant".to_string(),
            quantity: -12345.,
            large: false,
        },
        19,
        Some(&expected),
    );
    let expected = [
        14, 80, 97, 105, 110, 65, 117, 67, 104, 111, 99, 111, 108, 97, 116, 0, 0, 0, 74, 120, 222,
        177, 65, 0,
    ];
    test_roundtrip(
        BreakfastItem {
            name: "PainAuChocolat".to_string(),
            quantity: 299792458.0,
            large: false,
        },
        24,
        Some(&expected),
    );
}

#[derive(BinProtWrite, BinProtRead, Debug, PartialEq)]
#[polymorphic_variant]
enum BreakfastPoly<T> {
    Any(T),
    Eggs(i64),
    Pancakes(Pancakes),
    MorePancakes(MorePancakes),
    LotsOfPancakes(Pancakes, MorePancakes),
    Everything { eggs: i64, pancakes: i64 },
    Nothing,
}

#[test]
fn breakfast6() {
    let breakfast: BreakfastPoly<BreakfastPoly<i64>> = BreakfastPoly::Any(
        BreakfastPoly::MorePancakes(MorePancakes(-123, 2.71828182846, 0)),
    );
    let expected = [
        153, 101, 99, 0, 39, 152, 92, 190, 255, 133, 207, 95, 20, 139, 10, 191, 5, 64, 0,
    ];
    test_roundtrip(breakfast, 19, Some(&expected));
    test_roundtrip(BreakfastPoly::<i64>::Nothing, 4, None);
    let expected = [93, 118, 212, 91, 42];
    test_roundtrip(BreakfastPoly::<i64>::Eggs(42), 5, Some(&expected));
    test_roundtrip(
        binprot::WithLen(BreakfastPoly::<i64>::Eggs(42)),
        6,
        Some(&[5, 93, 118, 212, 91, 42]),
    );
}

#[test]
fn breakfast7() {
    let price_and_quantities: std::collections::HashMap<String, (i64, f64)> = [
        ("croissant", (4, 1.23)),
        ("JusDOrange", (1, 2.34)),
        ("PainAuChocolat", (2, 1.45)),
    ]
    .iter()
    .map(|(x, y)| (x.to_string(), *y))
    .collect();
    test_roundtrip(price_and_quantities.clone(), 64, None);
    let price_and_quantities: std::collections::BTreeMap<String, (i64, f64)> =
        price_and_quantities.into_iter().collect();
    test_roundtrip(price_and_quantities, 64, None);
}
