// This test checks that binprot_derive macros do not have
// issues with some BinProt traits not being imported
use binprot::macros::{BinProtRead, BinProtWrite};

#[derive(BinProtRead, BinProtWrite, Debug, PartialEq)]
struct Pancakes(i64);
