open Core_kernel

let print_digest shape =
  Bin_prot.Shape.eval_to_digest_string shape |> Stdio.printf "%s\n%!"

module _ = struct
  type t = int [@@deriving bin_io]

  let%expect_test _ =
    print_digest bin_shape_t;
    print_digest bin_shape_int;
    print_digest Int32.bin_shape_t;
    print_digest Int64.bin_shape_t;
    print_digest bin_shape_float;
    print_digest bin_shape_string;
    print_digest bin_shape_bool;
    print_digest bin_shape_char;
    [%expect
      {|
    698cfa4093fe5e51523842d37b92aeac
    698cfa4093fe5e51523842d37b92aeac
    0892f5f3797659e9ecf8a0faa5f76829
    0078f5c24ad346a7066cb6673cd5c3cb
    1fd923acb2dd9c5d401ad5b08b1d40cd
    d9a8da25d5656b016fb4dbdc2e4197fb
    a25306e4c5d30d35adbb5b0462a6b1b3
    84610d32d63dcff5c93f1033ec8cb1d5 |}]
end

module _ = struct
  type t = { t : int } [@@deriving bin_io]

  type u =
    { t : int
    ; u : float
    }
  [@@deriving bin_io]

  type v =
    { t : t
    ; u : u
    }
  [@@deriving bin_io]

  type w =
    { t : t
    ; u : u * u
    ; v : v * v * v
    }
  [@@deriving bin_io]

  let%expect_test _ =
    print_digest bin_shape_t;
    print_digest [%bin_shape: t * t];
    print_digest [%bin_shape: int * t];
    print_digest bin_shape_u;
    print_digest bin_shape_v;
    print_digest bin_shape_w;
    [%expect
      {|
    43fa87a0bac7a0bb295f67cdc685aa26
    d9aa33e00d47eb8eeb7f489b17d78d11
    4455e4c2995a2db383c16d4e99093686
    485a864ae3ab9d4e12534fd17f64a7c4
    3a9e779c28768361e904e90f37728927
    7a412f4ba96d992a85db1d498721b752 |}]
end

module _ = struct
  let print_shape_sexp shape =
    Stdio.printf
      "%s\n"
      (Bin_prot.Shape.eval shape |> Bin_prot.Shape.Canonical.sexp_of_t |> Sexp.to_string);
    print_digest shape

  type variant = Foo [@@deriving bin_io]

  type variant2 =
    | Foo
    | Bar of int
    | Bar2 of int * float
    | Baz of
        { x : int
        ; y : float
        }
  [@@deriving bin_io]

  type simple_rec = { foo : simple_rec option } [@@deriving bin_io]

  type int_list =
    | Empty
    | Cons of (int * int_list)
  [@@deriving bin_io]

  let%expect_test _ =
    print_shape_sexp [%bin_shape: int];
    print_shape_sexp [%bin_shape: int list];
    print_shape_sexp [%bin_shape: int array];
    print_shape_sexp [%bin_shape: int option];
    print_shape_sexp [%bin_shape: unit];
    print_shape_sexp [%bin_shape: variant];
    print_shape_sexp [%bin_shape: variant2];
    print_shape_sexp [%bin_shape: [ `A | `B of int ]];
    print_shape_sexp [%bin_shape: simple_rec];
    print_shape_sexp [%bin_shape: int_list];
    [%expect
      {|
    (Exp(Base int()))
    698cfa4093fe5e51523842d37b92aeac
    (Exp(Base list((Exp(Base int())))))
    4cd553520709511864846bda25c448d0
    (Exp(Base array((Exp(Base int())))))
    4c138035aa69ec9dd8b7a7119090f84a
    (Exp(Base option((Exp(Base int())))))
    33fd4ff7bde530bddf13dfa739207fae
    (Exp(Base unit()))
    86ba5df747eec837f0b391dd49f33f9e
    (Exp(Variant((Foo()))))
    81253431711eb0c9d669d0cf1c5ffea7
    (Exp(Variant((Foo())(Bar((Exp(Base int()))))(Bar2((Exp(Base int()))(Exp(Base float()))))(Baz((Exp(Record((x(Exp(Base int())))(y(Exp(Base float())))))))))))
    6b5a9ecfe97b786f98c8b9e502c3d6db
    (Exp(Poly_variant((sorted((A())(B((Exp(Base int())))))))))
    f08c6a40c6f063d21755d22e9e5f8a2c
    (Exp(Application(Exp(Record((foo(Exp(Base option((Exp(Rec_app 0())))))))))()))
    2e92d51efb901fcf492f243fc1c3601d
    (Exp(Application(Exp(Variant((Empty())(Cons((Exp(Tuple((Exp(Base int()))(Exp(Rec_app 0()))))))))))()))
    a0627068b62aa4530d1891cbe7f5d51e |}]
end
