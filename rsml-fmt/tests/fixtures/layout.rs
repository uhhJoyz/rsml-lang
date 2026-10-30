// Formatter stress input for the layout of blocks, macro arguments and
// comments: only tokenized and expanded by tests/corpus.rs, never compiled.

fn begin_end() {
    let a = ml! {
        if ready () then begin start (); log "started"; count := !count + 1 end else begin wait (); retry () end
    };
    let b = ml! {
        match event with
        | Click (x, y) -> begin handle_click x y; redraw () end
        | Key k when k = 'q' -> begin save (); quit () end
        | _ -> ()
    };
    let c = ml! { List.iter (fun item -> begin validate item; store item; notify_all subscribers item end) items };
    let d = ml! {
        for i = 0 to n do begin step i; check_invariants state i end done;
        while not (done_yet ()) do begin poll (); sleep_for_a_while milliseconds end done;
        for x in xs do begin push x; flush_everything_now output_buffer end done
    };
    let e = ml! {
        if c then (* the slow path *) begin first_step (); second_step (); third_step () end
        else begin (* nothing to do *) () end
    };
}

fn rust_blocks() {
    let a = ml! {
        if checked then unsafe { read_volatile pointer_to_the_register; fence (); write_back () } else async move { fetch url; parse body; store record }
    };
    let b = ml! { match x with A -> unsafe { danger_one (); danger_two () } | B -> async { later (); much_later () } };
}

fn std_macros() {
    let a = ml! { println!("{} {x}",   f   x,x = xs |> .len); eprintln!("{:?}", (* why *) state) };
    let b = ml! {
        assert_eq!(compute_the_total_of_everything first_argument second_argument, expected_total_value, "totals differ for {}", name)
    };
    let c = ml! { vec![0;n+1] @ vec![ f 1 , g 2 , ] @ std::vec![fun x -> x] };
    let d = ml! {
        assert!(xs |> .iter() |> .all(fun x -> !x > 0 && check_each_one_thoroughly x), "all {} positive", "items")
    };
    let e = ml! { write!(out, "{}", f x)?; todo!(); println!{"{}",   x}; matches!(x,Some(1)|None) };
}
