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

pub fn generate(
    output_type: Type,
    depth: usize,
    terminals: &Terminals,
    inputs: &[InputSpec],
) -> Vec<Connection> {
    let mut programs = Vec::new();

    if depth == 0 {
        match output_type {
            Type::Integer => {
                for value in &terminals.integers {
                    programs.push(Connection::terminal(
                        Value::Integer(*value),
                    ));
                }
            }

            Type::Grid => {
                programs.extend(
                    generate_inputs(
                        output_type,
                        inputs,
                    )
                );
            }
        }

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
