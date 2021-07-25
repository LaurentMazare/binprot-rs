// RPC Client compatible with https://github.com/janestreet/async_rpc_kernel
// This can be used with the example server (in OCaml) available here:
// https://github.com/janestreet/async/blob/v0.14/async_rpc/example/rpc_server.ml
//
// RPC magic number 4_411_474
use anyhow::Result;
use binprot::{BinProtRead, BinProtSize, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

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
    let len = v.binprot_size() as i64;
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

trait JRpcImpl {
    type Q; // Query
    type R; // Response
    type E; // Error

    fn rpc_impl(&mut self, q: Self::Q) -> std::result::Result<Self::R, Self::E>;
}

trait ErasedJRpcImpl {
    fn erased_rpc_impl(&mut self, stream: &mut TcpStream, id: i64) -> Result<()>;
}

//impl<Q, R, E> ErasedJRpcImpl for dyn JRpcImpl<Q = Q, R = R, E = E>
//where
//   Q: BinProtRead,
//    R: BinProtWrite,
//    E: std::error::Error,
impl<T> ErasedJRpcImpl for T
where
    T: JRpcImpl,
    T::Q: BinProtRead,
    T::R: BinProtWrite,
    T::E: std::error::Error,
{
    fn erased_rpc_impl(&mut self, stream: &mut TcpStream, id: i64) -> Result<()> {
        let query = T::Q::binprot_read(stream)?;
        let rpc_result = match self.rpc_impl(query) {
            Ok(response) => RpcResult::Ok(binprot::WithLen(response)),
            Err(error) => {
                let sexp = Sexp::Atom(error.to_string());
                RpcResult::Error(RpcError::UncaughtExn(sexp))
            }
        };
        let response = Response {
            id,
            data: rpc_result,
        };
        write_bin_prot(stream, &Message::Response::<(), T::R>(response))?;
        Ok(())
    }
}

#[allow(dead_code)]
struct RpcServer {
    listener: TcpListener,
    buffer: Vec<u8>,
    id: i64,
    rpc_impls: BTreeMap<String, Box<dyn ErasedJRpcImpl>>,
}

struct GetUniqueIdImpl(i64);

impl JRpcImpl for GetUniqueIdImpl {
    type Q = ();
    type R = i64;
    type E = std::convert::Infallible;

    fn rpc_impl(&mut self, _q: Self::Q) -> std::result::Result<Self::R, Self::E> {
        let result = self.0;
        self.0 += 1;
        Ok(result)
    }
}

// It is not easy to use [Query] on the server side as we do not
// know which rpcs will be triggered. So instead we use this type
// that only parses up to the length of the payload.
// This only works because the payload appears last in the
// serialized representation
#[derive(BinProtRead, BinProtWrite, Debug, Clone, PartialEq)]
struct ServerQuery {
    rpc_tag: String,
    version: i64,
    id: i64,
    data: binprot::Nat0,
}

#[derive(BinProtRead, BinProtWrite, Debug, Clone, PartialEq)]
enum ServerMessage<R> {
    Heartbeat,
    Query(ServerQuery),
    Response(R),
}

impl RpcServer {
    fn bind(address: &str) -> Result<Self> {
        let listener = TcpListener::bind(address)?;
        let buffer = vec![0u8; 256];
        println!("Successfully bound to {}", address);
        let mut rpc_impls: BTreeMap<String, Box<dyn ErasedJRpcImpl>> = BTreeMap::new();
        let get_unique_id_impl: Box<dyn ErasedJRpcImpl> = Box::new(GetUniqueIdImpl(0));
        rpc_impls.insert("get-unique-id".to_string(), get_unique_id_impl);
        Ok(RpcServer {
            listener,
            buffer,
            id: 0,
            rpc_impls,
        })
    }

    fn run(&mut self) -> Result<()> {
        for stream in self.listener.incoming() {
            let mut stream = stream?;
            println!("Got connection {:?}.", stream);
            write_bin_prot(&mut stream, &Handshake(vec![4411474, 1]))?;
            let handshake: Handshake = read_bin_prot(&mut stream, &mut self.buffer)?;
            println!("Received handshake {:?}", handshake);
            let mut recv_bytes = [0u8; 8];
            loop {
                // We don't know the type of rpcs that will be received so the
                // following parses the incoming messages in a "manual" way.
                stream.read_exact(&mut recv_bytes)?;
                let _recv_len = i64::from_le_bytes(recv_bytes);
                let query = ServerMessage::<()>::binprot_read(&mut stream)?;
                println!("Received rpc query {:?}", query);
                match query {
                    ServerMessage::Heartbeat => {}
                    ServerMessage::Query(query) => match self.rpc_impls.get_mut(&query.rpc_tag) {
                        None => {
                            let err = RpcError::UnimplementedRpc((
                                query.rpc_tag,
                                Version::Version(query.version),
                            ));
                            let message = ServerMessage::Response(Response::<()> {
                                id: query.id,
                                data: RpcResult::Error(err),
                            });
                            self.buffer.resize(query.data.0 as usize, 0u8);
                            stream.read_exact(&mut self.buffer)?;
                            write_bin_prot(&mut stream, &message)?
                        }
                        Some(r) => r.erased_rpc_impl(&mut stream, query.id)?,
                    },
                    ServerMessage::Response(()) => unimplemented!(),
                };
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let arg = std::env::args().skip(1).next();

    match arg.as_deref() {
        Some("client") => {
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
        }
        Some("server") => {
            let mut server = RpcServer::bind("localhost:8080")?;
            server.run()?
        }
        Some(_) => {
            panic!("unexpected argument, try client or server")
        }
        None => {
            panic!("missing argument")
        }
    }
    Ok(())
}
