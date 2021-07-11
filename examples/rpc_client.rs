// RPC Client compatible with https://github.com/janestreet/async_rpc_kernel
// This can be used with the example server (in OCaml) available here:
// https://github.com/janestreet/async/blob/v0.14/async_rpc/example/rpc_server.ml
//
// RPC magic number 4_411_474
use anyhow::Result;
use binprot::{BinProtRead, BinProtSize, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(BinProtRead, BinProtWrite, Debug, Clone, PartialEq)]
struct Handshake(Vec<i64>);

#[derive(BinProtRead, BinProtWrite, Clone, PartialEq)]
enum Sexp {
    Atom(String),
    List(Vec<Sexp>),
}

// Dummy formatter, escaping is not handled properly.
impl std::fmt::Debug for Sexp {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Sexp::Atom(atom) => {
                if atom.contains(|c: char| !c.is_alphanumeric()) {
                    fmt.write_str("\"")?;
                    for c in atom.escape_default() {
                        std::fmt::Write::write_char(fmt, c)?;
                    }
                    fmt.write_str("\"")?;
                } else {
                    fmt.write_str(&atom)?;
                }
                Ok(())
            }
            Sexp::List(list) => {
                fmt.write_str("(")?;
                for (index, sexp) in list.iter().enumerate() {
                    if index > 0 {
                        fmt.write_str(" ")?;
                    }
                    sexp.fmt(fmt)?;
                }
                fmt.write_str(")")?;
                Ok(())
            }
        }
    }
}

#[derive(BinProtRead, BinProtWrite, Debug, Clone, PartialEq)]
struct Query<T> {
    rpc_tag: String,
    version: i64,
    id: i64,
    data: binprot::WithLen<T>,
}

#[derive(BinProtRead, BinProtWrite, Debug, Clone, PartialEq)]
#[polymorphic_variant]
enum Version {
    Version(i64),
}

#[derive(BinProtRead, BinProtWrite, Debug, Clone, PartialEq)]
enum RpcError {
    BinIoExn(Sexp),
    ConnectionClosed,
    WriteError(Sexp),
    UncaughtExn(Sexp),
    UnimplementedRpc((String, Version)),
    UnknownQueryId(String),
}

#[derive(BinProtRead, BinProtWrite, Debug, Clone, PartialEq)]
enum RpcResult<T> {
    Ok(binprot::WithLen<T>),
    Error(RpcError),
}

#[derive(BinProtRead, BinProtWrite, Debug, Clone, PartialEq)]
struct Response<T> {
    id: i64,
    data: RpcResult<T>,
}

#[derive(BinProtRead, BinProtWrite, Debug, Clone, PartialEq)]
enum Message<Q, R> {
    Heartbeat,
    Query(Query<Q>),
    Response(Response<R>),
}

fn read_bin_prot<T: BinProtRead>(stream: &mut TcpStream, buffer: &mut Vec<u8>) -> Result<T> {
    let mut recv_bytes = [0u8; 8];
    stream.read_exact(&mut recv_bytes)?;
    let recv_len = i64::from_le_bytes(recv_bytes);
    buffer.resize(recv_len as usize, 0u8);
    stream.read_exact(buffer)?;
    let mut slice = buffer.as_slice();
    let data = T::binprot_read(&mut slice)?;
    Ok(data)
}

fn write_bin_prot<T: BinProtWrite>(stream: &mut TcpStream, v: &T) -> Result<()> {
    let len = v.binprot_size();
    stream.write_all(&len.to_le_bytes())?;
    v.binprot_write(stream)?;
    Ok(())
}

trait JRpc {
    const NAME: &'static str;
    const VERSION: i64;
    type Q;
    type R;
}

struct RpcGetUniqueId;
impl JRpc for RpcGetUniqueId {
    const NAME: &'static str = "get-unique-id";
    const VERSION: i64 = 0;
    type Q = ();
    type R = i64;
}

struct RpcGetUniqueIdTypo;
impl JRpc for RpcGetUniqueIdTypo {
    const NAME: &'static str = "get-unique-id2";
    const VERSION: i64 = 0;
    type Q = ();
    type R = i64;
}

struct RpcSetIdCounter;
impl JRpc for RpcSetIdCounter {
    const NAME: &'static str = "set-id-counter";
    const VERSION: i64 = 1;
    type Q = i64;
    type R = ();
}

struct RpcClient {
    stream: std::net::TcpStream,
    buffer: Vec<u8>,
    id: i64,
}

impl RpcClient {
    fn connect(address: &str) -> Result<Self> {
        let mut stream = TcpStream::connect(address)?;
        let mut buffer = vec![0u8; 256];
        println!("Successfully connected to {}", address);
        let handshake: Handshake = read_bin_prot(&mut stream, &mut buffer)?;
        println!("Received {:?}", handshake);
        write_bin_prot(&mut stream, &handshake)?;
        Ok(RpcClient {
            stream,
            buffer,
            id: 0,
        })
    }

    fn dispatch<T: JRpc>(&mut self, query: T::Q) -> Result<Response<T::R>>
    where
        T::Q: BinProtWrite,
        T::R: BinProtRead,
    {
        self.id = self.id + 1;
        let query = Query {
            rpc_tag: T::NAME.to_owned(),
            version: T::VERSION,
            id: self.id,
            data: binprot::WithLen(query),
        };
        write_bin_prot(&mut self.stream, &Message::Query::<T::Q, ()>(query))?;
        loop {
            let received: Message<(), T::R> =
                read_bin_prot(&mut self.stream, &mut self.buffer).unwrap();
            match received {
                Message::Heartbeat => (),
                Message::Response(r) => return Ok(r),
                Message::Query(_) => (),
            }
        }
    }
}

fn main() -> Result<()> {
    let mut client = RpcClient::connect("localhost:8080")?;
    let response = client.dispatch::<RpcGetUniqueId>(())?;
    println!(">> {:?}", response);
    let response = client.dispatch::<RpcGetUniqueId>(())?;
    println!(">> {:?}", response);
    let response = client.dispatch::<RpcSetIdCounter>(42)?;
    println!(">> {:?}", response);
    let response = client.dispatch::<RpcGetUniqueId>(())?;
    println!(">> {:?}", response);
    let response = client.dispatch::<RpcSetIdCounter>(0)?;
    println!(">> {:?}", response);
    let response = client.dispatch::<RpcGetUniqueIdTypo>(())?;
    println!(">> {:?}", response);
    Ok(())
}
