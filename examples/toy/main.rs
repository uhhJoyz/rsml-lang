//! The toy examples: one small, commented file per notation, in reading
//! order (README.md here has the index).
//!
//! ```sh
//! cargo run --example toy                 # every toy, in order
//! cargo run --example toy -- 42 pipe      # toy 042, and those named *pipe*
//! cargo run --example toy -- --list       # the number, name and summary of each
//! cargo test --example toy                # their tests
//! ```

#[path = "063_append.rs"]
mod append;
#[path = "001_application.rs"]
mod application;
#[path = "026_apply_operator.rs"]
mod apply_operator;
#[path = "097_arrays.rs"]
mod arrays;
#[path = "123_as_casts.rs"]
mod as_casts;
#[path = "073_as_patterns.rs"]
mod as_patterns;
#[path = "106_assert_macros.rs"]
mod assert_macros;
#[path = "087_assertions.rs"]
mod assertions;
#[path = "003_associated_functions.rs"]
mod associated_functions;
#[path = "117_async_and_const_fn.rs"]
mod async_and_const_fn;
#[path = "120_await_futures.rs"]
mod await_futures;
#[path = "092_bang_dereference.rs"]
mod bang_dereference;
#[path = "080_begin_end.rs"]
mod begin_end;
#[path = "004_borrows_and_labels.rs"]
mod borrows_and_labels;
#[path = "023_closure_typed_parameters.rs"]
mod closure_typed_parameters;
#[path = "067_comparisons_and_not.rs"]
mod comparisons_and_not;
#[path = "031_compose_applied.rs"]
mod compose_applied;
#[path = "030_compose_backward.rs"]
mod compose_backward;
#[path = "029_compose_forward.rs"]
mod compose_forward;
#[path = "032_compose_methods.rs"]
mod compose_methods;
#[path = "062_cons.rs"]
mod cons;
#[path = "057_constructor_values.rs"]
mod constructor_values;
#[path = "056_constructors.rs"]
mod constructors;
#[path = "037_custom_operator_rules.rs"]
mod custom_operator_rules;
#[path = "036_custom_operators.rs"]
mod custom_operators;
#[path = "091_deref_star.rs"]
mod deref_star;
#[path = "059_dotted_paths.rs"]
mod dotted_paths;
#[path = "137_expression_evaluator.rs"]
mod expression_evaluator;
#[path = "020_external.rs"]
mod external;
#[path = "021_external_paths.rs"]
mod external_paths;
#[path = "049_field_sections.rs"]
mod field_sections;
#[path = "131_fizzbuzz.rs"]
mod fizzbuzz;
#[path = "065_float_operators.rs"]
mod float_operators;
#[path = "082_for_downto.rs"]
mod for_downto;
#[path = "083_for_in.rs"]
mod for_in;
#[path = "081_for_to.rs"]
mod for_to;
#[path = "105_format_named_arguments.rs"]
mod format_named_arguments;
#[path = "012_fun.rs"]
mod fun;
#[path = "014_fun_applied.rs"]
mod fun_applied;
#[path = "015_fun_move.rs"]
mod fun_move;
#[path = "013_fun_parameters.rs"]
mod fun_parameters;
#[path = "016_function.rs"]
mod function;
#[path = "100_function_and_tuple_types.rs"]
mod function_and_tuple_types;
#[path = "017_function_in_pipelines.rs"]
mod function_in_pipelines;
#[path = "050_glued_pipe_dot.rs"]
mod glued_pipe_dot;
#[path = "074_guards.rs"]
mod guards;
#[path = "078_if_then_else.rs"]
mod if_then_else;
#[path = "055_indexing.rs"]
mod indexing;
#[path = "066_integer_operators.rs"]
mod integer_operators;
#[path = "035_keyword_operator_sections.rs"]
mod keyword_operator_sections;
#[path = "054_known_arity_calls.rs"]
mod known_arity_calls;
#[path = "006_let_in.rs"]
mod let_in;
#[path = "090_let_mut.rs"]
mod let_mut;
#[path = "010_let_mut_function.rs"]
mod let_mut_function;
#[path = "008_let_rec.rs"]
mod let_rec;
#[path = "009_let_rec_and.rs"]
mod let_rec_and;
#[path = "102_lifetimes.rs"]
mod lifetimes;
#[path = "070_list_patterns.rs"]
mod list_patterns;
#[path = "096_lists.rs"]
mod lists;
#[path = "077_literal_patterns.rs"]
mod literal_patterns;
#[path = "007_local_functions.rs"]
mod local_functions;
#[path = "085_loop_break_value.rs"]
mod loop_break_value;
#[path = "068_match_variants.rs"]
mod match_variants;
#[path = "138_matrix.rs"]
mod matrix;
#[path = "046_method_pipelines.rs"]
mod method_pipelines;
#[path = "045_method_refs.rs"]
mod method_refs;
#[path = "048_method_sections.rs"]
mod method_sections;
#[path = "047_methods_as_values.rs"]
mod methods_as_values;
#[path = "116_nested_modules.rs"]
mod nested_modules;
#[path = "039_numbered_placeholders.rs"]
mod numbered_placeholders;
#[path = "127_ocaml_comments.rs"]
mod ocaml_comments;
#[path = "033_operator_sections.rs"]
mod operator_sections;
#[path = "072_or_patterns.rs"]
mod or_patterns;
#[path = "002_parenthesized_arguments.rs"]
mod parenthesized_arguments;
#[path = "018_partial_application.rs"]
mod partial_application;
#[path = "019_partial_evaluated_once.rs"]
mod partial_evaluated_once;
#[path = "024_pipe.rs"]
mod pipe;
#[path = "028_pipe_into_rust_call.rs"]
mod pipe_into_rust_call;
#[path = "027_pipe_unit.rs"]
mod pipe_unit;
#[path = "025_pipe_value_last.rs"]
mod pipe_value_last;
#[path = "038_placeholder.rs"]
mod placeholder;
#[path = "040_placeholder_reach.rs"]
mod placeholder_reach;
#[path = "043_placeholders_applied.rs"]
mod placeholders_applied;
#[path = "044_placeholders_evaluate_once.rs"]
mod placeholders_evaluate_once;
#[path = "041_placeholders_in_arguments.rs"]
mod placeholders_in_arguments;
#[path = "139_prime_sieve.rs"]
mod prime_sieve;
#[path = "104_println.rs"]
mod println;
#[path = "119_question_mark.rs"]
mod question_mark;
#[path = "075_range_patterns.rs"]
mod range_patterns;
#[path = "088_ranges.rs"]
mod ranges;
#[path = "122_raw_identifiers.rs"]
mod raw_identifiers;
#[path = "051_receiver_and_unit.rs"]
mod receiver_and_unit;
#[path = "071_record_patterns.rs"]
mod record_patterns;
#[path = "094_record_punning.rs"]
mod record_punning;
#[path = "095_record_update.rs"]
mod record_update;
#[path = "093_records.rs"]
mod records;
#[path = "076_reference_patterns.rs"]
mod reference_patterns;
#[path = "089_refs.rs"]
mod refs;
#[path = "086_return_break_continue.rs"]
mod return_break_continue;
#[path = "113_rsml_closure_parameters.rs"]
mod rsml_closure_parameters;
#[path = "109_rsml_functions.rs"]
mod rsml_functions;
#[path = "112_rsml_generics_where.rs"]
mod rsml_generics_where;
#[path = "110_rsml_impl.rs"]
mod rsml_impl;
#[path = "111_rsml_trait.rs"]
mod rsml_trait;
#[path = "128_rsmlfmt_directives.rs"]
mod rsmlfmt_directives;
#[path = "136_run_length_encoding.rs"]
mod run_length_encoding;
#[path = "118_rust_attribute.rs"]
mod rust_attribute;
#[path = "053_rust_calls.rs"]
mod rust_calls;
#[path = "126_rust_comments.rs"]
mod rust_comments;
#[path = "130_rust_habits_blocks.rs"]
mod rust_habits_blocks;
#[path = "129_rust_habits_expressions.rs"]
mod rust_habits_expressions;
#[path = "052_rust_method_calls.rs"]
mod rust_method_calls;
#[path = "058_rust_paths.rs"]
mod rust_paths;
#[path = "140_sales_report.rs"]
mod sales_report;
#[path = "034_section_partial.rs"]
mod section_partial;
#[path = "115_self_and_type_names.rs"]
mod self_and_type_names;
#[path = "079_sequences.rs"]
mod sequences;
#[path = "114_sibling_arity.rs"]
mod sibling_arity;
#[path = "135_state_machine.rs"]
mod state_machine;
#[path = "064_string_concat.rs"]
mod string_concat;
#[path = "134_tokenizer.rs"]
mod tokenizer;
#[path = "011_toplevel_phrases.rs"]
mod toplevel_phrases;
#[path = "133_tree_fold.rs"]
mod tree_fold;
#[path = "069_tuple_patterns.rs"]
mod tuple_patterns;
#[path = "098_tuples.rs"]
mod tuples;
#[path = "103_turbofish.rs"]
mod turbofish;
#[path = "099_type_annotations.rs"]
mod type_annotations;
#[path = "022_type_constraint_arity.rs"]
mod type_constraint_arity;
#[path = "101_type_variables.rs"]
mod type_variables;
#[path = "042_typed_placeholders.rs"]
mod typed_placeholders;
#[path = "005_unit_rules.rs"]
mod unit_rules;
#[path = "121_unsafe_blocks.rs"]
mod unsafe_blocks;
#[path = "061_use_in_expressions.rs"]
mod use_in_expressions;
#[path = "060_use_items.rs"]
mod use_items;
#[path = "107_vec_macro.rs"]
mod vec_macro;
#[path = "084_while_loops.rs"]
mod while_loops;
#[path = "124_whole_file.rs"]
mod whole_file;
#[path = "125_whole_file_tests.rs"]
mod whole_file_tests;
#[path = "132_word_count.rs"]
mod word_count;
#[path = "108_write_macro.rs"]
mod write_macro;

/// Every toy: its file name (without `.rs`), its summary and its demo.
const TOYS: &[(&str, &str, fn())] = &[
    ("001_application", application::SUMMARY, application::run),
    (
        "002_parenthesized_arguments",
        parenthesized_arguments::SUMMARY,
        parenthesized_arguments::run,
    ),
    (
        "003_associated_functions",
        associated_functions::SUMMARY,
        associated_functions::run,
    ),
    (
        "004_borrows_and_labels",
        borrows_and_labels::SUMMARY,
        borrows_and_labels::run,
    ),
    ("005_unit_rules", unit_rules::SUMMARY, unit_rules::run),
    ("006_let_in", let_in::SUMMARY, let_in::run),
    (
        "007_local_functions",
        local_functions::SUMMARY,
        local_functions::run,
    ),
    ("008_let_rec", let_rec::SUMMARY, let_rec::run),
    ("009_let_rec_and", let_rec_and::SUMMARY, let_rec_and::run),
    (
        "010_let_mut_function",
        let_mut_function::SUMMARY,
        let_mut_function::run,
    ),
    (
        "011_toplevel_phrases",
        toplevel_phrases::SUMMARY,
        toplevel_phrases::run,
    ),
    ("012_fun", fun::SUMMARY, fun::run),
    (
        "013_fun_parameters",
        fun_parameters::SUMMARY,
        fun_parameters::run,
    ),
    ("014_fun_applied", fun_applied::SUMMARY, fun_applied::run),
    ("015_fun_move", fun_move::SUMMARY, fun_move::run),
    ("016_function", function::SUMMARY, function::run),
    (
        "017_function_in_pipelines",
        function_in_pipelines::SUMMARY,
        function_in_pipelines::run,
    ),
    (
        "018_partial_application",
        partial_application::SUMMARY,
        partial_application::run,
    ),
    (
        "019_partial_evaluated_once",
        partial_evaluated_once::SUMMARY,
        partial_evaluated_once::run,
    ),
    ("020_external", external::SUMMARY, external::run),
    (
        "021_external_paths",
        external_paths::SUMMARY,
        external_paths::run,
    ),
    (
        "022_type_constraint_arity",
        type_constraint_arity::SUMMARY,
        type_constraint_arity::run,
    ),
    (
        "023_closure_typed_parameters",
        closure_typed_parameters::SUMMARY,
        closure_typed_parameters::run,
    ),
    ("024_pipe", pipe::SUMMARY, pipe::run),
    (
        "025_pipe_value_last",
        pipe_value_last::SUMMARY,
        pipe_value_last::run,
    ),
    (
        "026_apply_operator",
        apply_operator::SUMMARY,
        apply_operator::run,
    ),
    ("027_pipe_unit", pipe_unit::SUMMARY, pipe_unit::run),
    (
        "028_pipe_into_rust_call",
        pipe_into_rust_call::SUMMARY,
        pipe_into_rust_call::run,
    ),
    (
        "029_compose_forward",
        compose_forward::SUMMARY,
        compose_forward::run,
    ),
    (
        "030_compose_backward",
        compose_backward::SUMMARY,
        compose_backward::run,
    ),
    (
        "031_compose_applied",
        compose_applied::SUMMARY,
        compose_applied::run,
    ),
    (
        "032_compose_methods",
        compose_methods::SUMMARY,
        compose_methods::run,
    ),
    (
        "033_operator_sections",
        operator_sections::SUMMARY,
        operator_sections::run,
    ),
    (
        "034_section_partial",
        section_partial::SUMMARY,
        section_partial::run,
    ),
    (
        "035_keyword_operator_sections",
        keyword_operator_sections::SUMMARY,
        keyword_operator_sections::run,
    ),
    (
        "036_custom_operators",
        custom_operators::SUMMARY,
        custom_operators::run,
    ),
    (
        "037_custom_operator_rules",
        custom_operator_rules::SUMMARY,
        custom_operator_rules::run,
    ),
    ("038_placeholder", placeholder::SUMMARY, placeholder::run),
    (
        "039_numbered_placeholders",
        numbered_placeholders::SUMMARY,
        numbered_placeholders::run,
    ),
    (
        "040_placeholder_reach",
        placeholder_reach::SUMMARY,
        placeholder_reach::run,
    ),
    (
        "041_placeholders_in_arguments",
        placeholders_in_arguments::SUMMARY,
        placeholders_in_arguments::run,
    ),
    (
        "042_typed_placeholders",
        typed_placeholders::SUMMARY,
        typed_placeholders::run,
    ),
    (
        "043_placeholders_applied",
        placeholders_applied::SUMMARY,
        placeholders_applied::run,
    ),
    (
        "044_placeholders_evaluate_once",
        placeholders_evaluate_once::SUMMARY,
        placeholders_evaluate_once::run,
    ),
    ("045_method_refs", method_refs::SUMMARY, method_refs::run),
    (
        "046_method_pipelines",
        method_pipelines::SUMMARY,
        method_pipelines::run,
    ),
    (
        "047_methods_as_values",
        methods_as_values::SUMMARY,
        methods_as_values::run,
    ),
    (
        "048_method_sections",
        method_sections::SUMMARY,
        method_sections::run,
    ),
    (
        "049_field_sections",
        field_sections::SUMMARY,
        field_sections::run,
    ),
    (
        "050_glued_pipe_dot",
        glued_pipe_dot::SUMMARY,
        glued_pipe_dot::run,
    ),
    (
        "051_receiver_and_unit",
        receiver_and_unit::SUMMARY,
        receiver_and_unit::run,
    ),
    (
        "052_rust_method_calls",
        rust_method_calls::SUMMARY,
        rust_method_calls::run,
    ),
    ("053_rust_calls", rust_calls::SUMMARY, rust_calls::run),
    (
        "054_known_arity_calls",
        known_arity_calls::SUMMARY,
        known_arity_calls::run,
    ),
    ("055_indexing", indexing::SUMMARY, indexing::run),
    ("056_constructors", constructors::SUMMARY, constructors::run),
    (
        "057_constructor_values",
        constructor_values::SUMMARY,
        constructor_values::run,
    ),
    ("058_rust_paths", rust_paths::SUMMARY, rust_paths::run),
    ("059_dotted_paths", dotted_paths::SUMMARY, dotted_paths::run),
    ("060_use_items", use_items::SUMMARY, use_items::run),
    (
        "061_use_in_expressions",
        use_in_expressions::SUMMARY,
        use_in_expressions::run,
    ),
    ("062_cons", cons::SUMMARY, cons::run),
    ("063_append", append::SUMMARY, append::run),
    (
        "064_string_concat",
        string_concat::SUMMARY,
        string_concat::run,
    ),
    (
        "065_float_operators",
        float_operators::SUMMARY,
        float_operators::run,
    ),
    (
        "066_integer_operators",
        integer_operators::SUMMARY,
        integer_operators::run,
    ),
    (
        "067_comparisons_and_not",
        comparisons_and_not::SUMMARY,
        comparisons_and_not::run,
    ),
    (
        "068_match_variants",
        match_variants::SUMMARY,
        match_variants::run,
    ),
    (
        "069_tuple_patterns",
        tuple_patterns::SUMMARY,
        tuple_patterns::run,
    ),
    (
        "070_list_patterns",
        list_patterns::SUMMARY,
        list_patterns::run,
    ),
    (
        "071_record_patterns",
        record_patterns::SUMMARY,
        record_patterns::run,
    ),
    ("072_or_patterns", or_patterns::SUMMARY, or_patterns::run),
    ("073_as_patterns", as_patterns::SUMMARY, as_patterns::run),
    ("074_guards", guards::SUMMARY, guards::run),
    (
        "075_range_patterns",
        range_patterns::SUMMARY,
        range_patterns::run,
    ),
    (
        "076_reference_patterns",
        reference_patterns::SUMMARY,
        reference_patterns::run,
    ),
    (
        "077_literal_patterns",
        literal_patterns::SUMMARY,
        literal_patterns::run,
    ),
    ("078_if_then_else", if_then_else::SUMMARY, if_then_else::run),
    ("079_sequences", sequences::SUMMARY, sequences::run),
    ("080_begin_end", begin_end::SUMMARY, begin_end::run),
    ("081_for_to", for_to::SUMMARY, for_to::run),
    ("082_for_downto", for_downto::SUMMARY, for_downto::run),
    ("083_for_in", for_in::SUMMARY, for_in::run),
    ("084_while_loops", while_loops::SUMMARY, while_loops::run),
    (
        "085_loop_break_value",
        loop_break_value::SUMMARY,
        loop_break_value::run,
    ),
    (
        "086_return_break_continue",
        return_break_continue::SUMMARY,
        return_break_continue::run,
    ),
    ("087_assertions", assertions::SUMMARY, assertions::run),
    ("088_ranges", ranges::SUMMARY, ranges::run),
    ("089_refs", refs::SUMMARY, refs::run),
    ("090_let_mut", let_mut::SUMMARY, let_mut::run),
    ("091_deref_star", deref_star::SUMMARY, deref_star::run),
    (
        "092_bang_dereference",
        bang_dereference::SUMMARY,
        bang_dereference::run,
    ),
    ("093_records", records::SUMMARY, records::run),
    (
        "094_record_punning",
        record_punning::SUMMARY,
        record_punning::run,
    ),
    (
        "095_record_update",
        record_update::SUMMARY,
        record_update::run,
    ),
    ("096_lists", lists::SUMMARY, lists::run),
    ("097_arrays", arrays::SUMMARY, arrays::run),
    ("098_tuples", tuples::SUMMARY, tuples::run),
    (
        "099_type_annotations",
        type_annotations::SUMMARY,
        type_annotations::run,
    ),
    (
        "100_function_and_tuple_types",
        function_and_tuple_types::SUMMARY,
        function_and_tuple_types::run,
    ),
    (
        "101_type_variables",
        type_variables::SUMMARY,
        type_variables::run,
    ),
    ("102_lifetimes", lifetimes::SUMMARY, lifetimes::run),
    ("103_turbofish", turbofish::SUMMARY, turbofish::run),
    ("104_println", println::SUMMARY, println::run),
    (
        "105_format_named_arguments",
        format_named_arguments::SUMMARY,
        format_named_arguments::run,
    ),
    (
        "106_assert_macros",
        assert_macros::SUMMARY,
        assert_macros::run,
    ),
    ("107_vec_macro", vec_macro::SUMMARY, vec_macro::run),
    ("108_write_macro", write_macro::SUMMARY, write_macro::run),
    (
        "109_rsml_functions",
        rsml_functions::SUMMARY,
        rsml_functions::run,
    ),
    ("110_rsml_impl", rsml_impl::SUMMARY, rsml_impl::run),
    ("111_rsml_trait", rsml_trait::SUMMARY, rsml_trait::run),
    (
        "112_rsml_generics_where",
        rsml_generics_where::SUMMARY,
        rsml_generics_where::run,
    ),
    (
        "113_rsml_closure_parameters",
        rsml_closure_parameters::SUMMARY,
        rsml_closure_parameters::run,
    ),
    (
        "114_sibling_arity",
        sibling_arity::SUMMARY,
        sibling_arity::run,
    ),
    (
        "115_self_and_type_names",
        self_and_type_names::SUMMARY,
        self_and_type_names::run,
    ),
    (
        "116_nested_modules",
        nested_modules::SUMMARY,
        nested_modules::run,
    ),
    (
        "117_async_and_const_fn",
        async_and_const_fn::SUMMARY,
        async_and_const_fn::run,
    ),
    (
        "118_rust_attribute",
        rust_attribute::SUMMARY,
        rust_attribute::run,
    ),
    (
        "119_question_mark",
        question_mark::SUMMARY,
        question_mark::run,
    ),
    (
        "120_await_futures",
        await_futures::SUMMARY,
        await_futures::run,
    ),
    (
        "121_unsafe_blocks",
        unsafe_blocks::SUMMARY,
        unsafe_blocks::run,
    ),
    (
        "122_raw_identifiers",
        raw_identifiers::SUMMARY,
        raw_identifiers::run,
    ),
    ("123_as_casts", as_casts::SUMMARY, as_casts::run),
    ("124_whole_file", whole_file::SUMMARY, whole_file::run),
    (
        "125_whole_file_tests",
        whole_file_tests::SUMMARY,
        whole_file_tests::run,
    ),
    (
        "126_rust_comments",
        rust_comments::SUMMARY,
        rust_comments::run,
    ),
    (
        "127_ocaml_comments",
        ocaml_comments::SUMMARY,
        ocaml_comments::run,
    ),
    (
        "128_rsmlfmt_directives",
        rsmlfmt_directives::SUMMARY,
        rsmlfmt_directives::run,
    ),
    (
        "129_rust_habits_expressions",
        rust_habits_expressions::SUMMARY,
        rust_habits_expressions::run,
    ),
    (
        "130_rust_habits_blocks",
        rust_habits_blocks::SUMMARY,
        rust_habits_blocks::run,
    ),
    ("131_fizzbuzz", fizzbuzz::SUMMARY, fizzbuzz::run),
    ("132_word_count", word_count::SUMMARY, word_count::run),
    ("133_tree_fold", tree_fold::SUMMARY, tree_fold::run),
    ("134_tokenizer", tokenizer::SUMMARY, tokenizer::run),
    (
        "135_state_machine",
        state_machine::SUMMARY,
        state_machine::run,
    ),
    (
        "136_run_length_encoding",
        run_length_encoding::SUMMARY,
        run_length_encoding::run,
    ),
    (
        "137_expression_evaluator",
        expression_evaluator::SUMMARY,
        expression_evaluator::run,
    ),
    ("138_matrix", matrix::SUMMARY, matrix::run),
    ("139_prime_sieve", prime_sieve::SUMMARY, prime_sieve::run),
    ("140_sales_report", sales_report::SUMMARY, sales_report::run),
];

/// Whether `arg` picks the toy `file`: by number (`42` or `042`) or by a
/// part of its name.
fn picks(arg: &str, file: &str) -> bool {
    let (number, name) = file.split_at(3);
    arg.parse::<usize>().ok() == number.parse().ok() || name[1..].contains(arg)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--list") {
        for (file, summary, _) in TOYS {
            let (number, name) = file.split_at(3);
            println!("{number}  {:<28} {summary}", &name[1..]);
        }
        return;
    }
    let chosen: Vec<_> = TOYS
        .iter()
        .filter(|(file, _, _)| args.is_empty() || args.iter().any(|a| picks(a, file)))
        .collect();
    if chosen.is_empty() {
        eprintln!("no toy matches {args:?}; `--list` lists them");
        std::process::exit(1);
    }
    for (file, summary, run) in &chosen {
        println!("== {file}: {summary}");
        run();
        println!();
    }
    println!("ran {} of {} toys", chosen.len(), TOYS.len());
}

#[cfg(test)]
mod tests {
    /// What `cargo run --example toy` does: every demo runs to its end.
    #[test]
    fn every_toy_runs() {
        for (_, _, run) in super::TOYS {
            run();
        }
    }
}
