mod types;
mod primitives;
mod registry;
mod search;
mod connection;
mod environment;
mod signature;

use crate::connection::{Dataset};
use crate::registry::{Type};
use crate::search::{generate, semantic_signature};
use crate::signature::InputSpec;

fn main() {
    let terminals = connection::Terminals {
        integers: vec![1, 2, 3],
    };

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
                vec![0, 1, 0],
            ],
        ],
    );

    let inputs = vec![
        InputSpec {
            name: "I".to_string(),
            ty: Type::Grid,
        },
    ];

    for depth in 0..=4 {
        let programs = generate(
            registry::Type::Grid,
            depth,
            &terminals,
            &inputs,
        );

        println!(
            "Grid depth {}: {} programs",
            depth,
            programs.len()
        );
    }

    use std::collections::BTreeMap;

    let programs = generate(
        registry::Type::Grid,
        4,
        &terminals,
        &inputs,
    );

    let mut classes: BTreeMap<Vec<Vec<Vec<i16>>>, Vec<String>> = BTreeMap::new();

    for program in &programs {
        let signature = semantic_signature(
            program,
            &dataset.environments,
        )
        .unwrap();

        classes
            .entry(signature)
            .or_default()
            .push(program.expression());
    }

    println!("Semantic classes: {}", classes.len());

    for (i, (_, expressions)) in classes.iter().enumerate() {
        println!("Class {}: {} programs", i + 1, expressions.len());

        for expression in expressions {
            println!("  {}", expression);
        }
    }
}
