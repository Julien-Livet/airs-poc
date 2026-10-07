mod types;
mod primitives;
mod registry;
mod search;
mod connection;
mod environment;
mod signature;
mod function;

use crate::registry::Type;
use crate::search::{generate_corpus_parallel, CorpusEntry};
use crate::signature::InputSpec;
use std::fs;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "airs-poc")]
struct Args {
    /// Maximum depth of generated expressions
    #[arg(long, default_value_t = 10)]
    depth: usize,

    /// Number of corpus entries to generate
    #[arg(long, default_value_t = 1000)]
    count: usize,

    /// Number of trials
    #[arg(long, default_value_t = 100)]
    trials: usize,

    /// Number of max_attempts
    #[arg(long, default_value_t = 5000)]
    max_attempts: usize,

    #[arg(long, default_value = "dsl_dataset")]
    output: String,
}

fn main() {
    let args = Args::parse();

    let inputs = vec![
        InputSpec {
            name: "I".to_string(),
            ty: Type::Grid,
        },
    ];

    let mut rng = rand::rng();

    let corpus = generate_corpus_parallel(
        args.depth,
        args.count,
        &inputs,
        &mut rng,
        args.trials,
        args.max_attempts,
    )
    .unwrap();

     let json = corpus
        .iter()
        .map(CorpusEntry::to_json)
        .collect::<Result<Vec<_>, _>>()
        .expect("expected JSON representations");

    let text = serde_json::to_string_pretty(&json)
        .expect("expected JSON serialization");

    let filename = format!("{}_depth{}_{}programs.json", args.output, args.depth, args.count);

    fs::write(filename, text)
        .expect("failed to write corpus.json");

    for (_index, entry) in corpus.iter().enumerate() {
        println!(
            "{}",
            entry.connection.expression()
        );
    }
}
