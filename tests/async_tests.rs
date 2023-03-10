use binprot::macros::{BinProtRead, BinProtWrite};
use std::fmt::Debug;

#[cfg(feature = "async")]
use binprot::{BinProtRead, BinProtWrite};

#[derive(BinProtRead, BinProtWrite, Clone, Debug, PartialEq)]
struct Pancakes(i64);

#[derive(BinProtRead, BinProtWrite, Clone, Debug, PartialEq)]
struct MorePancakes(i64, f64, i64);

#[derive(BinProtRead, BinProtWrite, Clone, Debug, PartialEq)]
struct Breakfasts {
    pancakes: Pancakes,
    more_pancakes: MorePancakes,
    value1: i64,
    value2: (f64, f64),
}

#[cfg(feature = "async")]
async fn roundtrip<
    T: 'static + Clone + BinProtRead + BinProtWrite + PartialEq + Debug + Send + Sync,
>(
    vs: &[T],
) -> Result<(), binprot::Error> {
    let (mut client, mut server) = tokio::io::duplex(1);
    let mut vs_for_spawn = vec![];
    vs_for_spawn.extend_from_slice(vs);
    tokio::spawn(async move {
        let mut buffer = binprot::async_read_write::AsyncBuffer::new(1);
        for v in vs_for_spawn.iter() {
            buffer.write_with_size(&mut client, v).await.unwrap();
        }
    });
    let mut buffer = binprot::async_read_write::AsyncBuffer::new(1);
    for v in vs.iter() {
        let w = buffer.read_with_size(&mut server).await?;
        assert_eq!(*v, w);
    }
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test]
async fn roundtrip_test() -> Result<(), binprot::Error> {
    roundtrip(&vec![Pancakes(42); 100]).await?;
    roundtrip(&[Pancakes(42), Pancakes(43), Pancakes(44)]).await?;
    let breakfasts = Breakfasts {
        pancakes: Pancakes(12),
        more_pancakes: MorePancakes(-123, 2.71828182846, 0),
        value1: -1234567890123456,
        value2: (3.141592, 6535.8979),
    };
    roundtrip(&vec![breakfasts; 100]).await?;
    Ok(())
}
