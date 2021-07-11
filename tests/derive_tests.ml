open Core_kernel

let print_binio t bin_writer_t =
  let buf = Bin_prot.Utils.bin_dump bin_writer_t t in
  let len = Bin_prot.Common.buf_len buf in
  let ts = Array.init len ~f:(fun i -> buf.{i} |> Char.to_int) in
  Stdio.printf "%s\n%!" ([%sexp_of: int array] ts |> Sexp.to_string_mach)

module MorePancakes = struct
  type t = int * float * int [@@deriving bin_io, sexp_of]

  let%expect_test _ =
    let t = 12, 3.141592, 1234567890123 in
    print_binio t bin_writer_t;
    [%expect {|
    (12 122 0 139 252 250 33 9 64 252 203 4 251 113 31 1 0 0) |}]
end

module Breakfasts = struct
  type t =
    { pancakes : int
    ; more_pancakes : MorePancakes.t
    ; value1 : int
    ; value2 : float * float
    }
  [@@deriving bin_io, sexp_of]

  let%expect_test _ =
    let t =
      { pancakes = 12
      ; more_pancakes = -123, 2.71828182846, 0
      ; value1 = -1234567890123456
      ; value2 = 3.141592, 6535.8979
      }
    in
    print_binio t bin_writer_t;
    [%expect
      {|
    (12 255 133 207 95 20 139 10 191 5 64 0 252 64 69 117 195 42 157 251 255 122 0 139 252 250 33 9 64 20 63 198 220 229 135 185 64) |}]
end

module BreakfastItem = struct
  type t =
    { name : string
    ; quantity : float
    ; large : bool
    }
  [@@deriving bin_io]

  let%expect_test _ =
    let t = { name = "egg"; quantity = 3.1415; large = true } in
    print_binio t bin_writer_t;
    let t = { name = "croissant"; quantity = -12345.; large = false } in
    print_binio t bin_writer_t;
    let t = { name = "PainAuChocolat"; quantity = 299792458.; large = false } in
    print_binio t bin_writer_t;
    [%expect
      {|
    (3 101 103 103 111 18 131 192 202 33 9 64 1)
    (9 99 114 111 105 115 115 97 110 116 0 0 0 0 128 28 200 192 0)
    (14 80 97 105 110 65 117 67 104 111 99 111 108 97 116 0 0 0 74 120 222 177 65 0) |}]
end

module BreakfastMenu = struct
  type 'a t =
    | Any of 'a
    | Eggs of int
    | Pancakes of int
    | MorePancakes of MorePancakes.t
    | LotsOfPancakes of int * MorePancakes.t
    | Everything of
        { eggs : int
        ; pancakes : int
        }
    | Nothing
  [@@deriving bin_io]

  let%expect_test _ =
    let t = Any (Everything { eggs = 123; pancakes = 456 }) in
    print_binio t (bin_writer_t (bin_writer_t Int.bin_writer_t));
    let t = Eggs 42 in
    print_binio t (bin_writer_t (bin_writer_t Int.bin_writer_t));
    [%expect
      {|
    (0 5 123 254 200 1)
    (1 42) |}]
end
