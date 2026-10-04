use crate::registry::{PrimitiveEntry, Type, Value};
use crate::types::{Grid, Integer};

pub struct Terminals {
    pub integers: Vec<Integer>,
}

use crate::environment::InputEnvironment;

pub struct Dataset {
    pub environments: Vec<InputEnvironment>,
}

impl Dataset {
    pub fn from_grids(grids: Vec<Grid>) -> Self {
        let environments = grids
            .into_iter()
            .map(|grid| {
                let mut environment = InputEnvironment::new();

                environment.insert(
                    "I".to_string(),
                    Value::Grid(grid),
                );

                environment
            })
            .collect();

        Self { environments }
    }
}

#[derive(Clone)]
pub enum Connection {
    Primitive {
        primitive: &'static PrimitiveEntry,
        inputs: Vec<Box<Connection>>,
    },
    Constant(Value),
    Input {
        name: String,
        ty: Type,
    },
}

impl Connection {
    pub fn new(
        primitive: &'static PrimitiveEntry,
        inputs: Vec<Box<Connection>>,
    ) -> Result<Self, String> {
        if primitive.inputs.len() != inputs.len() {
            return Err(format!(
                "{} expects {} inputs, got {}",
                primitive.name,
                primitive.inputs.len(),
                inputs.len()
            ));
        }

        for (input, expected_type) in inputs.iter().zip(primitive.inputs.iter()) {
            let actual_type = input.output_type();

            if actual_type != *expected_type {
                return Err(format!(
                    "{} expects {:?}, got {:?}",
                    primitive.name,
                    expected_type,
                    actual_type
                ));
            }
        }

        Ok(Self::Primitive {
            primitive,
            inputs,
        })
    }

    pub fn output_with_inputs(
        &self,
        environment: &InputEnvironment,
    ) -> Result<Value, String> {
        match self {
            Connection::Input { name, ty } => {
                let value = environment
                    .get(name)
                    .ok_or_else(|| {
                        format!("missing input '{}'", name)
                    })?;

                if value.output_type() != *ty {
                    return Err(format!(
                        "input '{}' expects {:?}, got {:?}",
                        name,
                        ty,
                        value.output_type()
                    ));
                }

                Ok(value.clone())
            }

            Connection::Constant(value) => {
                Ok(value.clone())
            }

            Connection::Primitive {
                primitive,
                inputs,
            } => {
                let mut values = Vec::with_capacity(inputs.len());

                for input in inputs {
                    values.push(
                        input.output_with_inputs(environment)?
                    );
                }

                (primitive.apply)(&values)
            }
        }
    }

    pub fn is_open(&self) -> bool {
        match self {
            Connection::Input { name: _, ty: _ } => true,

            Connection::Constant(_) => false,

            Connection::Primitive { inputs, .. } => {
                inputs.iter().any(|input| input.is_open())
            }
        }
    }

    pub fn terminal(value: Value) -> Self {
        Self::Constant(value)
    }
    
    pub fn input(
        name: impl Into<String>,
        input_type: Type,
    ) -> Self {
        Self::Input {
            name: name.into(),
            ty: input_type,
        }
    }

    pub fn output_type(&self) -> Type {
        match self {
            Connection::Primitive { primitive, .. } => primitive.output,
            Connection::Constant(value) => value.output_type(),
            Connection::Input { ty, .. } => {
                *ty
            }
        }
    }

    pub fn output(&self) -> Result<Value, String> {
        if self.is_open() {
            return Err("cannot evaluate an open program without an input".to_string());
        }

        match self {
            Connection::Input { .. } => {
                unreachable!("open inputs are rejected above")
            }

            Connection::Constant(value) => {
                Ok(value.clone())
            }

            Connection::Primitive {
                primitive,
                inputs,
            } => {
                let mut values = Vec::with_capacity(inputs.len());

                for input in inputs {
                    values.push(input.output()?);
                }

                (primitive.apply)(&values)
            }
        }
    }

    pub fn expression(&self) -> String {
        match self {
            Connection::Input { name, .. } => {
                name.clone()
            }

            Connection::Constant(value) => {
                match value {
                    Value::Integer(value) => value.to_string(),
                    Value::Grid(_) => "Grid(...)".to_string(),
                    Value::IntegerTuple(value) => format!("{value:?}"),
                    Value::Indices(value) => format!("{value:?}"),
                    Value::Object(value) => format!("{value:?}"),
                }
            }

            Connection::Primitive { primitive, inputs } => {
                let args = inputs
                    .iter()
                    .map(|input| input.expression())
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{}({})", primitive.name, args)
            }
        }
    }
}

impl Value {
    pub fn output_type(&self) -> Type {
        match self {
            Value::Integer(_) => Type::Integer,
            Value::Grid(_) => Type::Grid,
            Value::IntegerTuple(_) => Type::IntegerTuple,
            Value::Indices(_) => Type::Indices,
            Value::Object(_) => Type::Object,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Type;
    use crate::search::{generate, semantic_signature, generate_inputs};
    use crate::signature::InputSpec;
    use crate::registry::{PRIMITIVES, Value};

    #[test]
    fn integer_generation_has_expected_cardinality() {
        let terminals = Terminals {
            integers: vec![1, 2, 3],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let depth_1 = generate(
            Type::Integer,
            1,
            &terminals,
            &inputs,
        );

        let depth_2 = generate(
            Type::Integer,
            2,
            &terminals,
            &inputs,
        );

        assert_eq!(depth_1.len(), 9);
        assert_eq!(depth_2.len(), 81);
    }

    #[test]
    fn same_program_runs_on_different_grids() {
        let primitive = crate::registry::find_by_name_and_inputs(
                "hmirror",
                &[Type::Grid],
            )
            .unwrap();

        let program = Connection::new(
            primitive,
            vec![Box::new(
                Connection::input("I", Type::Grid),
            )],
        )
        .unwrap();

        let environments_a = vec![
            {
                let mut environment = InputEnvironment::new();

                environment.insert(
                    "I".to_string(),
                    Value::Grid(vec![
                        vec![1, 2, 3],
                        vec![4, 5, 6],
                    ]),
                );

                environment
            },
        ];

        let environments_b = vec![
            {
                let mut environment = InputEnvironment::new();

                environment.insert(
                    "I".to_string(),
                    Value::Grid(vec![
                        vec![7, 8],
                        vec![9, 0],
                    ]),
                );

                environment
            },
        ];

        let signature_a = semantic_signature(
            &program,
            &environments_a,
        )
        .unwrap();

        let signature_b = semantic_signature(
            &program,
            &environments_b,
        )
        .unwrap();

        assert_eq!(
            signature_a,
            vec![
                vec![
                    vec![3, 2, 1],
                    vec![6, 5, 4],
                ],
            ]
        );

        assert_eq!(
            signature_b,
            vec![
                vec![
                    vec![8, 7],
                    vec![0, 9],
                ],
            ]
        );
    }

    #[test]
    fn mirror_programs_have_two_semantic_classes() {
        let dataset = Dataset::from_grids(vec![
                vec![
                    vec![1, 2, 3],
                    vec![4, 5, 6],
                ],
                vec![
                    vec![7, 8],
                    vec![9, 0],
                ],
                vec![
                    vec![1, 0, 1],
                    vec![0, 1, 1],
                ],
            ],
        );

        let terminals = Terminals {
            integers: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let programs = generate(
            Type::Grid,
            4,
            &terminals,
            &inputs,
        );

        let programs: Vec<_> = programs
            .into_iter()
            .filter(|program| {
                let expression = program.expression();

                !expression.contains("vconcat")
            })
            .collect();

        assert_eq!(programs.len(), 16);

        let mut signatures = std::collections::BTreeSet::new();

        for program in &programs {
            let signature = semantic_signature(
                program,
                &dataset.environments,
            )
            .unwrap();

            signatures.insert(signature);
        }

        assert_eq!(signatures.len(), 2);
    }

    #[test]
    fn generated_grid_programs_use_named_input() {
        let terminals = Terminals {
            integers: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let programs = generate(
            Type::Grid,
            1,
            &terminals,
            &inputs,
        );

        let expressions: Vec<String> = programs
            .iter()
            .map(|program| program.expression())
            .collect();

        assert!(expressions.contains(&"hmirror(I)".to_string()));
        assert!(expressions.contains(&"vmirror(I)".to_string()));
    }

    #[test]
    fn integer_constant_evaluates() {
        let program = Connection::terminal(
            Value::Integer(42),
        );

        let result = program.output().unwrap();

        assert!(matches!(
            result,
            Value::Integer(42)
        ));
    }

    #[test]
    fn connection_tracks_whether_it_is_open() {
        let terminals = Terminals {
            integers: vec![],
        };

        let closed = Connection::terminal(
            Value::Integer(42),
        );

        let open = Connection::input("I", Type::Grid);

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let programs = crate::search::generate(
            Type::Grid,
            1,
            &terminals,
            &inputs,
        );

        assert!(!closed.is_open());
        assert!(open.is_open());

        for program in programs {
            assert!(program.is_open());
        }
    }

    #[test]
    fn output_requires_a_closed_program() {
        let closed = Connection::terminal(
            Value::Integer(42),
        );

        assert!(closed.output().is_ok());

        let open = Connection::input("I", Type::Grid);

        let result = open.output();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "cannot evaluate an open program without an input"
        );
    }

    #[test]
    fn same_named_input_can_be_reused() {
        let input_1 = Connection::input("I", Type::Grid);
        let input_2 = Connection::input("I", Type::Grid);

        let hmirror = crate::registry::find_by_name_and_inputs(
            "hmirror",
            &[Type::Grid],
        )
        .unwrap();

        let vmirror = crate::registry::find_by_name_and_inputs(
            "vmirror",
            &[Type::Grid],
        )
        .unwrap();

        let program_1 = Connection::new(
            hmirror,
            vec![Box::new(input_1)],
        )
        .unwrap();

        let program_2 = Connection::new(
            vmirror,
            vec![Box::new(input_2)],
        )
        .unwrap();

        let grid = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
        ];

        let mut environment = InputEnvironment::new();

        environment.insert(
            "I".to_string(),
            Value::Grid(grid),
        );

        let result_1 = program_1
            .output_with_inputs(&environment)
            .unwrap();

        let result_2 = program_2
            .output_with_inputs(&environment)
            .unwrap();

        assert!(matches!(
            result_1,
            Value::Grid(ref grid)
                if *grid == vec![
                    vec![3, 2, 1],
                    vec![6, 5, 4],
                ]
        ));

        assert!(matches!(
            result_2,
            Value::Grid(ref grid)
                if *grid == vec![
                    vec![4, 5, 6],
                    vec![1, 2, 3],
                ]
        ));
    }

    #[test]
    fn same_input_can_feed_multiple_arguments() {
        let input_1 = Connection::input("I", Type::Grid);
        let input_2 = Connection::input("I", Type::Grid);

        let hmirror = crate::registry::find_by_name_and_inputs(
            "hmirror",
            &[Type::Grid],
        )
        .unwrap();

        let vconcat = crate::registry::find_by_name_and_inputs(
            "vconcat",
            &[Type::Grid, Type::Grid],
        )
        .unwrap();

        let mirrored = Connection::new(
            hmirror,
            vec![Box::new(input_1)],
        )
        .unwrap();

        let program = Connection::new(
            vconcat,
            vec![
                Box::new(mirrored),
                Box::new(input_2),
            ],
        )
        .unwrap();

        assert_eq!(
            program.expression(),
            "vconcat(hmirror(I), I)"
        );

        let grid = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
        ];

        let mut environment = InputEnvironment::new();

        environment.insert(
            "I".to_string(),
            Value::Grid(grid),
        );

        let result = program
            .output_with_inputs(&environment)
            .unwrap();

        assert!(matches!(
            result,
            Value::Grid(ref grid)
                if *grid == vec![
                    vec![3, 2, 1],
                    vec![6, 5, 4],
                    vec![1, 2, 3],
                    vec![4, 5, 6],
                ]
        ));
    }

    #[test]
    fn named_inputs_can_bind_to_different_values() {
        let input_i = Connection::input("I", Type::Grid);
        let input_j = Connection::input("J", Type::Grid);

        let vconcat = crate::registry::PRIMITIVES
            .iter()
            .find(|p| p.name == "vconcat")
            .unwrap();

        let program = Connection::new(
            vconcat,
            vec![
                Box::new(input_i),
                Box::new(input_j),
            ],
        )
        .unwrap();

        let grid_i = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        let grid_j = vec![
            vec![5, 6],
        ];

        let mut environment = InputEnvironment::new();

        environment.insert(
            "I".to_string(),
            Value::Grid(grid_i),
        );

        environment.insert(
            "J".to_string(),
            Value::Grid(grid_j),
        );

        let result = program
            .output_with_inputs(&environment)
            .unwrap();

        assert!(matches!(
            result,
            Value::Grid(ref grid)
                if *grid == vec![
                    vec![1, 2],
                    vec![3, 4],
                    vec![5, 6],
                ]
        ));
    }

    #[test]
    fn input_generation_uses_available_named_inputs() {
        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
            InputSpec {
                name: "J".to_string(),
                ty: Type::Grid,
            },
            InputSpec {
                name: "N".to_string(),
                ty: Type::Integer,
            },
        ];

        let programs = generate_inputs(
            Type::Grid,
            &inputs,
        );

        assert_eq!(programs.len(), 2);

        let expressions = programs
            .iter()
            .map(Connection::expression)
            .collect::<Vec<_>>();

        assert!(expressions.contains(&"I".to_string()));
        assert!(expressions.contains(&"J".to_string()));
        assert!(!expressions.contains(&"N".to_string()));
    }

    #[test]
    fn grid_generation_supports_multiple_named_inputs() {
        let terminals = Terminals {
            integers: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
            InputSpec {
                name: "J".to_string(),
                ty: Type::Grid,
            },
        ];

        let programs = generate(
            Type::Grid,
            1,
            &terminals,
            &inputs,
        );

        assert_eq!(programs.len(), 8);

        let expressions = programs
            .iter()
            .map(Connection::expression)
            .collect::<Vec<_>>();

        assert!(expressions.contains(&"hmirror(I)".to_string()));
        assert!(expressions.contains(&"hmirror(J)".to_string()));
        assert!(expressions.contains(&"vmirror(I)".to_string()));
        assert!(expressions.contains(&"vmirror(J)".to_string()));

        assert!(expressions.contains(
            &"vconcat(I, I)".to_string()
        ));

        assert!(expressions.contains(
            &"vconcat(I, J)".to_string()
        ));

        assert!(expressions.contains(
            &"vconcat(J, I)".to_string()
        ));

        assert!(expressions.contains(
            &"vconcat(J, J)".to_string()
        ));
    }

    #[test]
    fn dataset_can_hold_multiple_named_inputs() {
        let grid_i = vec![
            vec![1, 2],
        ];

        let grid_j = vec![
            vec![3, 4],
        ];

        let mut environment = InputEnvironment::new();

        environment.insert(
            "I".to_string(),
            Value::Grid(grid_i.clone()),
        );

        environment.insert(
            "J".to_string(),
            Value::Grid(grid_j.clone()),
        );

        let dataset = Dataset {
            environments: vec![
                environment,
            ],
        };

        assert_eq!(
            dataset.environments.len(),
            1,
        );

        assert!(matches!(
            dataset.environments[0].get("I"),
            Some(Value::Grid(grid)) if *grid == grid_i
        ));

        assert!(matches!(
            dataset.environments[0].get("J"),
            Some(Value::Grid(grid)) if *grid == grid_j
        ));
    }

    #[test]
    fn indices_connection_can_feed_ulcorner() {
        let primitive = crate::registry::PRIMITIVES
            .iter()
            .find(|primitive| primitive.name == "ulcorner")
            .expect("ulcorner should be registered");

        let indices = std::collections::BTreeSet::from([
            (2, 5),
            (4, 1),
            (7, 9),
        ]);

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::terminal(
                    Value::Indices(indices),
                )),
            ],
        )
        .expect("connection should be valid");

        let result = connection
            .output()
            .expect("connection should evaluate");

        assert!(matches!(
            result,
            Value::IntegerTuple((2, 1))
        ));
    }

    #[test]
    fn all_corner_connections_produce_expected_values() {
        let indices = std::collections::BTreeSet::from([
            (2, 5),
            (4, 1),
            (7, 9),
        ]);

        let cases = [
            ("ulcorner", (2, 1)),
            ("urcorner", (2, 9)),
            ("llcorner", (7, 1)),
            ("lrcorner", (7, 9)),
        ];

        for (name, expected) in cases {
            let primitive = crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| primitive.name == name)
                .expect("corner should be registered");

            let connection = Connection::new(
                primitive,
                vec![
                    Box::new(Connection::terminal(
                        Value::Indices(indices.clone()),
                    )),
                ],
            )
            .expect("connection should be valid");

            let result = connection
                .output()
                .expect("connection should evaluate");

            assert!(
                matches!(result, Value::IntegerTuple(value) if value == expected),
                "unexpected result for {name}: {result:?}"
            );
        }
    }

    #[test]
    fn crop_connection_can_be_evaluated() {
        let grid = vec![
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 0],
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 0],
        ];

        let crop = PRIMITIVES
            .iter()
            .find(|primitive| primitive.name == "crop")
            .expect("crop should be registered");

        let connection = Connection::new(
            crop,
            vec![
                Box::new(Connection::terminal(Value::Grid(grid))),
                Box::new(Connection::terminal(
                    Value::IntegerTuple((1, 1)),
                )),
                Box::new(Connection::terminal(
                    Value::IntegerTuple((2, 3)),
                )),
            ],
        )
        .expect("crop connection should be valid");

        assert!(matches!(
            connection.output(),
            Ok(Value::Grid(grid))
                if grid == vec![
                    vec![7, 8, 9],
                    vec![2, 3, 4],
                ]
        ));
    }

    #[test]
    fn add_overloads_can_be_connected_and_evaluated() {
        let integer_integer = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .unwrap();

        let program = Connection::new(
            integer_integer,
            vec![
                Box::new(Connection::terminal(Value::Integer(2))),
                Box::new(Connection::terminal(Value::Integer(3))),
            ],
        )
        .unwrap();

        match program.output().unwrap() {
            Value::Integer(value) => assert_eq!(value, 5),
            other => panic!("expected Integer, got {:?}", other),
        }

        let tuple_tuple = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::IntegerTuple, Type::IntegerTuple],
        )
        .unwrap();

        let program = Connection::new(
            tuple_tuple,
            vec![
                Box::new(Connection::terminal(
                    Value::IntegerTuple((2, 3)),
                )),
                Box::new(Connection::terminal(
                    Value::IntegerTuple((4, 5)),
                )),
            ],
        )
        .unwrap();

        match program.output().unwrap() {
            Value::IntegerTuple(value) => {
                assert_eq!(value, (6, 8));
            }
            other => panic!("expected IntegerTuple, got {:?}", other),
        }

        let integer_tuple = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::IntegerTuple],
        )
        .unwrap();

        let program = Connection::new(
            integer_tuple,
            vec![
                Box::new(Connection::terminal(Value::Integer(2))),
                Box::new(Connection::terminal(
                    Value::IntegerTuple((4, 5)),
                )),
            ],
        )
        .unwrap();

        match program.output().unwrap() {
            Value::IntegerTuple(value) => {
                assert_eq!(value, (6, 7));
            }
            other => panic!("expected IntegerTuple, got {:?}", other),
        }

        let tuple_integer = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::IntegerTuple, Type::Integer],
        )
        .unwrap();

        let program = Connection::new(
            tuple_integer,
            vec![
                Box::new(Connection::terminal(
                    Value::IntegerTuple((4, 5)),
                )),
                Box::new(Connection::terminal(Value::Integer(2))),
            ],
        )
        .unwrap();

        match program.output().unwrap() {
            Value::IntegerTuple(value) => {
                assert_eq!(value, (6, 7));
            }
            other => panic!("expected IntegerTuple, got {:?}", other),
        }
    }
}
