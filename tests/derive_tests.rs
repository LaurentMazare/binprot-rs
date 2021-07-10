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
    test_roundtrip(breakfasts, 37, None)
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
    test_roundtrip(breakfast, 6, None);
    test_roundtrip(BreakfastMenu::<i64>::Nothing, 1, None);
    test_roundtrip(BreakfastMenu::<i64>::Eggs(42), 2, None);
}
