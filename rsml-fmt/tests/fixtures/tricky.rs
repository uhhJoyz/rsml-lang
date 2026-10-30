// Formatter stress input: only tokenized and expanded by tests/corpus.rs,
// never compiled.

fn comments() {
    let a = ml! { (* leading *) 1 + (* inside *) 2 (* trailing *) };
    let b = ml! {
        // a line comment first
        let x = 1 in (* after in *)
        (* own line before y *)
        let y = x + 1 in


        (* after two blank lines *)
        x + y // trailing line comment
    };
    let c = ml! {
        match xs with
        (* before the first case *)
        | [] -> 0 (* empty *)
        // the interesting one
        | x :: rest -> !x
    };
    let d = ml! {
        f a (* between args *) b
          c
    };
    let e = ml! {
        (* a comment
           spanning lines *)
        let z = 3 in
        z
    };
    let f = ml! { xs |> .iter |> .map _ f (* map *) |> .filter _ g (* filter *) };
}

fn layouts() {
    let long = ml! {
        some_function_name first_argument second_argument third_argument fourth_argument fifth
    };
    let nested = ml! {
        match a with
        | Some x -> begin match b with Some y -> x + y | None -> x end
        | None -> (match c with Some z -> z | None -> 0)
    };
    let lambdas = ml! {
        items |> .into_iter
        |> Iterator::map _ (fun item -> let doubled = item * 2 in let tripled = item * 3 in doubled + tripled)
        |> Iterator::filter _ (fun x -> !x mod 2 = 0)
        |> Iterator::fold _ 0 (fun acc x -> acc + x)
    };
    let method_chain = ml! {
        xs.iter().map(fun x -> x + 1).filter(fun x -> !x > 2).collect::<Vec<i64>>().into_iter().sum::<i64>()
    };
    let records = ml! {
        let p = Point { x = 1; y = 2 } in
        let q = { p with x = 3 } in
        let Point { x; _ } = q in
        match q with Point { x = 0; y } -> y | Point { x; _ } -> x
    };
    let arrays = ml! { ([| -1; 2; -3 |], [||], [1; 2; 3], [], [| f x? |], [|a; b|]) };
    let ops = ml! {
        let ( +! ) a b = a + b + 1 in
        let ( >>= ) m f = match m with Some x -> f x | None -> None in
        Some 1 >>= fun x -> Some (x +! 2)
    };
    let ifs = ml! {
        if a then b else if c then d else if e then f else g
    };
    let long_if = ml! {
        if some_long_condition_name && another_condition then compute_the_first_value 1 2 else compute_the_other_value 3 4
    };
    let loops = ml! {
        let r = ref 0 in
        for i = 1 to 10 do r := !r + i done;
        for i = 10 downto 1 do incr r; decr r done;
        while !r > 0 do r := !r - 1 done;
        !r
    };
    let seq = ml! { print_string "a"; print_string "b"; print_newline () };
}

fn syntax() {
    let labels = ml! { let f ~x ~y:(y : i64) = x + y in f ~x:1 ~y:2 + f ~x ~y };
    let types = ml! {
        external add : i64 -> i64 -> i64 = "crate::util::add" ;;
        external get : &'a [u8] -> Option<i64> -> Result<(i64, String), 'e> = "get" ;;
        let rec go (acc : Vec<i64>) (l : &[i64]) : Vec<i64> = match l with [] -> acc | x :: t -> go (!x :: acc) t
        and unused (x : 'a) : 'a = x ;;
        let v : Vec<i64> = [1] ;;
        let it : Box<dyn Iterator<Item = u8>> = f () ;;
        open std::cmp ;;
        go [] &[1; 2] |> Vec::len |> max 3
    };
    let misc = ml! {
        let mut n = 0 in
        n <- n + 1;
        let t = (1, (2, 3)) in
        let u = t.1.0 + a.(0) + u32::from s.[1] + .len xs + xs.len() + .count (.iter xs) in
        assert (u > 0);
        let neg = - (-n) - -. 1.0 in
        let bits = (n land 3) lor (n lsl 2) lxor (n asr 1) in
        let pw = 2.0 ** 3.0 ** 0.5 in
        let s = "a" ^ "b" ^ "c" in
        let l = [1] @ [2] @ 3 :: [4] in
        let f = fun move x -> x in
        let g = (f >> i64::abs) << i64::signum in
        let h = (( - ) _1 _0) 1 @@ g @@ 3 in
        let sec = (xs |> .iter |> .map _ (_ * 2), clamp 0 _ 10, f _1 _0 _, .len (_ : &str), ( * ) _0 _0) in
        let dots = (- .len xs, [| .len xs |], g ~f:(.len) ~n: .len, .iter >> .count, .collect::<Vec<_>> it) in
        let open std::cmp in
        let c = function 'a' .. 'z' as ch when ch <> 'q' -> 1 | 'A' | 'B' -> 2 | _ -> 3 in
        let k = (x : i64) in
        let o = &mut n in
        let w = &x as &i64 in
        let m = vec![1, 2,
            3] in
        let raw = "multi
line string" in
        println!("{} {}", n,
            s);
        print!("{} {}\n", n, s);
        let () = drop (begin end) in
        (u, neg, bits, pw, l, h, sec, dots, c 'x', k, o, w, m, raw, ( * ), (+), ( mod ))
    };
}

fn directives() {
    let skipped = ml! {
        (* rsmlfmt: skip *)
        let   x =    1 in x
    };
    let narrow = ml! {
        (* rsmlfmt: max_width = 30; pipe.indent = align *)
        xs |> .iter |> .map _ f |> .filter _ g |> .count
    };
}

rsml! {
    // rsmlfmt: let.body = fit

    async fn fetch(url: &str) -> Result<String, Error> {
        let resp = get url .await? in resp.text().await
    }

    impl Shape {
        fn area(&self) -> f64 {
            match self with Shape::Circle r -> 3.14 *. !r *. !r | Shape::Rect (w, h) -> !w *. !h
        }
    }
}
