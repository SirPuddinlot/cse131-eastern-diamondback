mod infra;

// Your tests go here!
success_tests! {
    test_input: { file: "input", input: "2", expected: "2" },
    test_input_tc: { file: "input", input: "false", expected: "false", typecheck: true },

}

runtime_error_tests! {
    test_overflow_error: { file: "overflow", expected: "overflow" },
}

static_error_tests! {
    test_parse_error: { file: "parse", input: "2", expected: "Invalid" },
}


repl_tests! {
    test_simple_bools: {commands:["(define x true)", "x", "false"], expected: ["true", "false"]},
    test_define_and_use: { commands: ["(define a 10)", "(define b (+ a 5))", "(+ a b)"], expected: ["25"], typecheck: true },
    repl_complicated_tc: { commands: [
        "(define acc 0)",
        "(fun (even (x : Num)) -> Num (set! acc (add1 acc)))",
        "(fun (process (n : Num)) -> Num (let ((i 1))  (loop  (if (> i n) (break acc) (block (even i) (set! i (+ i 1)))))))",
        "(process 7)",
        "(process 14)",
        "(even 15)",
        "(block (print 3) (+ 1 true))",
        "acc"
    ], expected: ["7", "21", "22", "22"], typecheck: true},

}
