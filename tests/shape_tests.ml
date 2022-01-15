open Core_kernel

let print_digest shape =
  Bin_prot.Shape.eval_to_digest_string shape |> Stdio.printf "%s\n%!"

module _ = struct
  type t = int [@@deriving bin_io]

  let%expect_test _ =
    print_digest bin_shape_t;
    [%expect {|
    698cfa4093fe5e51523842d37b92aeac |}]
end

module _ = struct
  type t = Int64.t [@@deriving bin_io]

  let%expect_test _ =
    print_digest bin_shape_t;
    [%expect {|
    0078f5c24ad346a7066cb6673cd5c3cb |}]
end

module _ = struct
  type t = { t : int } [@@deriving bin_io]

  let%expect_test _ =
    print_digest bin_shape_t;
    [%expect {|
    43fa87a0bac7a0bb295f67cdc685aa26 |}]
end
