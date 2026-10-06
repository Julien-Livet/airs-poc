use crate::connection::{Connection, Terminals};
use crate::registry::{Type, Value};
use crate::types::Grid;
use crate::environment::InputEnvironment;
use crate::signature::InputSpec;

fn generate_apply(
    output_type: Type,
    depth: usize,
    terminals: &Terminals,
    inputs: &[InputSpec],
    function_types: &mut crate::registry::FunctionTypeRegistry,
) -> Vec<Connection> {
    let mut programs = Vec::new();

    let function_types_snapshot = function_types
        .all_types()
        .map(|(id, function_type)| {
            (id, function_type.clone())
        })
        .collect::<Vec<_>>();

    for (function_id, function_type) in function_types_snapshot {
        if function_type.output != output_type {
            continue;
        }

        let function = Type::Function(function_id);

        let functions = generate_up_to(
            function,
            depth - 1,
            terminals,
            inputs,
            function_types,
        );

        if functions.is_empty() {
            continue;
        }

        let mut argument_lists = Vec::new();
        let mut valid = true;

        for argument_type in &function_type.inputs {
            let arguments = generate_up_to(
                *argument_type,
                depth - 1,
                terminals,
                inputs,
                function_types,
            );

            if arguments.is_empty() {
                valid = false;
                break;
            }

            argument_lists.push(
                arguments
                    .into_iter()
                    .map(Box::new)
                    .collect::<Vec<_>>()
            );
        }

        if !valid {
            continue;
        }

        for function_program in functions {
            for arguments in cartesian_product(
                &argument_lists,
            ) {
                let arguments = arguments
                    .into_iter()
                    .map(|argument| *argument)
                    .collect::<Vec<_>>();

                if let Ok(connection) = Connection::apply(
                    function_program.clone(),
                    arguments,
                    function_types,
                ) {
                    programs.push(connection);
                }
            }
        }
    }

    programs
}

fn generate_rbind(
    output_type: Type,
    depth: usize,
    terminals: &Terminals,
    inputs: &[InputSpec],
    function_types: &mut crate::registry::FunctionTypeRegistry,
) -> Vec<Connection> {
    let mut programs = Vec::new();

    let Type::Function(output_id) = output_type else {
        return programs;
    };

    let Some(output_inputs) =
        function_types.inputs(output_id)
    else {
        return programs;
    };

    let Some(output_output) =
        function_types.output(output_id)
    else {
        return programs;
    };

    let candidates = function_types
        .all_types()
        .filter_map(|(source_id, source_type)| {
            if source_type.inputs.len()
                != output_inputs.len() + 1
            {
                return None;
            }

            let last = source_type.inputs.len() - 1;

            if source_type.inputs[..last]
                != *output_inputs
            {
                return None;
            }

            if source_type.output != output_output {
                return None;
            }

            Some((
                Type::Function(source_id),
                source_type.inputs[last],
            ))
        })
        .collect::<Vec<_>>();

    for (source_type, fixed_type) in candidates {
        let functions = generate_bounded(
            source_type,
            depth - 1,
            terminals,
            inputs,
            function_types,
        );

        let fixed_values = generate_bounded(
            fixed_type,
            depth - 1,
            terminals,
            inputs,
            function_types,
        );

        for function in functions {
            for fixed in &fixed_values {
                if let Ok(connection) = Connection::rbind(
                    function.clone(),
                    fixed.clone(),
                    function_types,
                ) {
                    programs.push(connection);
                }
            }
        }
    }

    programs
}

fn generate_lbind(
    output_type: Type,
    depth: usize,
    terminals: &Terminals,
    inputs: &[InputSpec],
    function_types: &mut crate::registry::FunctionTypeRegistry,
) -> Vec<Connection> {
    let mut programs = Vec::new();

    let Type::Function(output_id) = output_type else {
        return programs;
    };

    let Some(output_inputs) =
        function_types.inputs(output_id)
    else {
        return programs;
    };

    let Some(output_output) =
        function_types.output(output_id)
    else {
        return programs;
    };

    let candidates = function_types
        .all_types()
        .filter_map(|(source_id, source_type)| {
            if source_type.inputs.len()
                != output_inputs.len() + 1
            {
                return None;
            }

            if source_type.inputs[1..]
                != *output_inputs
            {
                return None;
            }

            if source_type.output != output_output {
                return None;
            }

            Some((
                Type::Function(source_id),
                source_type.inputs[0],
            ))
        })
        .collect::<Vec<_>>();

    for (source_type, fixed_type) in candidates {
        let functions = generate_bounded(
            source_type,
            depth - 1,
            terminals,
            inputs,
            function_types,
        );

        let fixed_values = generate_bounded(
            fixed_type,
            depth - 1,
            terminals,
            inputs,
            function_types,
        );

        for function in functions {
            for fixed in &fixed_values {
                if let Ok(connection) = Connection::lbind(
                    function.clone(),
                    fixed.clone(),
                    function_types,
                ) {
                    programs.push(connection);
                }
            }
        }
    }

    programs
}

pub fn generate_inputs(
    output_type: Type,
    inputs: &[InputSpec],
) -> Vec<Connection> {
    inputs
        .iter()
        .filter(|input| input.accepts(output_type))
        .map(InputSpec::connection)
        .collect()
}

fn cartesian_product(
    lists: &[Vec<Box<Connection>>],
) -> Vec<Vec<Box<Connection>>> {
    if lists.is_empty() {
        return vec![Vec::new()];
    }

    let mut result = Vec::new();

    for input in &lists[0] {
        for mut combination in cartesian_product(&lists[1..]) {
            let mut current = vec![
                Box::new(input.as_ref().clone())
            ];

            current.append(&mut combination);
            result.push(current);
        }
    }

    result
}

pub fn generate_up_to(
    output_type: Type,
    depth: usize,
    terminals: &Terminals,
    inputs: &[InputSpec],
    function_types: &mut crate::registry::FunctionTypeRegistry,
) -> Vec<Connection> {
    generate_bounded(
        output_type,
        depth,
        terminals,
        inputs,
        function_types,
    )
}

pub fn generate(
    output_type: Type,
    depth: usize,
    terminals: &Terminals,
    inputs: &[InputSpec],
    function_types: &mut crate::registry::FunctionTypeRegistry,
) -> Vec<Connection> {
    let mut programs = Vec::new();

    for terminal in &terminals.values {
        terminal.value.output_type_with(function_types);
    }

    if depth == 0 {
        for terminal in &terminals.values {
            if terminal.value.output_type_with(function_types) == output_type {
                programs.push(
                    Connection::named_terminal(
                        terminal.name,
                        terminal.value.clone(),
                    )
                );
            }
        }

        programs.extend(
            generate_inputs(
                output_type,
                inputs,
            )
        );

        return programs;
    }

    for primitive in crate::registry::PRIMITIVES {
        if primitive.output != output_type {
            continue;
        }

        let mut input_lists: Vec<Vec<Box<Connection>>> = Vec::new();
        let mut valid = true;

        for input_type in primitive.inputs {
            let subprograms = generate(
                *input_type,
                depth - 1,
                terminals,
                inputs,
                function_types,
            );

            if subprograms.is_empty() {
                valid = false;
                break;
            }

            let inputs = subprograms
                .into_iter()
                .map(Box::new)
                .collect();

            input_lists.push(inputs);
        }

        if !valid {
            continue;
        }

        for inputs in cartesian_product(&input_lists) {
            if let Ok(connection) = Connection::new(
                primitive,
                inputs,
            ) {
                programs.push(connection);
            }
        }
    }

    programs.extend(
        generate_lbind(
            output_type,
            depth,
            terminals,
            inputs,
            function_types,
        )
    );

    programs.extend(
        generate_rbind(
            output_type,
            depth,
            terminals,
            inputs,
            function_types,
        )
    );

    programs.extend(
        generate_apply(
            output_type,
            depth,
            terminals,
            inputs,
            function_types,
        )
    );

    programs
}

fn generate_bounded(
    output_type: Type,
    depth: usize,
    terminals: &Terminals,
    inputs: &[InputSpec],
    function_types: &mut crate::registry::FunctionTypeRegistry,
) -> Vec<Connection> {
    let mut programs = Vec::new();

    if depth == 0 {
        return generate(
            output_type,
            0,
            terminals,
            inputs,
            function_types,
        );
    }

    programs.extend(
        generate_bounded(
            output_type,
            depth - 1,
            terminals,
            inputs,
            function_types,
        )
    );

    for primitive in crate::registry::PRIMITIVES {
        if primitive.output != output_type {
            continue;
        }

        let mut input_lists: Vec<Vec<Box<Connection>>> = Vec::new();
        let mut valid = true;

        for input_type in primitive.inputs {
            let subprograms = generate_bounded(
                *input_type,
                depth - 1,
                terminals,
                inputs,
                function_types,
            );

            if subprograms.is_empty() {
                valid = false;
                break;
            }

            input_lists.push(
                subprograms
                    .into_iter()
                    .map(Box::new)
                    .collect()
            );
        }

        if !valid {
            continue;
        }

        for inputs in cartesian_product(&input_lists) {
            if let Ok(connection) = Connection::new(
                primitive,
                inputs,
            ) {
                programs.push(connection);
            }
        }
    }

    programs.extend(
        generate_lbind(
            output_type,
            depth,
            terminals,
            inputs,
            function_types,
        )
    );

    programs.extend(
        generate_rbind(
            output_type,
            depth,
            terminals,
            inputs,
            function_types,
        )
    );

    programs
}

pub fn semantic_signature(
    program: &Connection,
    environments: &[InputEnvironment],
) -> Result<Vec<Grid>, String> {
    environments
        .iter()
        .map(|environment| {
            match program.output_with_inputs(environment)? {
                Value::Grid(result) => Ok(result),
                _ => Err("program does not produce a Grid".to_string()),
            }
        })
        .collect()
}


#[cfg(test)]
mod tests
{
    use super::*;
    use crate::NamedTerminal;
    use crate::function::Function;

    #[test]
    fn crop_can_be_generated_from_typed_inputs() {
        let terminals = Terminals {
            values: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
            InputSpec {
                name: "UL".to_string(),
                ty: Type::IntegerTuple,
            },
            InputSpec {
                name: "LR".to_string(),
                ty: Type::IntegerTuple,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::Grid,
            1,
            &terminals,
            &inputs,
            &mut function_types,
        );

        assert!(
            programs.iter().any(|program| {
                program.expression() == "crop(I, UL, LR)"
            })
        );

        let crop_programs = programs
            .iter()
            .filter(|program| {
                program.expression().starts_with("crop(")
            })
            .count();

        assert_eq!(crop_programs, 4);
    }

    #[test]
    fn integer_tuple_generation_uses_typed_inputs() {
        let terminals = Terminals {
            values: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
            InputSpec {
                name: "UL".to_string(),
                ty: Type::IntegerTuple,
            },
            InputSpec {
                name: "LR".to_string(),
                ty: Type::IntegerTuple,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::IntegerTuple,
            0,
            &terminals,
            &inputs,
            &mut function_types,
        );

        assert_eq!(programs.len(), 2);

        assert!(
            programs.iter().any(|program| {
                program.expression() == "UL"
            })
        );

        assert!(
            programs.iter().any(|program| {
                program.expression() == "LR"
            })
        );
    }

    #[test]
    fn corner_programs_can_be_generated_from_indices_input() {
        let terminals = Terminals {
            values: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Indices,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::IntegerTuple,
            1,
            &terminals,
            &inputs,
            &mut function_types,
        );

        assert_eq!(programs.len(), 4);

        assert!(
            programs.iter().any(|program| {
                program.expression() == "ulcorner(I)"
            })
        );

        assert!(
            programs.iter().any(|program| {
                program.expression() == "urcorner(I)"
            })
        );

        assert!(
            programs.iter().any(|program| {
                program.expression() == "llcorner(I)"
            })
        );

        assert!(
            programs.iter().any(|program| {
                program.expression() == "lrcorner(I)"
            })
        );
    }

    #[test]
    fn crop_can_be_composed_with_corners() {
        let terminals = Terminals {
            values: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
            InputSpec {
                name: "S".to_string(),
                ty: Type::Indices,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate_up_to(
            Type::Grid,
            2,
            &terminals,
            &inputs,
            &mut function_types,
        );

        assert!(
            programs.iter().any(|program| {
                program.expression()
                    == "crop(I, ulcorner(S), lrcorner(S))"
            })
        );
    }

    #[test]
    fn add_overloads_are_generated_from_typed_inputs() {
        let terminals = Terminals {
            values: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Integer,
            },
            InputSpec {
                name: "T".to_string(),
                ty: Type::IntegerTuple,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::IntegerTuple,
            1,
            &terminals,
            &inputs,
            &mut function_types,
        );

        let expressions: std::collections::BTreeSet<_> = programs
            .iter()
            .map(|program| program.expression())
            .collect();

        assert!(expressions.contains("add(I, T)"));
        assert!(expressions.contains("add(T, I)"));
        assert!(expressions.contains("add(T, T)"));
        assert!(!expressions.contains("add(I, I)"));
    }

    #[test]
    fn integer_add_is_generated_from_typed_inputs() {
        let terminals = Terminals {
            values: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "A".to_string(),
                ty: Type::Integer,
            },
            InputSpec {
                name: "B".to_string(),
                ty: Type::Integer,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::Integer,
            1,
            &terminals,
            &inputs,
            &mut function_types,
        );

        let expressions: std::collections::BTreeSet<_> = programs
            .iter()
            .map(|program| program.expression())
            .collect();

        assert!(expressions.contains("add(A, A)"));
        assert!(expressions.contains("add(A, B)"));
        assert!(expressions.contains("add(B, A)"));
        assert!(expressions.contains("add(B, B)"));
    }

    #[test]
    fn integer_tuple_generation_uses_tuple_terminals() {
        let terminals = Terminals {
            values: vec![
                NamedTerminal::new("ORIGIN", Value::IntegerTuple((0, 0))),
                NamedTerminal::new("UP", Value::IntegerTuple((1, 0))),
                NamedTerminal::new("RIGHT", Value::IntegerTuple((0, 1))),
            ],
        };

        let inputs = vec![];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::IntegerTuple,
            0,
            &terminals,
            &inputs,
            &mut function_types,
        );

        let expressions: std::collections::BTreeSet<_> = programs
            .iter()
            .map(|program| program.expression())
            .collect();

        assert_eq!(programs.len(), 3);

        assert!(expressions.contains("ORIGIN"));
        assert!(expressions.contains("UP"));
        assert!(expressions.contains("RIGHT"));
    }

    #[test]
    fn arc_agi_terminals_are_named_and_typed() {
        let terminals = Terminals::arc_agi();

        assert_eq!(terminals.values.len(), 28);

        assert!(terminals.values.iter().any(|terminal| {
            terminal.name == "ZERO"
                && matches!(terminal.value, Value::Integer(0))
        }));

        assert!(terminals.values.iter().any(|terminal| {
            terminal.name == "NEG_TWO"
                && matches!(terminal.value, Value::Integer(-2))
        }));

        assert!(terminals.values.iter().any(|terminal| {
            terminal.name == "ORIGIN"
                && matches!(terminal.value, Value::IntegerTuple((0, 0)))
        }));

        assert!(terminals.values.iter().any(|terminal| {
            terminal.name == "UP"
                && matches!(terminal.value, Value::IntegerTuple((1, 0)))
        }));

        assert!(terminals.values.iter().any(|terminal| {
            terminal.name == "THREE_BY_THREE"
                && matches!(terminal.value, Value::IntegerTuple((3, 3)))
        }));
    }

    #[test]
    fn named_integer_terminal_is_generated() {
        let terminals = Terminals::arc_agi();

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::Integer,
            0,
            &terminals,
            &[],
            &mut function_types,
        );

        assert!(programs.iter().any(|program| {
            program.expression() == "TWO"
        }));
    }

    #[test]
    fn named_terminals_can_be_composed() {
        let terminals = Terminals::arc_agi();

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::Integer,
            1,
            &terminals,
            &[],
            &mut function_types,
        );

        let program = programs
            .into_iter()
            .find(|program| {
                program.expression() == "add(TWO, ONE)"
            })
            .expect("expected add(TWO, ONE) to be generated");

        let environment = InputEnvironment::new();

        let result = program
            .output_with_inputs(&environment)
            .expect("expected program to evaluate");

        match result {
            Value::Integer(value) => assert_eq!(value, 3),
            _ => panic!("expected Integer result"),
        }
    }

    #[test]
    fn named_integer_tuple_terminal_is_generated() {
        let terminals = Terminals::arc_agi();

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::IntegerTuple,
            0,
            &terminals,
            &[],
            &mut function_types,
        );

        let program = programs
            .into_iter()
            .find(|program| {
                program.expression() == "UP"
            })
            .expect("expected UP to be generated");

        let result = program
            .output_with_inputs(&InputEnvironment::new())
            .expect("expected program to evaluate");

        match result {
            Value::IntegerTuple(value) => {
                assert_eq!(value, (1, 0));
            }
            _ => panic!("expected IntegerTuple result"),
        }
    }

    #[test]
    fn named_integer_tuple_terminals_can_be_composed() {
        let terminals = Terminals::arc_agi();

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::IntegerTuple,
            1,
            &terminals,
            &[],
            &mut function_types,
        );

        let program = programs
            .into_iter()
            .find(|program| {
                program.expression() == "add(UP, RIGHT)"
            })
            .expect("expected add(UP, RIGHT) to be generated");

        let result = program
            .output_with_inputs(&InputEnvironment::new())
            .expect("expected program to evaluate");

        match result {
            Value::IntegerTuple(value) => {
                assert_eq!(value, (1, 1));
            }
            _ => panic!("expected IntegerTuple result"),
        }
    }

    #[test]
    fn named_terminals_are_filtered_by_type() {
        let terminals = Terminals::arc_agi();

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let integers = generate(
            Type::Integer,
            0,
            &terminals,
            &[],
            &mut function_types,
        );

        assert!(integers.iter().all(|program| {
            matches!(
                program.output_with_inputs(&InputEnvironment::new()),
                Ok(Value::Integer(_))
            )
        }));

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let integer_tuples = generate(
            Type::IntegerTuple,
            0,
            &terminals,
            &[],
            &mut function_types,
        );

        assert!(integer_tuples.iter().all(|program| {
            matches!(
                program.output_with_inputs(&InputEnvironment::new()),
                Ok(Value::IntegerTuple(_))
            )
        }));
    }

    #[test]
    fn boolean_terminals_are_generated() {
        let terminals = Terminals::arc_agi();

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::Boolean,
            0,
            &terminals,
            &[],
            &mut function_types,
        );

        assert!(programs.iter().any(|program| {
            program.expression() == "F"
        }));

        assert!(programs.iter().any(|program| {
            program.expression() == "T"
        }));
    }

    #[test]
    fn flip_can_be_generated_from_boolean_terminal() {
        let terminals = Terminals::arc_agi();

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::Boolean,
            1,
            &terminals,
            &[],
            &mut function_types,
        );

        let program = programs
            .into_iter()
            .find(|program| {
                program.expression() == "flip(F)"
            })
            .expect("expected flip(F) to be generated");

        let result = program
            .output_with_inputs(&InputEnvironment::new())
            .expect("expected program to evaluate");

        match result {
            Value::Boolean(value) => assert!(value),
            _ => panic!("expected Boolean result"),
        }
    }

    #[test]
    fn flip_true_terminal_evaluates_to_false() {
        let terminals = Terminals::arc_agi();

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::Boolean,
            1,
            &terminals,
            &[],
            &mut function_types,
        );

        let program = programs
            .into_iter()
            .find(|program| {
                program.expression() == "flip(T)"
            })
            .expect("expected flip(T) to be generated");

        let result = program
            .output_with_inputs(&InputEnvironment::new())
            .expect("expected program to evaluate");

        match result {
            Value::Boolean(value) => assert!(!value),
            _ => panic!("expected Boolean result"),
        }
    }

    #[test]
    fn typed_inputs_can_form_a_function_body() {
        let terminals = Terminals {
            values: vec![
                NamedTerminal::new("TWO", Value::Integer(2)),
            ],
        };

        let inputs = vec![
            InputSpec {
                name: "X".to_string(),
                ty: Type::Integer,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::Integer,
            1,
            &terminals,
            &inputs,
            &mut function_types,
        );

        assert!(
            programs.iter().any(|program| {
                program.expression() == "add(TWO, X)"
            })
        );
    }

    #[test]
    fn function_terminals_are_generated_by_function_type() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function_type = function_types.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        let function = crate::function::Function::new(
            vec![
                InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let terminals = Terminals {
            values: vec![
                NamedTerminal::new(
                    "IDENTITY",
                    Value::Function(Box::new(function)),
                ),
            ],
        };

        assert_eq!(
            terminals.values[0]
                .value
                .output_type_with(&mut function_types),
            function_type,
        );

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function_type = function_types.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        let function = crate::function::Function::new(
            vec![
                InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let terminals = Terminals {
            values: vec![
                NamedTerminal::new(
                    "IDENTITY",
                    Value::Function(Box::new(function)),
                ),
            ],
        };

        assert_eq!(
            terminals.values[0]
                .value
                .output_type_with(&mut function_types),
            function_type,
        );

        let programs = generate(
            function_type,
            0,
            &terminals,
            &[],
            &mut function_types,
        );

        assert!(
            programs.iter().any(|program| {
                program.expression() == "IDENTITY"
            })
        );
    }

    #[test]
    fn generate_includes_lbind() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        function_types.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let target_type = function_types.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        let function = Function::new(
            vec![
                InputSpec {
                    name: "x".to_string(),
                    ty: Type::Integer,
                },
                InputSpec {
                    name: "y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "x".to_string(),
                Type::Integer,
            ),
        );

        let terminals = Terminals {
            values: vec![
                NamedTerminal {
                    name: "F",
                    value: Value::Function(Box::new(function)),
                },
                NamedTerminal {
                    name: "10",
                    value: Value::Integer(10),
                },
            ],
        };

        let programs = generate(
            target_type,
            1,
            &terminals,
            &[],
            &mut function_types,
        );

        assert!(
            programs
                .iter()
                .any(|program| {
                    program.expression()
                        == "lbind(F, 10)"
                })
        );
    }

    #[test]
    fn generated_lbind_evaluates() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let target_type = function_types.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        let function = Function::new(
            vec![
                InputSpec {
                    name: "x".to_string(),
                    ty: Type::Integer,
                },
                InputSpec {
                    name: "y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "x".to_string(),
                Type::Integer,
            ),
        );

        let terminals = Terminals {
            values: vec![
                NamedTerminal {
                    name: "F",
                    value: Value::Function(Box::new(function)),
                },
                NamedTerminal {
                    name: "10",
                    value: Value::Integer(10),
                },
            ],
        };

        let programs = generate(
            target_type,
            1,
            &terminals,
            &[],
            &mut function_types,
        );

        let program = programs
            .iter()
            .find(|program| {
                println!(
                    "candidate: {:?}",
                    program.expression()
                );

                program.expression() == "lbind(F, 10)"
            })
            .expect("generated lbind not found");

        let value = program
            .output()
            .expect("generated lbind should evaluate");

        match value {
            Value::Function(function) => {
                assert_eq!(function.inputs.len(), 1);
                assert_eq!(
                    function.inputs[0].ty,
                    Type::Integer
                );
            }

            other => {
                panic!(
                    "expected Function, got {:?}",
                    other
                );
            }
        }
    }

    #[test]
    fn generate_includes_chained_lbind() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        function_types.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        function_types.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        let target_type = function_types.type_of(
            &[],
            Type::Integer,
        );

        let function = Function::new(
            vec![
                InputSpec {
                    name: "x".to_string(),
                    ty: Type::Integer,
                },
                InputSpec {
                    name: "y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "x".to_string(),
                Type::Integer,
            ),
        );

        let terminals = Terminals {
            values: vec![
                NamedTerminal {
                    name: "F",
                    value: Value::Function(Box::new(function)),
                },
                NamedTerminal {
                    name: "10",
                    value: Value::Integer(10),
                },
                NamedTerminal {
                    name: "20",
                    value: Value::Integer(20),
                },
            ],
        };

        let programs = generate(
            target_type,
            2,
            &terminals,
            &[],
            &mut function_types,
        );

        assert!(
            programs
                .iter()
                .any(|program| {
                    program.expression()
                        == "lbind(lbind(F, 10), 20)"
                })
        );
    }

    #[test]
    fn generate_includes_apply() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = Function::new(
            vec![
                InputSpec {
                    name: "x".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "x".to_string(),
                Type::Integer,
            ),
        );

        let terminals = Terminals {
            values: vec![
                NamedTerminal {
                    name: "F",
                    value: Value::Function(Box::new(function)),
                },
                NamedTerminal {
                    name: "10",
                    value: Value::Integer(10),
                },
            ],
        };

        let programs = generate(
            Type::Integer,
            1,
            &terminals,
            &[],
            &mut function_types,
        );

        assert!(
            programs.iter().any(|program| {
                program.expression() == "apply(F, 10)"
            })
        );
    }

    #[test]
    fn generated_apply_evaluates() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = Function::new(
            vec![
                InputSpec {
                    name: "x".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "x".to_string(),
                Type::Integer,
            ),
        );

        let terminals = Terminals {
            values: vec![
                NamedTerminal {
                    name: "F",
                    value: Value::Function(Box::new(function)),
                },
                NamedTerminal {
                    name: "10",
                    value: Value::Integer(10),
                },
            ],
        };

        let programs = generate(
            Type::Integer,
            1,
            &terminals,
            &[],
            &mut function_types,
        );

        let program = programs
            .iter()
            .find(|program| {
                program.expression() == "apply(F, 10)"
            })
            .expect("generated apply not found");

        let value = program
            .output()
            .expect("generated apply should evaluate");

        match value {
            Value::Integer(value) => {
                assert_eq!(value, 10);
            }

            other => {
                panic!(
                    "expected Integer, got {:?}",
                    other
                );
            }
        }
    }

    #[test]
    fn generate_includes_apply_with_multiple_arguments() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = Function::new(
            vec![
                InputSpec {
                    name: "x".to_string(),
                    ty: Type::Integer,
                },
                InputSpec {
                    name: "y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "x".to_string(),
                Type::Integer,
            ),
        );

        let terminals = Terminals {
            values: vec![
                NamedTerminal {
                    name: "F",
                    value: Value::Function(Box::new(function)),
                },
                NamedTerminal {
                    name: "10",
                    value: Value::Integer(10),
                },
                NamedTerminal {
                    name: "20",
                    value: Value::Integer(20),
                },
            ],
        };

        let programs = generate(
            Type::Integer,
            1,
            &terminals,
            &[],
            &mut function_types,
        );

        assert!(
            programs.iter().any(|program| {
                program.expression()
                    == "apply(F, 10, 20)"
            })
        );
    }

    #[test]
    fn generated_apply_with_multiple_arguments_evaluates() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = Function::new(
            vec![
                InputSpec {
                    name: "x".to_string(),
                    ty: Type::Integer,
                },
                InputSpec {
                    name: "y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "x".to_string(),
                Type::Integer,
            ),
        );

        let terminals = Terminals {
            values: vec![
                NamedTerminal {
                    name: "F",
                    value: Value::Function(Box::new(function)),
                },
                NamedTerminal {
                    name: "10",
                    value: Value::Integer(10),
                },
                NamedTerminal {
                    name: "20",
                    value: Value::Integer(20),
                },
            ],
        };

        let programs = generate(
            Type::Integer,
            1,
            &terminals,
            &[],
            &mut function_types,
        );

        let program = programs
            .iter()
            .find(|program| {
                program.expression()
                    == "apply(F, 10, 20)"
            })
            .expect("generated apply not found");

        let value = program
            .output()
            .expect("generated apply should evaluate");

        match value {
            Value::Integer(value) => {
                assert_eq!(value, 10);
            }

            other => {
                panic!(
                    "expected Integer, got {:?}",
                    other
                );
            }
        }
    }
}
