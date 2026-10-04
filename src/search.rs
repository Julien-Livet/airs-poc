use crate::connection::{Connection, Terminals};
use crate::registry::{Type, Value};
use crate::types::Grid;
use crate::environment::InputEnvironment;
use crate::signature::InputSpec;

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
) -> Vec<Connection> {
    generate_bounded(
        output_type,
        depth,
        terminals,
        inputs,
    )
}

pub fn generate(
    output_type: Type,
    depth: usize,
    terminals: &Terminals,
    inputs: &[InputSpec],
) -> Vec<Connection> {
    let mut programs = Vec::new();

    if depth == 0 {
        for terminal in &terminals.values {
            if terminal.value.ty() == output_type {
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

    programs
}

fn generate_bounded(
    output_type: Type,
    depth: usize,
    terminals: &Terminals,
    inputs: &[InputSpec],
) -> Vec<Connection> {
    let mut programs = Vec::new();

    if depth == 0 {
        return generate(
            output_type,
            0,
            terminals,
            inputs,
        );
    }

    programs.extend(
        generate_bounded(
            output_type,
            depth - 1,
            terminals,
            inputs,
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

        let programs = generate(
            Type::Grid,
            1,
            &terminals,
            &inputs,
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

        let programs = generate(
            Type::IntegerTuple,
            0,
            &terminals,
            &inputs,
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

        let programs = generate(
            Type::IntegerTuple,
            1,
            &terminals,
            &inputs,
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

        let programs = generate_up_to(
            Type::Grid,
            2,
            &terminals,
            &inputs,
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

        let programs = generate(
            Type::IntegerTuple,
            1,
            &terminals,
            &inputs,
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

        let programs = generate(
            Type::Integer,
            1,
            &terminals,
            &inputs,
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

        let programs = generate(
            Type::IntegerTuple,
            0,
            &terminals,
            &inputs,
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

        let programs = generate(
            Type::Integer,
            0,
            &terminals,
            &[],
        );

        assert!(programs.iter().any(|program| {
            program.expression() == "TWO"
        }));
    }

    #[test]
    fn named_terminals_can_be_composed() {
        let terminals = Terminals::arc_agi();

        let programs = generate(
            Type::Integer,
            1,
            &terminals,
            &[],
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

        let programs = generate(
            Type::IntegerTuple,
            0,
            &terminals,
            &[],
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

        let programs = generate(
            Type::IntegerTuple,
            1,
            &terminals,
            &[],
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

        let integers = generate(
            Type::Integer,
            0,
            &terminals,
            &[],
        );

        assert!(integers.iter().all(|program| {
            matches!(
                program.output_with_inputs(&InputEnvironment::new()),
                Ok(Value::Integer(_))
            )
        }));

        let integer_tuples = generate(
            Type::IntegerTuple,
            0,
            &terminals,
            &[],
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

        let programs = generate(
            Type::Boolean,
            0,
            &terminals,
            &[],
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

        let programs = generate(
            Type::Boolean,
            1,
            &terminals,
            &[],
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

        let programs = generate(
            Type::Boolean,
            1,
            &terminals,
            &[],
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
}
