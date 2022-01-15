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
    [%expect {|
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

  let%expect_test _ =
    print_digest bin_shape_t;
    [%expect {|
    43fa87a0bac7a0bb295f67cdc685aa26 |}]
end
