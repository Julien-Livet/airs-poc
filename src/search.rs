use crate::connection::{Connection, Terminals};
use crate::registry::{Type, Value, PrimitiveEntry, FunctionTypeRegistry, FunctionTypeId};
use crate::types::Grid;
use crate::environment::InputEnvironment;
use crate::signature::InputSpec;
use std::collections::BTreeMap;
use rand::seq::SliceRandom;
use rand::RngExt;

const CORPUS_BATCH_SIZE: usize = 32;

fn select_corpus_entries(
    entries: Vec<CorpusEntry>,
    count: usize,
) -> Result<Vec<CorpusEntry>, String> {
    let entries =
        deduplicate_corpus_entries(entries);

    if entries.len() < count {
        return Err(format!(
            "could not generate {} corpus entries; got {}",
            count,
            entries.len(),
        ));
    }

    Ok(entries.into_iter().take(count).collect())
}

fn generate_corpus_entry_with_seed(
    depth: usize,
    inputs: &[InputSpec],
    seed: u64,
    trials: usize,
) -> Option<CorpusEntry> {
    let mut function_types =
        FunctionTypeRegistry::new();

    generate_corpus_entry(
        depth,
        inputs,
        &mut function_types,
        seed,
        trials,
    )
}

fn corpus_seeds(
    max_attempts: usize,
    rng: &mut impl rand::Rng,
) -> Vec<u64> {
    generate_seeds(max_attempts, rng)
}

fn corpus_entry_key(
    entry: &CorpusEntry,
) -> (Grid, String) {
    (
        entry.input.clone(),
        entry.connection.expression(),
    )
}

fn generate_corpus_parallel(
    depth: usize,
    count: usize,
    inputs: &[InputSpec],
    rng: &mut impl rand::Rng,
    trials: usize,
    max_attempts: usize,
) -> Result<Vec<CorpusEntry>, String> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let seeds =
        corpus_seeds(max_attempts, rng);

    let mut entries = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for batch in seeds.chunks(CORPUS_BATCH_SIZE) {
        for entry in
            generate_corpus_entries_parallel(
                depth,
                batch,
                inputs,
                trials,
            )
        {
            if seen.insert(corpus_entry_key(&entry)) {
                entries.push(entry);
            }
        }

        if entries.len() >= count {
            break;
        }
    }

    select_corpus_entries(
        entries,
        count,
    )
}

fn generate_corpus_entries_parallel(
    depth: usize,
    seeds: &[u64],
    inputs: &[InputSpec],
    trials: usize,
) -> Vec<CorpusEntry> {
    use rayon::prelude::*;

    seeds
        .par_iter()
        .filter_map(|&seed| {
            generate_corpus_entry_with_seed(
                depth,
                inputs,
                seed,
                trials,
            )
        })
        .collect()
}

fn generate_seeds(
    count: usize,
    rng: &mut impl rand::Rng,
) -> Vec<u64> {
    (0..count)
        .map(|_| rng.random())
        .collect()
}

fn generate_corpus_entry(
    depth: usize,
    inputs: &[InputSpec],
    function_types: &mut FunctionTypeRegistry,
    seed: u64,
    trials: usize,
) -> Option<CorpusEntry> {
    use rand::SeedableRng;

    let mut rng =
        rand::rngs::StdRng::seed_from_u64(seed);

    let connection =
        build_connection(
            Type::Grid,
            depth,
            inputs,
            function_types,
            &mut rng,
        )
        .ok()?;

    if !uses_input_i(&connection) {
        return None;
    }

    build_corpus_entry(
        connection,
        trials,
        &mut rng,
    )
}

fn generate_corpus(
    depth: usize,
    count: usize,
    inputs: &[InputSpec],
    rng: &mut impl rand::Rng,
    trials: usize,
    max_attempts: usize,
) -> Result<Vec<CorpusEntry>, String> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let mut corpus = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let seeds = corpus_seeds(max_attempts, rng);

    for seed in seeds {
        if corpus.len() == count {
            break;
        }

        if let Some(entry) =
            generate_corpus_entry_with_seed(
                depth,
                inputs,
                seed,
                trials,
            )
        {
            let key =
                corpus_entry_key(&entry);

            if seen.insert(key) {
                corpus.push(entry);
            }
        }
    }

    if corpus.len() < count {
        return Err(format!(
            "could not generate {} corpus entries after {} attempts; got {}",
            count,
            max_attempts,
            corpus.len(),
        ));
    }

    Ok(corpus)
}

fn deduplicate_corpus_entries(
    entries: Vec<CorpusEntry>,
) -> Vec<CorpusEntry> {
    let mut seen = std::collections::BTreeSet::new();
    let mut result = Vec::new();

    for entry in entries {
        if seen.insert(corpus_entry_key(&entry)) {
            result.push(entry);
        }
    }

    result
}

fn generate_corpus_entries(
    depth: usize,
    count: usize,
    inputs: &[InputSpec],
    function_types: &mut FunctionTypeRegistry,
    rng: &mut impl rand::Rng,
    trials: usize,
) -> Vec<CorpusEntry> {
    let connections = generate_connections(
        depth,
        count,
        inputs,
        function_types,
        rng,
    );

    let connections =
        deduplicate_connections(connections);

    connections
        .into_iter()
        .filter_map(|connection| {
            build_corpus_entry(
                connection,
                trials,
                rng,
            )
        })
        .collect()
}

fn build_corpus_entry(
    connection: Connection,
    trials: usize,
    rng: &mut impl rand::Rng,
) -> Option<CorpusEntry> {
    let input =
        find_valid_grid(
            &connection,
            trials,
            rng,
        )?;

    Some(CorpusEntry {
        input,
        connection,
    })
}

fn find_valid_grid(
    connection: &Connection,
    trials: usize,
    rng: &mut impl rand::Rng,
) -> Option<Grid> {
    for _ in 0..trials {
        let (input, _) =
            generate_structured_grid(
                (4, 4),
                (30, 30),
                rng,
            );

        if connection_produces_valid_grid(
            connection,
            input.clone(),
        ) {
            return Some(input);
        }
    }

    None
}

#[derive(Debug)]
struct CorpusEntry {
    input: Grid,
    connection: Connection,
}

fn add_corpus_entry(
    corpus: &mut Vec<CorpusEntry>,
    connection: Connection,
    input: Grid,
) {
    corpus.push(CorpusEntry {
        input,
        connection,
    });
}

fn generate_acceptable_connections(
    depth: usize,
    count: usize,
    inputs: &[InputSpec],
    function_types: &mut FunctionTypeRegistry,
    rng: &mut impl rand::Rng,
    trials: usize,
) -> Vec<Connection> {
    let connections = generate_connections(
        depth,
        count,
        inputs,
        function_types,
        rng,
    );

    let connections =
        acceptable_connections(
            connections,
            trials,
            rng,
        );

    deduplicate_connections(connections)
}

fn generate_connections(
    depth: usize,
    count: usize,
    inputs: &[InputSpec],
    function_types: &mut FunctionTypeRegistry,
    rng: &mut impl rand::Rng,
) -> Vec<Connection> {
    let mut connections = Vec::new();

    for _ in 0..count {
        if let Ok(connection) = build_connection(
            Type::Grid,
            depth,
            inputs,
            function_types,
            rng,
        ) {
            connections.push(connection);
        }
    }

    connections
}

fn deduplicate_connections(
    connections: Vec<Connection>,
) -> Vec<Connection> {
    let mut seen = std::collections::BTreeSet::new();
    let mut result = Vec::new();

    for connection in connections {
        let expression = connection.expression();

        if seen.insert(expression) {
            result.push(connection);
        }
    }

    result
}

fn acceptable_connections(
    connections: Vec<Connection>,
    trials: usize,
    rng: &mut impl rand::Rng,
) -> Vec<Connection> {
    connections
        .into_iter()
        .filter(|connection| {
            connection_is_acceptable(
                connection,
                trials,
                rng,
            )
        })
        .collect()
}

fn connection_is_acceptable(
    connection: &Connection,
    trials: usize,
    rng: &mut impl rand::Rng,
) -> bool {
    if !uses_input_i(connection) {
        return false;
    }

    connection_passes_grid_trials(
        connection,
        trials,
        rng,
    )
}

fn uses_input_i(connection: &Connection) -> bool {
    match connection {
        Connection::Input { name, .. } => {
            name == "I"
        }

        Connection::Constant { .. } => false,

        Connection::Primitive { inputs, .. }
        | Connection::Dynamic { inputs, .. } => {
            inputs.iter().any(|input| {
                uses_input_i(input)
            })
        }
    }
}

fn connection_passes_grid_trials(
    connection: &Connection,
    trials: usize,
    rng: &mut impl rand::Rng,
) -> bool {
    for _ in 0..trials {
        let (input, _) =
            generate_structured_grid(
                (4, 4),
                (30, 30),
                rng,
            );

        if connection_produces_valid_grid(
            connection,
            input,
        ) {
            return true;
        }
    }

    false
}

fn connection_produces_valid_grid(
    connection: &Connection,
    input: Grid,
) -> bool {
    let result = std::panic::catch_unwind(
        std::panic::AssertUnwindSafe(|| {
            evaluate_grid_connection(
                connection,
                input.clone(),
            )
        }),
    );

    match result {
        Ok(Ok(output)) => {
            is_valid_grid_output(&input, &output)
        }

        _ => false,
    }
}

fn evaluate_grid_connection(
    connection: &Connection,
    input: Grid,
) -> Result<Grid, String> {
    let environment =
        InputEnvironment::from([(
            "I".to_string(),
            Value::Grid(input),
        )]);

    match connection.output_with_inputs(&environment)? {
        Value::Grid(grid) => Ok(grid),
        _ => Err("program does not produce a Grid".to_string()),
    }
}

fn is_valid_grid_output(
    input: &Grid,
    output: &Grid,
) -> bool {
    if output.is_empty() {
        return false;
    }

    if output == input {
        return false;
    }

    if output
        .iter()
        .flatten()
        .any(|&value| !(0..=9).contains(&value))
    {
        return false;
    }

    true
}

fn random_grid_kind(rng: &mut impl rand::Rng) -> &'static str {
    let kinds = [
        "stripes",
        "blocks",
        "pattern",
        "gradient",
        "sparse",
        "random",
    ];

    kinds[rng.random_range(0..kinds.len())]
}

fn generate_structured_grid(
    min_size: (usize, usize),
    max_size: (usize, usize),
    rng: &mut impl rand::Rng,
) -> (Grid, &'static str) {
    let height = rng.random_range(min_size.0..=max_size.0);
    let width = rng.random_range(min_size.1..=max_size.1);
    let kind = random_grid_kind(rng);

    let mut grid = vec![vec![0; width]; height];

    if kind == "gradient" {
        for i in 0..height {
            let value =
                (i * 9) / std::cmp::max(height - 1, 1);

            for cell in &mut grid[i] {
                *cell = value as i16;
            }
        }
    }
    else if kind == "stripes" {
        for row in &mut grid {
            if rng.random_bool(0.5) {
                let color = rng.random_range(1..=9);
                row.fill(color);
            }
        }
    }
    else if kind == "blocks" {
        let block_count = rng.random_range(2..=5);

        for _ in 0..block_count {
            let mut r1 = rng.random_range(0..height);
            let mut r2 = rng.random_range(0..height);
            if r1 > r2 {
                std::mem::swap(&mut r1, &mut r2);
            }

            let mut c1 = rng.random_range(0..width);
            let mut c2 = rng.random_range(0..width);
            if c1 > c2 {
                std::mem::swap(&mut c1, &mut c2);
            }

            let color = rng.random_range(1..=9);

            for row in grid.iter_mut().take(r2 + 1).skip(r1) {
                for cell in row.iter_mut().take(c2 + 1).skip(c1) {
                    *cell = color;
                }
            }
        }
    }
    else if kind == "random" {
        for row in &mut grid {
            for cell in row {
                *cell = rng.random_range(0..=9);
            }
        }
    }
    else if kind == "sparse" {
        let count = rng.random_range(3..=9);

        for _ in 0..count {
            let row = rng.random_range(0..height);
            let col = rng.random_range(0..width);
            grid[row][col] = rng.random_range(1..=9);
        }
    }
    else if kind == "pattern" {
        let max_base_height = std::cmp::max(3, height / 2);
        let max_base_width = std::cmp::max(3, width / 2);

        let base_height =
            rng.random_range(2..max_base_height);
        let base_width =
            rng.random_range(2..max_base_width);

        let mut base =
            vec![vec![0i16; base_width]; base_height];

        for row in &mut base {
            for cell in row {
                *cell = rng.random_range(0..=9);
            }
        }

        for i in 0..height {
            for j in 0..width {
                grid[i][j] =
                    base[i % base_height][j % base_width];
            }
        }
    }

    (grid, kind)
}

fn dynamic_candidate_group(
    candidates: Vec<DynamicCandidate>,
) -> Option<CandidateGroup> {
    if candidates.is_empty() {
        return None;
    }

    Some(CandidateGroup::Dynamic { candidates })
}

fn primitive_candidate_group(
    name: &'static str,
    candidates: Vec<&'static PrimitiveEntry>,
) -> Option<CandidateGroup> {
    if candidates.is_empty() {
        return None;
    }

    Some(CandidateGroup::Primitive {
        name,
        candidates,
    })
}

fn shuffle_candidate_groups(
    groups: &mut [CandidateGroup],
    rng: &mut impl rand::Rng,
) {
    groups.shuffle(rng);

    for group in groups {
        match group {
            CandidateGroup::Dynamic {
                candidates, ..
            } => {
                candidates.shuffle(rng);
            }

            CandidateGroup::Primitive {
                candidates, ..
            } => {
                candidates.shuffle(rng);
            }
        }
    }
}

#[derive(Debug)]
enum CandidateGroup {
    Dynamic {
        candidates: Vec<DynamicCandidate>,
    },
    Primitive {
        name: &'static str,
        candidates: Vec<&'static PrimitiveEntry>,
    },
}

impl CandidateGroup {
    fn name(&self) -> &'static str {
        match self {
            CandidateGroup::Dynamic { candidates } => candidates[0].name(),
            CandidateGroup::Primitive { name, .. } => name,
        }
    }
}

impl CandidateGroup {
    fn len(&self) -> usize {
        match self {
            CandidateGroup::Dynamic {
                candidates, ..
            } => candidates.len(),

            CandidateGroup::Primitive {
                candidates, ..
            } => candidates.len(),
        }
    }
}

fn candidate_groups(
    output_type: Type,
    function_types: &FunctionTypeRegistry,
) -> Vec<CandidateGroup> {
    let mut groups = Vec::new();

    if let Some(group) =
        dynamic_candidate_group(
            lbind_candidates(
                output_type,
                function_types,
            )
            .into_iter()
            .map(|(id, ty)| {
                DynamicCandidate::lbind(id, ty)
            })
            .collect(),
        )
    {
        groups.push(group);
    }

    if let Some(group) =
        dynamic_candidate_group(
            rbind_candidates(
                output_type,
                function_types,
            )
            .into_iter()
            .map(|(id, ty)| {
                DynamicCandidate::rbind(id, ty)
            })
            .collect(),
        )
    {
        groups.push(group);
    }

    if let Some(group) =
        dynamic_candidate_group(
            apply_candidates(
                output_type,
                function_types,
            )
            .into_iter()
            .map(DynamicCandidate::apply)
            .collect(),
        )
    {
        groups.push(group);
    }

    for (name, candidates) in
        primitive_candidates_by_name(output_type)
    {
        if let Some(group) =
            primitive_candidate_group(
                name,
                candidates,
            )
        {
            groups.push(group);
        }
    }

    groups
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DynamicCandidate {
    Lbind(FunctionTypeId, Type),
    Rbind(FunctionTypeId, Type),
    Apply(FunctionTypeId),
}

impl DynamicCandidate {
    fn lbind(function_id: FunctionTypeId, fixed_type: Type) -> Self {
        Self::Lbind(function_id, fixed_type)
    }

    fn rbind(function_id: FunctionTypeId, fixed_type: Type) -> Self {
        Self::Rbind(function_id, fixed_type)
    }

    fn apply(function_id: FunctionTypeId) -> Self {
        Self::Apply(function_id)
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Lbind(..) => "lbind",
            Self::Rbind(..) => "rbind",
            Self::Apply(..) => "apply",
        }
    }
}

fn apply_candidates(
    output_type: Type,
    function_types: &FunctionTypeRegistry,
) -> Vec<FunctionTypeId> {
    function_types
        .all_types()
        .filter_map(|(id, function_type)| {
            if function_type.output == output_type {
                Some(id)
            } else {
                None
            }
        })
        .collect()
}

fn rbind_candidates(
    output_type: Type,
    function_types: &FunctionTypeRegistry,
) -> Vec<(FunctionTypeId, Type)> {
    let Type::Function(output_id) = output_type else {
        return Vec::new();
    };

    let Some(output_inputs) =
        function_types.inputs(output_id)
    else {
        return Vec::new();
    };

    let Some(output_output) =
        function_types.output(output_id)
    else {
        return Vec::new();
    };

    function_types
        .all_types()
        .filter_map(|(source_id, source_type)| {
            if source_type.inputs.len()
                != output_inputs.len() + 1
            {
                return None;
            }

            let last =
                source_type.inputs.len() - 1;

            if source_type.inputs[..last]
                != *output_inputs
            {
                return None;
            }

            if source_type.output != output_output {
                return None;
            }

            Some((
                source_id,
                source_type.inputs[last],
            ))
        })
        .collect()
}

fn lbind_candidates(
    output_type: Type,
    function_types: &FunctionTypeRegistry,
) -> Vec<(FunctionTypeId, Type)> {
    let Type::Function(output_id) = output_type else {
        return Vec::new();
    };

    let Some(output_inputs) =
        function_types.inputs(output_id)
    else {
        return Vec::new();
    };

    let Some(output_output) =
        function_types.output(output_id)
    else {
        return Vec::new();
    };

    function_types
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
                source_id,
                source_type.inputs[0],
            ))
        })
        .collect()
}

fn try_dynamic_candidate(
    candidate: DynamicCandidate,
    depth: usize,
    inputs: &[InputSpec],
    function_types: &mut FunctionTypeRegistry,
    rng: &mut impl rand::Rng,
) -> Option<Connection> {
    match candidate {
        DynamicCandidate::Lbind(
            source_id,
            fixed_type,
        ) => {
            let source_type =
                Type::Function(source_id);

            let function =
                build_connection(
                    source_type,
                    depth - 1,
                    inputs,
                    function_types,
                    rng,
                )
                .ok()?;

            let fixed =
                build_connection(
                    fixed_type,
                    depth - 1,
                    inputs,
                    function_types,
                    rng,
                )
                .ok()?;

            Connection::lbind(
                function,
                fixed,
                function_types,
            )
            .ok()
        }

        DynamicCandidate::Rbind(
            source_id,
            fixed_type,
        ) => {
            let source_type =
                Type::Function(source_id);

            let function =
                build_connection(
                    source_type,
                    depth - 1,
                    inputs,
                    function_types,
                    rng,
                )
                .ok()?;

            let fixed =
                build_connection(
                    fixed_type,
                    depth - 1,
                    inputs,
                    function_types,
                    rng,
                )
                .ok()?;

            Connection::rbind(
                function,
                fixed,
                function_types,
            )
            .ok()
        }

        DynamicCandidate::Apply(function_id) => {
            let function_type =
                Type::Function(function_id);

            let function =
                build_connection(
                    function_type,
                    depth - 1,
                    inputs,
                    function_types,
                    rng,
                )
                .ok()?;

            let argument_types =
                function_types
                    .inputs(function_id)?
                    .to_vec();

            let arguments =
                argument_types
                    .iter()
                    .map(|argument_type| {
                        build_connection(
                            *argument_type,
                            depth - 1,
                            inputs,
                            function_types,
                            rng,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?;

            Connection::apply(
                function,
                arguments,
                function_types,
            )
            .ok()
        }
    }
}

fn build_connection(
    output_type: Type,
    depth: usize,
    inputs: &[InputSpec],
    function_types: &mut FunctionTypeRegistry,
    rng: &mut impl rand::Rng,
) -> Result<Connection, String> {
    if depth == 0 {
        return choose_input(output_type, inputs, rng)
            .ok_or_else(|| {
                format!(
                    "no input available for type {:?}",
                    output_type
                )
            });
    }

    let mut groups =
        candidate_groups(
            output_type,
            function_types,
        );

    shuffle_candidate_groups(
        &mut groups,
        rng,
    );

    for group in groups {
        match group {
            CandidateGroup::Dynamic {
                candidates,
                ..
            } => {
                for candidate in candidates {
                    if let Some(connection) =
                        try_dynamic_candidate(
                            candidate,
                            depth,
                            inputs,
                            function_types,
                            rng,
                        )
                    {
                        return Ok(connection);
                    }
                }
            }

            CandidateGroup::Primitive {
                candidates,
                ..
            } => {
                for primitive in candidates {
                    let inputs = primitive
                        .inputs
                        .iter()
                        .map(|input_type| {
                            build_connection(
                                *input_type,
                                depth - 1,
                                inputs,
                                function_types,
                                rng,
                            )
                            .map(Box::new)
                        })
                        .collect::<Result<Vec<_>, _>>();

                    let Ok(inputs) = inputs else {
                        continue;
                    };

                    if let Ok(connection) =
                        Connection::new(
                            primitive,
                            inputs,
                        )
                    {
                        return Ok(connection);
                    }
                }
            }
        }
    }

    Err(format!(
        "could not build connection for type {:?} at depth {}",
        output_type,
        depth
    ))
}

fn choose_input(
    output_type: Type,
    inputs: &[InputSpec],
    rng: &mut impl rand::Rng,
) -> Option<Connection> {
    let mut candidates = inputs
        .iter()
        .filter(|input| input.ty == output_type)
        .map(InputSpec::connection)
        .collect::<Vec<_>>();

    candidates.shuffle(rng);

    candidates.into_iter().next()
}

fn choose_primitive(
    groups: &[Vec<&'static PrimitiveEntry>],
) -> Option<&'static PrimitiveEntry> {
    groups
        .first()
        .and_then(|group| group.first())
        .copied()
}

fn shuffle_primitive_overloads(
    groups: &mut [Vec<&'static PrimitiveEntry>],
    rng: &mut impl rand::Rng,
) {
    for group in groups {
        group.shuffle(rng);
    }
}

fn shuffled_primitive_groups(
    output_type: Type,
    rng: &mut impl rand::Rng,
) -> Vec<Vec<&'static PrimitiveEntry>> {
    let candidates =
        primitive_candidates_by_name(output_type);

    let mut groups: Vec<_> =
        candidates.into_values().collect();

    groups.shuffle(rng);

    groups
}

fn primitive_candidates_by_name(
    output_type: Type,
) -> BTreeMap<&'static str, Vec<&'static PrimitiveEntry>> {
    let mut candidates = BTreeMap::new();

    for primitive in crate::registry::PRIMITIVES {
        if primitive.output != output_type {
            continue;
        }

        candidates
            .entry(primitive.name)
            .or_insert_with(Vec::new)
            .push(primitive);
    }

    candidates
}

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
    use rand::SeedableRng;
    use crate::registry::DynamicPrimitive;

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

    #[test]
    fn primitive_candidates_are_grouped_by_name() {
        let mut rng = rand::rng();

        let groups =
            shuffled_primitive_groups(Type::Grid, &mut rng);

        assert!(!groups.is_empty());

        for group in groups {
            assert!(!group.is_empty());

            let name = group[0].name;

            assert!(group.iter().all(|primitive| {
                primitive.name == name
            }));
        }
    }

    #[test]
    fn primitive_candidates_are_grouped_and_shuffled() {
        let mut rng = rand::rng();

        let mut groups =
            shuffled_primitive_groups(Type::Grid, &mut rng);

        shuffle_primitive_overloads(
            &mut groups,
            &mut rng,
        );

        assert!(!groups.is_empty());

        for group in groups {
            assert!(!group.is_empty());

            let name = group[0].name;

            assert!(group.iter().all(|primitive| {
                primitive.name == name
            }));
        }
    }

    #[test]
    fn can_choose_a_primitive_for_output_type() {
        let mut rng = rand::rng();

        let mut groups =
            shuffled_primitive_groups(Type::Grid, &mut rng);

        shuffle_primitive_overloads(
            &mut groups,
            &mut rng,
        );

        let primitive = choose_primitive(&groups)
            .expect("expected a Grid-producing primitive");

        assert_eq!(primitive.output, Type::Grid);
    }

    #[test]
    fn can_choose_input_at_depth_zero() {
        let mut rng = rand::rng();

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
            InputSpec {
                name: "N".to_string(),
                ty: Type::Integer,
            },
        ];

        let connection =
            choose_input(Type::Grid, &inputs, &mut rng)
                .expect("expected a Grid input");

        assert_eq!(connection.output_type(), Type::Grid);
    }

    #[test]
    fn build_connection_handles_depth_zero() {
        let mut rng = rand::rng();

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types =
            FunctionTypeRegistry::new();

        let connection =
            build_connection(
                Type::Grid,
                0,
                &inputs,
                &mut function_types,
                &mut rng,
            )
            .expect("expected a Grid input");

        assert!(matches!(
            connection,
            Connection::Input { name, ty }
                if name == "I" && ty == Type::Grid
        ));
    }

    #[test]
    fn build_connection_selects_primitive_above_depth_zero() {
        let mut rng = rand::rng();

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types =
            FunctionTypeRegistry::new();

        let connection =
            build_connection(
                Type::Grid,
                1,
                &inputs,
                &mut function_types,
                &mut rng,
            )
            .expect("expected a Grid primitive");

        assert!(matches!(
            connection,
            Connection::Primitive { .. }
        ));
    }

    #[test]
    fn build_connection_is_reproducible_with_seeded_rng() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng1 = rand::rngs::StdRng::seed_from_u64(42);
        let mut rng2 = rand::rngs::StdRng::seed_from_u64(42);

        let mut function_types1 =
            FunctionTypeRegistry::new();

        let mut function_types2 =
            FunctionTypeRegistry::new();

        let connection1 = build_connection(
            Type::Grid,
            2,
            &inputs,
            &mut function_types1,
            &mut rng1,
        )
        .expect("expected a connection");

        let connection2 = build_connection(
            Type::Grid,
            2,
            &inputs,
            &mut function_types2,
            &mut rng2,
        )
        .expect("expected a connection");

        assert_eq!(
            connection1.expression(),
            connection2.expression()
        );
    }

    fn connection_depth(connection: &Connection) -> usize {
        match connection {
            Connection::Input { .. }
            | Connection::Constant { .. } => 0,

            Connection::Primitive { inputs, .. }
            | Connection::Dynamic { inputs, .. } => {
                1 + inputs
                    .iter()
                    .map(|input| connection_depth(input))
                    .max()
                    .unwrap_or(0)
            }
        }
    }

    #[test]
    fn build_connection_respects_requested_depth() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let mut function_types =
            FunctionTypeRegistry::new();

        let connection = build_connection(
            Type::Grid,
            3,
            &inputs,
            &mut function_types,
            &mut rng,
        )
        .expect("expected a connection");

        assert!(connection_depth(&connection) <= 3);
    }

    #[test]
    fn lbind_candidates_find_source_function_type() {
        let mut function_types = FunctionTypeRegistry::new();

        let source_type = function_types.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let Type::Function(source_id) = source_type else {
            panic!("expected function type");
        };

        let output_type = function_types.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        let candidates =
            lbind_candidates(
                output_type,
                &function_types,
            );

        assert_eq!(
            candidates,
            vec![(source_id, Type::Integer)]
        );
    }

    fn rbind_candidates(
        output_type: Type,
        function_types: &FunctionTypeRegistry,
    ) -> Vec<(FunctionTypeId, Type)> {
        let Type::Function(output_id) = output_type else {
            return Vec::new();
        };

        let Some(output_inputs) =
            function_types.inputs(output_id)
        else {
            return Vec::new();
        };

        let Some(output_output) =
            function_types.output(output_id)
        else {
            return Vec::new();
        };

        function_types
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
                    source_id,
                    source_type.inputs[last],
                ))
            })
            .collect()
    }

    #[test]
    fn rbind_candidates_find_source_function_type() {
        let mut function_types = FunctionTypeRegistry::new();

        let source_type = function_types.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let Type::Function(source_id) = source_type else {
            panic!("expected function type");
        };

        let output_type = function_types.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        let candidates =
            rbind_candidates(
                output_type,
                &function_types,
            );

        assert_eq!(
            candidates,
            vec![(source_id, Type::Integer)]
        );
    }

    #[test]
    fn rbind_candidates_handle_multi_argument_functions() {
        let mut function_types = FunctionTypeRegistry::new();

        let source_type = function_types.type_of(
            &[
                Type::Integer,
                Type::Grid,
                Type::IntegerTuple,
            ],
            Type::Object,
        );

        let Type::Function(source_id) = source_type else {
            panic!("expected function type");
        };

        let output_type = function_types.type_of(
            &[Type::Integer, Type::Grid],
            Type::Object,
        );

        let candidates =
            rbind_candidates(
                output_type,
                &function_types,
            );

        assert_eq!(
            candidates,
            vec![(source_id, Type::IntegerTuple)]
        );
    }

    #[test]
    fn apply_candidates_find_functions_with_requested_output() {
        let mut function_types = FunctionTypeRegistry::new();

        let integer_function =
            function_types.type_of(
                &[Type::Integer],
                Type::Integer,
            );

        let tuple_function =
            function_types.type_of(
                &[Type::IntegerTuple],
                Type::Integer,
            );

        let Type::Function(integer_id) = integer_function else {
            panic!("expected function type");
        };

        let Type::Function(tuple_id) = tuple_function else {
            panic!("expected function type");
        };

        let candidates =
            apply_candidates(
                Type::Integer,
                &function_types,
            );

        assert_eq!(
            candidates,
            vec![integer_id, tuple_id]
        );
    }

    #[test]
    fn lbind_candidates_handle_multi_argument_functions() {
        let mut function_types = FunctionTypeRegistry::new();

        let source_type = function_types.type_of(
            &[Type::Integer, Type::Grid, Type::IntegerTuple],
            Type::Object,
        );

        let Type::Function(source_id) = source_type else {
            panic!("expected function type");
        };

        let output_type = function_types.type_of(
            &[Type::Grid, Type::IntegerTuple],
            Type::Object,
        );

        let candidates =
            lbind_candidates(
                output_type,
                &function_types,
            );

        assert_eq!(
            candidates,
            vec![(source_id, Type::Integer)]
        );
    }

    #[test]
    fn build_connection_can_build_lbind() {
        let mut function_types =
            FunctionTypeRegistry::new();

        let source_type = function_types.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let output_type = function_types.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        let inputs = vec![
            InputSpec {
                name: "F".to_string(),
                ty: source_type,
            },
            InputSpec {
                name: "I".to_string(),
                ty: Type::Integer,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(0);

        let connection = build_connection(
            output_type,
            1,
            &inputs,
            &mut function_types,
            &mut rng,
        )
        .expect("expected lbind connection");

        assert!(matches!(
            connection,
            Connection::Dynamic { .. }
        ));
    }

    #[test]
    fn build_connection_can_build_rbind() {
        let mut function_types =
            FunctionTypeRegistry::new();

        let source_type = function_types.type_of(
            &[
                Type::Integer,
                Type::Grid,
                Type::IntegerTuple,
            ],
            Type::Object,
        );

        let output_type = function_types.type_of(
            &[Type::Integer, Type::Grid],
            Type::Object,
        );

        let inputs = vec![
            InputSpec {
                name: "F".to_string(),
                ty: source_type,
            },
            InputSpec {
                name: "T".to_string(),
                ty: Type::IntegerTuple,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(0);

        let connection = build_connection(
            output_type,
            1,
            &inputs,
            &mut function_types,
            &mut rng,
        )
        .expect("expected rbind connection");

        assert!(matches!(
            connection,
            Connection::Dynamic {
                primitive: DynamicPrimitive::Rbind,
                ..
            }
        ));
    }

    #[test]
    fn build_connection_can_build_apply() {
        let mut function_types =
            FunctionTypeRegistry::new();

        let function_type = function_types.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let inputs = vec![
            InputSpec {
                name: "F".to_string(),
                ty: function_type,
            },
            InputSpec {
                name: "A".to_string(),
                ty: Type::Integer,
            },
            InputSpec {
                name: "B".to_string(),
                ty: Type::Integer,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(0);

        let connection = build_connection(
            Type::Integer,
            1,
            &inputs,
            &mut function_types,
            &mut rng,
        )
        .expect("expected apply connection");

        assert_eq!(
            connection.output_type_with(&mut function_types),
            Type::Integer
        );
    }

    #[test]
    fn candidate_groups_collect_dynamic_and_primitive_families() {
        let mut function_types =
            FunctionTypeRegistry::new();

        let function_type =
            function_types.type_of(
                &[Type::Integer, Type::Integer],
                Type::Integer,
            );

        let output_type = Type::Integer;

        let groups =
            candidate_groups(
                output_type,
                &function_types,
            );

        assert!(groups.iter().any(|group| {
            matches!(
                group,
                CandidateGroup::Dynamic {
                    ..
                }
            )
        }));

        assert!(groups.iter().any(|group| {
            matches!(
                group,
                CandidateGroup::Primitive { .. }
            )
        }));

        let _ = function_type;
    }

    #[test]
    fn shuffle_candidate_groups_preserves_candidates() {
        let mut function_types =
            FunctionTypeRegistry::new();

        let output_type =
            function_types.type_of(
                &[Type::Integer],
                Type::Integer,
            );

        let mut groups =
            candidate_groups(
                output_type,
                &function_types,
            );

        let before = groups
            .iter()
            .map(|group| match group {
                CandidateGroup::Dynamic {
                    candidates, ..
                } => candidates.len(),

                CandidateGroup::Primitive {
                    candidates, ..
                } => candidates.len(),
            })
            .collect::<Vec<_>>();

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(0);

        shuffle_candidate_groups(
            &mut groups,
            &mut rng,
        );

        let after = groups
            .iter()
            .map(|group| match group {
                CandidateGroup::Dynamic {
                    candidates, ..
                } => candidates.len(),

                CandidateGroup::Primitive {
                    candidates, ..
                } => candidates.len(),
            })
            .collect::<Vec<_>>();

        let mut before = before;
        let mut after = after;

        before.sort_unstable();
        after.sort_unstable();

        assert_eq!(before, after);
    }

    #[test]
    fn candidate_group_has_family_name() {
        let mut function_types = FunctionTypeRegistry::new();

        let source_type = function_types.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let function_id = match source_type {
            Type::Function(id) => id,
            _ => unreachable!(),
        };

        let group = CandidateGroup::Dynamic {
            candidates: vec![
                DynamicCandidate::Lbind(
                    function_id,
                    Type::Integer,
                ),
            ],
        };

        assert_eq!(group.name(), "lbind");
        assert_eq!(group.len(), 1);
    }

    #[test]
    fn candidate_groups_include_lbind_and_rbind() {
        let mut function_types =
            FunctionTypeRegistry::new();

        let _source_type =
            function_types.type_of(
                &[Type::Integer, Type::Integer],
                Type::Integer,
            );

        let output_type =
            function_types.type_of(
                &[Type::Integer],
                Type::Integer,
            );

        function_types.type_of(
            &[Type::Integer],
            output_type,
        );

        let groups =
            candidate_groups(
                output_type,
                &function_types,
            );

        let lbind =
            groups.iter().find(|group| {
                group.name() == "lbind"
            });

        let rbind =
            groups.iter().find(|group| {
                group.name() == "rbind"
            });

        assert_eq!(
            lbind.map(CandidateGroup::len),
            Some(1)
        );

        assert_eq!(
            rbind.map(CandidateGroup::len),
            Some(1)
        );

        let apply =
            groups.iter().find(|group| {
                group.name() == "apply"
            });

        assert_eq!(
            apply.map(CandidateGroup::len),
            Some(1)
        );
    }

    #[test]
    fn lbind_candidates_reject_mismatched_input_types() {
        let mut function_types = FunctionTypeRegistry::new();

        function_types.type_of(
            &[Type::Integer, Type::Grid],
            Type::Object,
        );

        let output_type = function_types.type_of(
            &[Type::IntegerTuple],
            Type::Object,
        );

        let candidates =
            lbind_candidates(
                output_type,
                &function_types,
            );

        assert!(candidates.is_empty());
    }

    #[test]
    fn rbind_candidates_reject_mismatched_input_types() {
        let mut function_types = FunctionTypeRegistry::new();

        function_types.type_of(
            &[Type::Integer, Type::Grid],
            Type::Object,
        );

        let output_type = function_types.type_of(
            &[Type::IntegerTuple],
            Type::Object,
        );

        let candidates =
            rbind_candidates(
                output_type,
                &function_types,
            );

        assert!(candidates.is_empty());
    }

    #[test]
    fn apply_candidates_reject_functions_with_other_output() {
        let mut function_types = FunctionTypeRegistry::new();

        let integer_function =
            function_types.type_of(
                &[Type::Integer],
                Type::Integer,
            );

        let _grid_function =
            function_types.type_of(
                &[Type::Integer],
                Type::Grid,
            );

        let Type::Function(integer_id) =
            integer_function
        else {
            panic!("expected function type");
        };

        let candidates =
            apply_candidates(
                Type::Integer,
                &function_types,
            );

        assert_eq!(
            candidates,
            vec![integer_id]
        );
    }

    #[test]
    fn generate_structured_grid_respects_requested_size_range() {
        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let (grid, _) = generate_structured_grid(
            (4, 4),
            (30, 30),
            &mut rng,
        );

        assert!((4..=30).contains(&grid.len()));
        assert!((4..=30).contains(&grid[0].len()));
    }

    #[test]
    fn random_grid_kind_returns_known_kind() {
        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let kind = random_grid_kind(&mut rng);

        assert!([
            "stripes",
            "blocks",
            "pattern",
            "gradient",
            "sparse",
            "random",
        ]
        .contains(&kind));
    }

    #[test]
    fn generate_structured_grid_produces_valid_grids() {
        let mut rng =
            rand::rngs::StdRng::seed_from_u64(123);

        for _ in 0..100 {
            let (grid, kind) = generate_structured_grid(
                (4, 4),
                (30, 30),
                &mut rng,
            );

            assert!(!grid.is_empty(), "kind: {kind}");
            assert!((4..=30).contains(&grid.len()));

            let width = grid[0].len();
            assert!((4..=30).contains(&width));

            assert!(
                grid.iter().all(|row| row.len() == width),
                "kind: {kind}"
            );

            assert!(
                grid.iter()
                    .flatten()
                    .all(|&value| (0..=9).contains(&value)),
                "kind: {kind}"
            );
        }
    }

    #[test]
    fn valid_grid_output_is_accepted() {
        let input = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        let output = vec![
            vec![4, 3],
            vec![2, 1],
        ];

        assert!(is_valid_grid_output(&input, &output));
    }

    #[test]
    fn identical_grid_output_is_rejected() {
        let input = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        assert!(!is_valid_grid_output(&input, &input));
    }

    #[test]
    fn empty_grid_output_is_rejected() {
        let input = vec![vec![1]];

        assert!(!is_valid_grid_output(
            &input,
            &Vec::new(),
        ));
    }

    #[test]
    fn out_of_range_grid_output_is_rejected() {
        let input = vec![vec![1]];

        let output = vec![
            vec![10],
        ];

        assert!(!is_valid_grid_output(
            &input,
            &output,
        ));
    }

    #[test]
    fn evaluate_grid_connection_returns_input_grid() {
        let connection =
            Connection::input("I", Type::Grid);

        let input = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        let output =
            evaluate_grid_connection(
                &connection,
                input.clone(),
            )
            .expect("expected Grid output");

        assert_eq!(output, input);
    }

    #[test]
    fn input_connection_is_rejected_as_identity() {
        let connection =
            Connection::input("I", Type::Grid);

        let input = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        assert!(!connection_produces_valid_grid(
            &connection,
            input,
        ));
    }

    #[test]
    fn transforming_connection_is_accepted() {
        let primitive =
            crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| {
                    primitive.name == "hmirror"
                        && primitive.inputs == &[Type::Grid]
                        && primitive.output == Type::Grid
                })
                .expect("expected hmirror");

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
        )
        .expect("expected valid connection");

        let input = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        assert!(connection_produces_valid_grid(
            &connection,
            input,
        ));
    }

    #[test]
    fn invalid_output_is_rejected() {
        let connection =
            Connection::terminal(Value::Grid(vec![
                vec![10],
            ]));

        let input = vec![vec![1]];

        assert!(!connection_produces_valid_grid(
            &connection,
            input,
        ));
    }

    #[test]
    fn connection_passes_grid_trials_when_it_can_transform_input() {
        use rand::SeedableRng;

        let primitive =
            crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| {
                    primitive.name == "hmirror"
                        && primitive.inputs == &[Type::Grid]
                        && primitive.output == Type::Grid
                })
                .expect("expected hmirror");

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
        )
        .expect("expected valid connection");

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        assert!(connection_passes_grid_trials(
            &connection,
            100,
            &mut rng,
        ));
    }

    #[test]
    fn connection_using_i_is_detected() {
        let connection =
            Connection::input("I", Type::Grid);

        assert!(uses_input_i(&connection));
    }

    #[test]
    fn connection_using_right_is_not_mistaken_for_i() {
        let connection =
            Connection::terminal(Value::IntegerTuple(
                (1, 0),
            ));

        assert!(!uses_input_i(&connection));
    }

    #[test]
    fn connection_without_i_is_rejected() {
        let connection =
            Connection::terminal(Value::Grid(vec![
                vec![1],
            ]));

        assert!(!uses_input_i(&connection));
    }

    #[test]
    fn connection_using_i_and_transforming_grid_is_acceptable() {
        use rand::SeedableRng;

        let primitive =
            crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| {
                    primitive.name == "hmirror"
                        && primitive.inputs == &[Type::Grid]
                        && primitive.output == Type::Grid
                })
                .expect("expected hmirror");

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
        )
        .expect("expected valid connection");

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        assert!(connection_is_acceptable(
            &connection,
            100,
            &mut rng,
        ));
    }

    #[test]
    fn connection_without_i_is_not_acceptable() {
        use rand::SeedableRng;

        let connection =
            Connection::terminal(Value::Grid(vec![
                vec![1],
            ]));

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        assert!(!connection_is_acceptable(
            &connection,
            100,
            &mut rng,
        ));
    }

    #[test]
    fn acceptable_connections_filters_connections() {
        use rand::SeedableRng;

        let primitive =
            crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| {
                    primitive.name == "hmirror"
                        && primitive.inputs == &[Type::Grid]
                        && primitive.output == Type::Grid
                })
                .expect("expected hmirror");

        let transforming = Connection::new(
            primitive,
            vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
        )
        .expect("expected valid connection");

        let constant =
            Connection::terminal(Value::Grid(vec![
                vec![1],
            ]));

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let connections =
            acceptable_connections(
                vec![transforming, constant],
                100,
                &mut rng,
            );

        assert_eq!(connections.len(), 1);
        assert!(uses_input_i(&connections[0]));
    }

    #[test]
    fn duplicate_connections_are_removed() {
        let connection =
            Connection::input("I", Type::Grid);

        let duplicate =
            Connection::input("I", Type::Grid);

        let other =
            Connection::input("J", Type::Grid);

        let result =
            deduplicate_connections(
                vec![connection, duplicate, other],
            );

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].expression(), "I");
        assert_eq!(result[1].expression(), "J");
    }

    #[test]
    fn generate_connections_produces_requested_number_when_possible() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types =
            FunctionTypeRegistry::new();

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let connections =
            generate_connections(
                1,
                10,
                &inputs,
                &mut function_types,
                &mut rng,
            );

        assert_eq!(connections.len(), 10);
        assert!(
            connections
                .iter()
                .all(|connection| {
                    connection.output_type()
                        == Type::Grid
                })
        );
    }

    #[test]
    fn generate_acceptable_connections_returns_only_acceptable_programs() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types =
            FunctionTypeRegistry::new();

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let connections =
            generate_acceptable_connections(
                1,
                20,
                &inputs,
                &mut function_types,
                &mut rng,
                100,
            );

        assert!(
            connections
                .iter()
                .all(|connection| {
                    uses_input_i(connection)
                })
        );

        assert!(
            connections
                .iter()
                .all(|connection| {
                    connection.output_type()
                        == Type::Grid
                })
        );
    }

    #[test]
    fn panicking_connection_is_rejected() {
        let primitive =
            crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| {
                    primitive.name == "hmirror"
                        && primitive.inputs == &[Type::Grid]
                        && primitive.output == Type::Grid
                })
                .expect("expected hmirror");

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
        )
        .expect("expected valid connection");

        let input = vec![
            vec![1, 2, 2, 1],
            vec![3, 4, 4, 3],
        ];

        assert!(
            !connection_produces_valid_grid(
                &connection,
                input,
            )
        );
    }

    #[test]
    fn generate_acceptable_connections_are_unique() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types =
            FunctionTypeRegistry::new();

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let connections =
            generate_acceptable_connections(
                2,
                100,
                &inputs,
                &mut function_types,
                &mut rng,
                100,
            );

        let expressions = connections
            .iter()
            .map(Connection::expression)
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            expressions.len(),
            connections.len()
        );
    }

    #[test]
    fn corpus_entry_can_be_added() {
        let mut corpus = Vec::new();

        let connection =
            Connection::input("I", Type::Grid);

        let input = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        add_corpus_entry(
            &mut corpus,
            connection,
            input.clone(),
        );

        assert_eq!(corpus.len(), 1);
        assert_eq!(
            corpus[0].input,
            input
        );
        assert_eq!(
            corpus[0].connection.expression(),
            "I"
        );
    }

    #[test]
    fn find_valid_grid_returns_input_for_transforming_connection() {
        use rand::SeedableRng;

        let primitive =
            crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| {
                    primitive.name == "hmirror"
                        && primitive.inputs == &[Type::Grid]
                        && primitive.output == Type::Grid
                })
                .expect("expected hmirror");

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
        )
        .expect("expected valid connection");

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let input =
            find_valid_grid(
                &connection,
                100,
                &mut rng,
            );

        assert!(input.is_some());
    }

    #[test]
    fn build_corpus_entry_keeps_valid_input() {
        use rand::SeedableRng;

        let primitive =
            crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| {
                    primitive.name == "hmirror"
                        && primitive.inputs == &[Type::Grid]
                        && primitive.output == Type::Grid
                })
                .expect("expected hmirror");

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
        )
        .expect("expected valid connection");

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let entry =
            build_corpus_entry(
                connection,
                100,
                &mut rng,
            )
            .expect("expected corpus entry");

        assert!(
            is_valid_grid_output(
                &entry.input,
                &evaluate_grid_connection(
                    &entry.connection,
                    entry.input.clone(),
                )
                .expect("expected Grid output"),
            )
        );
    }

    #[test]
    fn generate_corpus_entries_produces_valid_entries() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types =
            FunctionTypeRegistry::new();

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let entries =
            generate_corpus_entries(
                1,
                20,
                &inputs,
                &mut function_types,
                &mut rng,
                100,
            );

        assert!(
            entries.iter().all(|entry| {
                uses_input_i(&entry.connection)
                    && entry.connection.output_type()
                        == Type::Grid
                    && is_valid_grid_output(
                        &entry.input,
                        &evaluate_grid_connection(
                            &entry.connection,
                            entry.input.clone(),
                        )
                        .expect("expected Grid output"),
                    )
            })
        );
    }

    #[test]
    fn duplicate_corpus_entries_are_removed() {
        let connection =
            Connection::input("I", Type::Grid);

        let input = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        let entries = vec![
            CorpusEntry {
                input: input.clone(),
                connection: connection.clone(),
            },
            CorpusEntry {
                input,
                connection,
            },
        ];

        let result =
            deduplicate_corpus_entries(entries);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn generate_corpus_reaches_requested_count() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let corpus =
            generate_corpus(
                1,
                10,
                &inputs,
                &mut rng,
                100,
                100,
            )
            .expect("expected corpus generation to succeed");

        assert_eq!(corpus.len(), 10);
    }

    #[test]
    fn generate_corpus_fails_when_attempt_limit_is_zero() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let result =
            generate_corpus(
                1,
                10,
                &inputs,
                &mut rng,
                100,
                0,
            );

        assert!(result.is_err());
    }

    #[test]
    fn generated_corpus_entries_are_unique() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let corpus =
            generate_corpus(
                2,
                20,
                &inputs,
                &mut rng,
                100,
                100,
            )
            .expect("expected corpus generation to succeed");

        let keys = corpus
            .iter()
            .map(|entry| {
                (
                    entry.input.clone(),
                    entry.connection.expression(),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(keys.len(), corpus.len());
    }

    #[test]
    fn generate_corpus_entry_is_reproducible_with_seed() {
        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types_a =
            FunctionTypeRegistry::new();

        let mut function_types_b =
            FunctionTypeRegistry::new();

        let a =
            generate_corpus_entry(
                2,
                &inputs,
                &mut function_types_a,
                42,
                100,
            );

        let b =
            generate_corpus_entry(
                2,
                &inputs,
                &mut function_types_b,
                42,
                100,
            );

        assert_eq!(
            a.as_ref()
                .map(|entry| entry.connection.expression()),
            b.as_ref()
                .map(|entry| entry.connection.expression()),
        );
    }

    #[test]
    fn generate_seeds_is_reproducible() {
        use rand::SeedableRng;

        let mut rng_a =
            rand::rngs::StdRng::seed_from_u64(42);

        let mut rng_b =
            rand::rngs::StdRng::seed_from_u64(42);

        let a =
            generate_seeds(10, &mut rng_a);

        let b =
            generate_seeds(10, &mut rng_b);

        assert_eq!(a, b);
    }

    #[test]
    fn parallel_generation_is_reproducible_for_same_seeds() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut seed_rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let seeds =
            generate_seeds(20, &mut seed_rng);

        let parallel =
            generate_corpus_entries_parallel(
                2,
                &seeds,
                &inputs,
                100,
            );

        let sequential =
            seeds
                .iter()
                .filter_map(|&seed| {
                    let mut function_types =
                        FunctionTypeRegistry::new();

                    generate_corpus_entry(
                        2,
                        &inputs,
                        &mut function_types,
                        seed,
                        100,
                    )
                })
                .collect::<Vec<_>>();

        let parallel_expressions =
            parallel
                .iter()
                .map(|entry| entry.connection.expression())
                .collect::<Vec<_>>();

        let sequential_expressions =
            sequential
                .iter()
                .map(|entry| entry.connection.expression())
                .collect::<Vec<_>>();

        assert_eq!(
            parallel_expressions,
            sequential_expressions
        );
    }

    #[test]
    fn generate_corpus_parallel_reaches_requested_count() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let corpus =
            generate_corpus_parallel(
                2,
                10,
                &inputs,
                &mut rng,
                100,
                100,
            )
            .expect("expected corpus generation to succeed");

        assert_eq!(corpus.len(), 10);
    }

    #[test]
    fn parallel_corpus_entries_are_valid_and_unique() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let corpus =
            generate_corpus_parallel(
                2,
                20,
                &inputs,
                &mut rng,
                100,
                100,
            )
            .expect("expected corpus generation to succeed");

        assert_eq!(corpus.len(), 20);

        assert!(
            corpus.iter().all(|entry| {
                uses_input_i(&entry.connection)
                    && entry.connection.output_type()
                        == Type::Grid
                    && is_valid_grid_output(
                        &entry.input,
                        &evaluate_grid_connection(
                            &entry.connection,
                            entry.input.clone(),
                        )
                        .expect("expected Grid output"),
                    )
            })
        );

        let keys = corpus
            .iter()
            .map(|entry| {
                (
                    entry.input.clone(),
                    entry.connection.expression(),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            keys.len(),
            corpus.len()
        );
    }

    #[test]
    fn parallel_corpus_is_reproducible() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng_a =
            rand::rngs::StdRng::seed_from_u64(42);

        let mut rng_b =
            rand::rngs::StdRng::seed_from_u64(42);

        let corpus_a =
            generate_corpus_parallel(
                2,
                20,
                &inputs,
                &mut rng_a,
                100,
                100,
            )
            .expect("expected corpus generation to succeed");

        let corpus_b =
            generate_corpus_parallel(
                2,
                20,
                &inputs,
                &mut rng_b,
                100,
                100,
            )
            .expect("expected corpus generation to succeed");

        let key = |entry: &CorpusEntry| {
            (
                entry.input.clone(),
                entry.connection.expression(),
            )
        };

        let mut keys_a =
            corpus_a.iter().map(key).collect::<Vec<_>>();

        let mut keys_b =
            corpus_b.iter().map(key).collect::<Vec<_>>();

        keys_a.sort();
        keys_b.sort();

        assert_eq!(keys_a, keys_b);
    }

    #[test]
    fn corpus_seeds_are_reproducible() {
        use rand::SeedableRng;

        let mut rng_a =
            rand::rngs::StdRng::seed_from_u64(42);

        let mut rng_b =
            rand::rngs::StdRng::seed_from_u64(42);

        assert_eq!(
            corpus_seeds(20, &mut rng_a),
            corpus_seeds(20, &mut rng_b),
        );
    }

    #[test]
    fn seeded_corpus_entry_wrapper_matches_direct_generation() {
        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let seed = 42;

        let mut function_types =
            FunctionTypeRegistry::new();

        let direct =
            generate_corpus_entry(
                2,
                &inputs,
                &mut function_types,
                seed,
                100,
            );

        let wrapped =
            generate_corpus_entry_with_seed(
                2,
                &inputs,
                seed,
                100,
            );

        assert_eq!(
            direct.as_ref()
                .map(|entry| entry.connection.expression()),
            wrapped.as_ref()
                .map(|entry| entry.connection.expression()),
        );
    }

    #[test]
    fn sequential_and_parallel_generation_match() {
        use rand::SeedableRng;

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut seed_rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let seeds =
            corpus_seeds(100, &mut seed_rng);

        let sequential =
            seeds
                .iter()
                .filter_map(|&seed| {
                    generate_corpus_entry_with_seed(
                        2,
                        &inputs,
                        seed,
                        100,
                    )
                })
                .collect::<Vec<_>>();

        let parallel =
            generate_corpus_entries_parallel(
                2,
                &seeds,
                &inputs,
                100,
            );

        let mut sequential_keys =
            sequential
                .iter()
                .map(corpus_entry_key)
                .collect::<Vec<_>>();

        let mut parallel_keys =
            parallel
                .iter()
                .map(corpus_entry_key)
                .collect::<Vec<_>>();

        sequential_keys.sort();
        parallel_keys.sort();

        assert_eq!(
            sequential_keys,
            parallel_keys
        );
    }

    #[test]
    fn select_corpus_entries_respects_count() {
        let connection =
            Connection::input("I", Type::Grid);

        let entries = (0..10)
            .map(|value| CorpusEntry {
                input: vec![
                    vec![value],
                ],
                connection: connection.clone(),
            })
            .collect::<Vec<_>>();

        let selected =
            select_corpus_entries(
                entries,
                5,
            )
            .expect("expected selection to succeed");

        assert_eq!(selected.len(), 5);
    }

    #[test]
    fn select_corpus_entries_fails_when_count_is_unreachable() {
        let connection =
            Connection::input("I", Type::Grid);

        let entries = vec![
            CorpusEntry {
                input: vec![vec![1]],
                connection,
            },
        ];

        let result =
            select_corpus_entries(
                entries,
                2,
            );

        assert!(result.is_err());
    }

    #[test]
    fn select_corpus_entries_deduplicates_before_count() {
        let input = vec![vec![1, 0], vec![0, 1]];

        let connection =
            Connection::input(
                "I".to_string(),
                Type::Grid,
            );

        let entries = vec![
            CorpusEntry {
                input: input.clone(),
                connection: connection.clone(),
            },
            CorpusEntry {
                input,
                connection,
            },
        ];

        let selected =
            select_corpus_entries(
                entries,
                1,
            )
            .unwrap();

        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn parallel_corpus_counts_unique_entries_across_batches() {
        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let corpus =
            generate_corpus_parallel(
                2,
                2,
                &inputs,
                &mut rng,
                10,
                100,
            )
            .unwrap();

        assert_eq!(corpus.len(), 2);

        let keys = corpus
            .iter()
            .map(corpus_entry_key)
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(keys.len(), corpus.len());
    }

    #[test]
    fn generate_corpus_parallel_with_zero_count_returns_empty() {
        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let corpus =
            generate_corpus_parallel(
                2,
                0,
                &inputs,
                &mut rng,
                10,
                100,
            )
            .unwrap();

        assert!(corpus.is_empty());
    }

    #[test]
    fn generate_corpus_with_zero_count_returns_empty() {
        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let corpus =
            generate_corpus(
                2,
                0,
                &inputs,
                &mut rng,
                10,
                100,
            )
            .unwrap();

        assert!(corpus.is_empty());
    }

    #[test]
    fn generate_corpus_parallel_fails_when_attempt_limit_is_zero() {
        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let result =
            generate_corpus_parallel(
                2,
                1,
                &inputs,
                &mut rng,
                10,
                0,
            );

        assert!(result.is_err());
    }

    #[test]
    fn corpus_batching_respects_max_attempts() {
        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let result =
            generate_corpus_parallel(
                2,
                10,
                &inputs,
                &mut rng,
                10,
                1,
            );

        assert!(result.is_err());
    }

    #[test]
    fn corpus_batching_handles_partial_batch() {
        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut rng =
            rand::rngs::StdRng::seed_from_u64(42);

        let result =
            generate_corpus_parallel(
                2,
                10,
                &inputs,
                &mut rng,
                10,
                7,
            );

        assert!(result.is_err());
    }
}
