mod types;
mod primitives;
mod registry;
mod search;
mod connection;
mod environment;
mod signature;
mod function;

use crate::registry::Type;
use crate::search::generate_corpus_parallel;
use crate::signature::InputSpec;

fn main() {
    let inputs = vec![
        InputSpec {
            name: "I".to_string(),
            ty: Type::Grid,
        },
    ];

    let mut function_types =
        crate::registry::FunctionTypeRegistry::new();

    for primitive in crate::registry::PRIMITIVES {
        function_types.type_of(
            primitive.inputs,
            primitive.output,
        );
    }

    println!(
        "Registered function types: {}",
        function_types.all_types().count()
    );

    for (id, function_type) in function_types.all_types() {
        println!(
            "{:?}: {:?} -> {:?}",
            id,
            function_type.inputs,
            function_type.output,
        );
    }

    let mut rng = rand::rng();

    let corpus = generate_corpus_parallel(
        4,
        1000,
        &inputs,
        &mut rng,
        100,
        5000,
    )
    .unwrap();

    println!(
        "Random corpus: {} entries",
        corpus.len()
    );

    for (index, entry) in corpus.iter().enumerate() {
        println!(
            "{}: {}",
            index + 1,
            entry.connection.expression()
        );
    }
}
