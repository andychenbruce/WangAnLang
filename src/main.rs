mod ast;
mod interpreter;
mod interpreter_soft;
mod parser;
mod type_checker;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let filename = &args
        .get(1)
        .unwrap_or_else(|| panic!("usage: {} FILENAME [FUNC NAME] [ARGS ...]", args[0]));

    let program_parse_tree_result = parsel::parse_str(&std::fs::read_to_string(filename).unwrap());

    let program_parse_tree: parser::ProgramParseTree = match program_parse_tree_result {
        Ok(x) => x,
        Err(err) => panic!(
            "error parsing program at {:?}: {:?}",
            err.span().start(),
            err
        ),
    };

    let mut program_ast = ast::make_program(program_parse_tree);

    if let Some(func_name) = args.get(2) {
        let func_args: Vec<interpreter::Value> = args[3..]
            .iter()
            .map(|x| {
                if x.contains('.') {
                    interpreter::Value::Float(x.parse::<f64>().unwrap())
                } else {
                    interpreter::Value::Int(x.parse::<i64>().unwrap())
                }
            })
            .collect();

        let val = interpreter::run_function(&program_ast, func_name, func_args.clone());
        eprintln!("val = {:?}", val);

        let soft_args: Vec<interpreter_soft::ValueType> = func_args
            .iter()
            .map(|arg| match arg {
                interpreter::Value::Int(x) => interpreter_soft::ValueType::Int(*x as f64),
                interpreter::Value::Float(x) => interpreter_soft::ValueType::Float(*x),
                _ => unreachable!(),
            })
            .collect();

        let soft_val = interpreter_soft::soft_run_function(&program_ast, func_name, soft_args);

        eprintln!("soft val = {:?}", soft_val);
    } else {
        eprintln!(
            "test cases = {:?}",
            interpreter::eval_test_cases(&program_ast)
        );

        for _ in 0..5 {
            let soft_cases = interpreter_soft::soft_eval_test_cases(&program_ast);

            eprintln!("soft test cases = {:?}", soft_cases);

            let average_grad = soft_cases.into_iter().fold(
                interpreter_soft::make_oneshot(program_ast.num_ids, crate::ast::LitId(None)),
                |acc, new| acc + new.1,
            );

            interpreter_soft::apply_gradient_program(&mut program_ast, &average_grad);
        }

        eprintln!(
            "test cases fixed maybe = {:?}",
            interpreter::eval_test_cases(&program_ast)
        );
    }
}
