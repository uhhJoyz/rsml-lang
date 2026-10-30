// Formatter stress input for Rust's syntax inside OCaml: only tokenized and
// expanded by tests/corpus.rs, never compiled. A space before `(` or `[`
// changes the meaning, so the formatter must keep every one as written.

fn calls() {
    let a = ml! { add(1, 2) + add (1, 2) + f(a)(b) + f (a)(b) + f(a) (b) + f() + f (()) + f(()) };
    let b = ml! {
        some_function_with_a_long_name(first_argument, second_argument, third_argument, fourth)
    };
    let c = ml! { xs.iter().map(fun x -> x + 1)(y) |> g(_, 1) |> (f >> g)(x) |> h(fun a -> a)(b) };
    let d = ml! {
        Some(1, 2) :: Pair(a, b) :: Vec::new() :: size_of::<u8>() :: t.0(1, 2) :: (Some)(1) :: []
    };
    let e = ml! { f((* inside *) a, b (* after b *)) (* after *) };
    let f = ml! { register(fun event -> handle event; log event)(more) };
    let g = ml! {
        let add3 x y z = x + y + z in
        add3(1)(2)(3) + add3(1, 2, 3) + add3 (1) (2) (3)
    };
}

fn constructors() {
    // Written against the constructor or spaced, both mean the same and
    // both are kept.
    let a = ml! { Some(1, 2) :: Some (1, 2) :: Shape::Rect(2.0, 3.0) :: Shape.Rect (w, h) :: [] };
    let b = ml! { f Some(x) (Some)(y) Ok(()) None::<i64> Some[1; 2] Some [3] Some(fun x -> x) };
    let c = ml! {
        match shape with
        | Shape::Rect(w, h) when w > h -> Some(w)
        | Shape::Rect (w, _) -> Some (w)
        | Shape::Circle(r) -> Ok(())
    };
    let d = ml! {
        Shape::Rect(compute_width first_argument second_argument, compute_height third_argument)
    };
    // Against the constructor, `.area()` is the shape's; spaced, the argument's.
    let e = ml! { Shape::Rect(w, h).area() + Some(x).unwrap() + Wrapper(v).0 + Some (x).unwrap() };
    let f = ml! {
        Shape::Rect(compute_width first_argument second_argument, compute_height third).area()
        + Shape::Rect (compute_width first_argument second_argument, compute_height third).area()
    };
}

fn indexing() {
    let a = ml! { v[0] + v [0] + m[&k] + f(x)[0] + x.f[1] + t.0[1] + grid[i][j] + a.(i) + s.as_bytes()[0] };
    let b = ml! { f [1; 2] [|3; 4|] f[|5; 6|] };
    let c = ml! { v[i] <- v[i] + 1; xs[1..]; xs[..n]; xs[..] };
}

fn method_sections() {
    let a = ml! { xs |> .iter() |> .map(fun x -> x * 2) |> .filter(fun x -> !x > 2) |> .sum::<i64>() };
    let b = ml! { .push(1) xs; .len () xs; .len (s); (.len)(x); .collect::<Vec<_>>() it };
    let c = ml! { let g a b ~f = a in g (.map(f)) (.iter()) ~f: .len() };
    let e = ml! { let f ~k a b = k in pairs |> .iter() |> .map(.0) |> .sum::<i64>() + (x |> .1) + .0.1 t + f ~k: .0 (! (.1)) [| .0 |] };
    let d = ml! {
        items
        |> .into_iter()
        |> .map(fun item -> let doubled = item * 2 in let tripled = item * 3 in doubled + tripled)
        |> .fold(0, fun acc x -> acc + x)
    };
}

fn loops() {
    let a = ml! {
        for (k, v) in map do println!("{} {}", k, v) done;
        for k, v in pairs do f k v done;
        for x in xs.iter().filter(fun x -> !x > 0) do total <- total + !x; count <- count + 1 done;
        while true do if done_yet () then break; step () done
    };
    let b = ml! {
        for i in 0..n do
          (* comment in the body *)
          if i mod 2 = 0 then continue;
          out.push(i)
        done
    };
}

fn ranges() {
    let a = ml! { (0..n, a..=b, a.., ..b, .., ..=b, 0..n + 1, a * 2..b, 0..-1, x? .., .. -1) };
    let b = ml! { 0..n |> .map(f) |> .collect::<Vec<_>>() };
    let c = ml! { [| ..b |] };
    let d = ml! {
        match (c, n) with
        | ('a' .. 'z', 1..=3) -> 1
        | ('A'..'Z', -5..=-1) -> 2
        | _ -> 0
    };
}

fn derefs() {
    let a = ml! { *self_ + **q + !*p + &*s + *x.len + !r + (*x) + f ( *a) + g(*a, b) + v.(*i) };
    let b = ml! { let f ~k ~i = k in .m(*x) y + - *x + [| *x |] + f ~k: !r ~i: *r; *p <- *p + 1; a..*b };
}

fn jumps_and_blocks() {
    let a = ml! { if n < 0 then return 0; n * 2 };
    let b = ml! { match x with A -> return 1 | B -> break 2 | C -> continue | D -> return };
    let c = ml! { unsafe { f x } + unsafe {} + unsafe { let y = 1 in y } };
    let d = ml! { async { fetch url } |> spawn; async move { let body = fetch url in parse body } };
    let e = ml! {
        unsafe {
          (* before *)
          let long_name_number_one = compute_something_expensive first_argument in
          long_name_number_one + another_function_call second_argument third_argument
          (* last *)
        }
    };
    let f = ml! { return (a, b); return - 1; x + return 1; fun x -> return x };
}

rsml! {
    fn first_negative(xs: &[i32]) -> Option<usize> {
        for (i, x) in xs.iter().enumerate() do if !x < 0 then return Some(i) done;
        None
    }
}
