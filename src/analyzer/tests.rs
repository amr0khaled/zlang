use crate::*;

fn run(line: &str) {
    let mut parser = Parser::new(line);
    let mut node = Box::new(parser.parse());

    let mut analyzer = Analyzer::new();
    analyzer.scope_check(&mut node);
}
mod variables {
    use super::*;

    // ===== LET =====
    #[test]
    fn test_let_no_type_no_value() {
        // let x is valid, type and value are optional
        let line = "let x;";
        run(line);
    }
    #[test]
    fn test_let_with_type() {
        let line = "let x: i32;";
        run(line);
    }
    #[test]
    fn test_let_with_value() {
        let line = "let x = 5;";
        run(line);
    }
    #[test]
    fn test_let_with_type_and_value() {
        let line = "let x: i32 = 5;";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_let_type_mismatch() {
        // declared as i32 but assigned a bool
        let line = "let x: i32 = true;";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_let_undeclared_value() {
        // y is not declared
        let line = "let x = y;";
        run(line);
    }
    #[test]
    fn test_let_use_after_declare() {
        let line = "let x = 5; let y = x;";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_let_use_before_declare() {
        let line = "let y = x; let x = 5;";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_let_shadow() {
        // same name declared twice in same scope
        let line = "let x = 5; let x = 10;";
        run(line);
    }

    // ===== CONST =====
    #[test]
    #[should_panic]
    fn test_const_with_value() {
        let line = "const x = 5;";
        run(line);
    }
    #[test]
    fn test_const_with_type_and_value() {
        let line = "const x: i32 = 5;";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_const_without_value() {
        // const requires a value
        let line = "const x: i32;";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_const_type_mismatch() {
        let line = "const x: i32 = true;";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_const_reassign() {
        // const cannot be reassigned
        let line = "const x = 5; x = 10;";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_const_compound_assign() {
        let line = "const x = 5; x += 1;";
        run(line);
    }
}

mod scopes {
    use super::*;

    #[test]
    fn test_inner_scope_sees_outer() {
        let line = "let x = 5; { let y = x; }";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_outer_scope_cannot_see_inner() {
        let line = "{ let x = 5; } let y = x;";
        run(line);
    }
    #[test]
    fn test_shadow_in_inner_scope() {
        // x in inner scope shadows outer x
        let line = "let x = 5; { let x = true; }";
        run(line);
    }
    #[test]
    fn test_inner_scope_does_not_pollute_outer() {
        // after inner scope, outer x is still i32
        let line = "let x = 5; { let x = true; } let y = x;";
        run(line);
    }
}

mod functions {
    use super::*;

    #[test]
    fn test_void_function() {
        let line = "fn name() {}";
        run(line);
    }
    #[test]
    fn test_function_with_return_type() {
        let line = "fn name: i32() { return 5; }";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_function_missing_return() {
        // declared return type but no return statement
        let line = "fn name: i32() { let x = 5; }";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_function_wrong_return_type() {
        let line = "fn name: i32() { return true; }";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_void_function_returns_value() {
        let line = "fn name() { return 5; }";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_function_bare_return() {
        // bare return is valid in a void function
        let line = "fn name() { return; }";
        run(line);
    }
    #[test]
    fn test_function_params() {
        let line = "fn name(x: i32, y: i32) { let z = x; }";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_function_param_undeclared_outside() {
        // params are not visible outside the function
        let line = "fn name(x: i32) {} let y = x;";
        run(line);
    }
    #[test]
    fn test_recursive_function() {
        let line = "fn name: i32() { return name(); }";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_call_undeclared_function() {
        let line = "name()";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_call_non_function() {
        let line = "let x = 5; x();";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_call_wrong_arg_count() {
        let line = "fn name(x: i32) {} name(1, 2);";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_call_wrong_arg_type() {
        let line = "fn name(x: i32) {} name(true);";
        run(line);
    }
    #[test]
    fn test_call_correct_args() {
        let line = "fn name(x: i32, y: i32) {} name(1, 2);";
        run(line);
    }
    #[test]
    fn test_compile_time_params() {
        let line = "fn name[a: i32](b: i32) {} name[1](2);";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_compile_time_wrong_count() {
        let line = "fn name[a: i32](b: i32) {} name[1, 2](2);";
        run(line);
    }
}

mod types {
    use super::*;

    #[test]
    #[should_panic]
    fn test_add_bool_and_int() {
        let line = "let x = true + 5;";
        run(line);
    }
    #[test]
    fn test_add_two_ints() {
        let line = "let x = 5 + 4;";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_assign_wrong_type() {
        let line = "let x: i32 = 5; x = true;";
        run(line);
    }
    #[test]
    fn test_bool_expression() {
        let line = "let x = 5 == 4;";
        run(line);
    }
    #[test]
    fn test_comparison_produces_bool() {
        let line = "let x: bool = 5 < 10;";
        run(line);
    }
    #[test]
    #[should_panic]
    fn test_logical_on_non_bool() {
        // && requires bool operands
        let line = "let x = 5 && 4;";
        run(line);
    }
}
mod operations {
    use super::*;
    use crate::parser::ops::Operator;

    const LOGICAL_OPS: &[(&str, Operator)] = &[("&&", Operator::And), ("||", Operator::Or)];
    const BINARY_OPS: &[(&str, Operator)] = &[
        ("+", Operator::Add),
        ("-", Operator::Sub),
        ("*", Operator::Mul),
        ("**", Operator::Power),
        ("/", Operator::Div),
        ("&", Operator::BinaryAnd),
        ("^", Operator::BinaryXor),
        ("%", Operator::Mod),
        ("==", Operator::Equality),
        ("!=", Operator::NotEquality),
        ("<", Operator::LesserThan),
        ("<=", Operator::LesserThanOrEqual),
        (">", Operator::GreaterThan),
        (">=", Operator::GreaterThanOrEqual),
    ];
    const UNARY_OPS: &[(&str, Operator)] = &[
        ("!", Operator::Not),
        ("++", Operator::PreIncrement),
        ("++", Operator::PostIncrement),
        ("--", Operator::PreDecrement),
        ("--", Operator::PostDecrement),
    ];
    const ASSIGN_OPS: &[(&str, Operator)] = &[
        // ("<-", Operator::AssignNew),
        ("+=", Operator::AssignPlus),
        ("-=", Operator::AssignMinus),
        ("*=", Operator::AssignMultiply),
        ("**=", Operator::AssignPower),
        ("/=", Operator::AssignDivide),
        ("%=", Operator::AssignModolus),
        ("&&=", Operator::AssignAnd),
        ("||=", Operator::AssignOr),
        ("&=", Operator::AssignBinaryAnd),
        ("|=", Operator::AssignBinaryOr),
        ("^=", Operator::AssignBinaryXor),
    ];
    #[test]
    fn test_logical_expressions_without_ident() {
        for &(sym, _) in LOGICAL_OPS {
            println!("TEST ({})", sym);
            let line = format!("false {} true", sym);
            run(&line);
        }
    }
    #[test]
    fn test_logical_expressions() {
        for &(sym, _) in LOGICAL_OPS {
            let line = format!(
                "let a = true;
let b = false;
a {} b;",
                sym
            );
            run(&line);
        }
    }
    #[test]
    fn test_binary_expressions_without_ident() {
        for &(sym, _) in BINARY_OPS {
            println!("TEST ({})", sym);
            let line = format!("5 {} 4", sym);
            run(&line);
        }
    }
    #[test]
    fn test_binary_expressions() {
        for &(sym, _) in BINARY_OPS {
            let line = format!(
                "let a = 4;
let b = 10;
a {} b;",
                sym
            );
            run(&line);
        }
    }
    #[test]
    fn test_unary_expresssions() {
        for &(sym, ref op) in UNARY_OPS {
            let pre = {
                if op == &Operator::PostIncrement || op == &Operator::PostDecrement {
                    false
                } else {
                    true
                }
            };
            let line = format!(
                "let a = 4;
{}",
                if pre {
                    format!("{}a", sym)
                } else {
                    println!("POST {}", sym);
                    format!("a{}", sym)
                }
            );
            run(&line);
        }
    }
    #[test]
    fn test_assign_expressions() {
        for &(sym, _) in ASSIGN_OPS {
            let line = format!(
                "let a = 4;
a {} 4;",
                sym
            );
            run(&line);
        }
    }
    #[test]
    #[should_panic]
    fn test_assign_new_expression() {
        let line = "let a = 4;
a <- 4;";
        run(line);
    }
}
