use binprot::{BinProtRead, BinProtSize, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};

#[derive(BinProtRead, BinProtWrite, Debug, PartialEq)]
struct Pancakes(i64);

#[test]
fn breakfast1() {
    let pancakes = Pancakes(12);
    assert_eq!(pancakes.binprot_size(), 1);
    let mut data: Vec<u8> = Vec::new();
    pancakes.binprot_write(&mut data).unwrap();
    let mut slice = data.as_slice();
    assert_eq!(slice, [12]);
    let flipped = Pancakes::binprot_read(&mut slice).unwrap();
    assert_eq!(pancakes, flipped)
}

#[derive(BinProtRead, BinProtWrite, Debug, PartialEq)]
struct MorePancakes(i64, f64, i64);

#[test]
fn breakfast2() {
    let more_pancakes = MorePancakes(12, 3.141592, 1234567890123);
    assert_eq!(more_pancakes.binprot_size(), 18);
    let mut data: Vec<u8> = Vec::new();
    more_pancakes.binprot_write(&mut data).unwrap();
    let mut slice = data.as_slice();
    assert_eq!(
        slice,
        [12, 122, 0, 139, 252, 250, 33, 9, 64, 252, 203, 4, 251, 113, 31, 1, 0, 0]
    );
    let flipped = MorePancakes::binprot_read(&mut slice).unwrap();
    assert_eq!(more_pancakes, flipped)
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
    assert_eq!(breakfasts.binprot_size(), 37);
    let mut data: Vec<u8> = Vec::new();
    breakfasts.binprot_write(&mut data).unwrap();
    let mut slice = data.as_slice();
    let flipped = Breakfasts::binprot_read(&mut slice).unwrap();
    assert_eq!(breakfasts, flipped)
}

#[derive(BinProtWrite, Debug, PartialEq)]
enum BreakfastMenu<T> {
    Any(T),
    Eggs(i64),
    Pancakes(Pancakes),
    MorePancakes(MorePancakes),
    LotsOfPancakes(Pancakes, MorePancakes),
    Everything { eggs: i64, pancakes: i64 },
}

#[test]
fn breakfast4() {
    let breakfast: BreakfastMenu<BreakfastMenu<i64>> =
        BreakfastMenu::Any(BreakfastMenu::Everything {
            eggs: 123,
            pancakes: 456,
        });
    assert_eq!(breakfast.binprot_size(), 6);
    let mut data: Vec<u8> = Vec::new();
    breakfast.binprot_write(&mut data).unwrap();
    let mut slice = data.as_slice();
    assert_eq!(slice, [0, 5, 123, 254, 200, 1],);
    // let flipped = BreakfastMenu::binprot_read(&mut slice).unwrap();
    // assert_eq!(breakfast, flipped)
}
