# binprot-rs
Bin_prot binary protocols in Rust

[![Build Status](https://github.com/LaurentMazare/binprot-rs/workflows/Continuous%20integration/badge.svg)](https://github.com/LaurentMazare/binprot-rs/actions)
[![Latest version](https://img.shields.io/crates/v/binprot.svg)](https://crates.io/crates/binprot)
[![Documentation](https://docs.rs/binprot/badge.svg)](https://docs.rs/binprot)
![License](https://img.shields.io/crates/l/binprot.svg)


This does not use serde (see
[serde-binprot](https://github.com/LaurentMazare/serde-binprot)) but instead
implements the `derive` macro independently so as to provide better control on
serialization. In particular polymorphic variants can be supported thanks
to this.
