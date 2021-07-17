# binprot-rs
Bin_prot binary protocols in Rust

[![Build Status](https://github.com/LaurentMazare/binprot-rs/workflows/Continuous%20integration/badge.svg)](https://github.com/LaurentMazare/binprot-rs/actions)
[![Latest version](https://img.shields.io/crates/v/binprot.svg)](https://crates.io/crates/binprot)
[![Documentation](https://docs.rs/binprot/badge.svg)](https://docs.rs/binprot)
![License](https://img.shields.io/crates/l/binprot.svg)

This crates provides [bin_prot](https://github.com/janestreet/bin_prot) serialization
and tries to be compatible with the OCaml version for similar types.

The `examples` directory includes a tiny RPC implementation compatible with
OCaml [Async_rpc](https://github.com/janestreet/async/tree/master/async_rpc).
The `Query` message is defined as follows in OCaml as can be found in the
[implementation](https://github.com/janestreet/async_rpc_kernel/blob/v0.14/src/protocol.ml#L61-L70).
```ocaml
module Query = struct
  type 'a needs_length =
    { tag     : Rpc_tag.t
    ; version : int
    ; id      : Query_id.t
    ; data    : 'a
    }
  [@@deriving bin_io]
  type 'a t = 'a needs_length [@@deriving bin_read]
end
```

The equivalent type using Rust would be:
```rust
#[derive(BinProtRead, BinProtWrite)]
struct Query<T> {
    rpc_tag: String,
    version: i64,
    id: i64,
    data: binprot::WithLen<T>,
}
```

This does not use serde (see
[serde-binprot](https://github.com/LaurentMazare/serde-binprot)) but instead
implements the `derive` macro independently so as to provide better control on
serialization. In particular polymorphic variants can be supported thanks
to this.
