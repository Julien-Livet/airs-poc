use crate::types::*;
use primitive_macro::primitive;

const SHOOT_DISTANCE: usize = 42;
const MAX_SIZE: usize = 30;
use std::collections::BTreeMap;

pub type IntegerCountMap = BTreeMap<Integer, Integer>;

#[primitive("ofcolor")]
pub fn ofcolor(grid: Grid, value: Integer) -> Indices {
    if grid.is_empty() {
        panic!("ofcolor received empty grid");
    }

    let width = grid[0].len();

    if width == 0 || grid.iter().any(|row| row.len() != width) {
        panic!("ofcolor received wrong grid");
    }

    let mut indices = Indices::new();

    for (i, row) in grid.iter().enumerate() {
        for (j, &cell) in row.iter().enumerate() {
            if cell == value {
                indices.insert((i as Integer, j as Integer));
            }
        }
    }

    indices
}

#[primitive("vfrontier")]
pub fn vfrontier(location: IntegerTuple) -> Indices {
    let (_, j) = location;
    (0..MAX_SIZE)
        .map(|i| (i as Integer, j))
        .collect()
}

#[primitive("hfrontier")]
pub fn hfrontier(location: IntegerTuple) -> Indices {
    let (i, _) = location;
    (0..MAX_SIZE)
        .map(|j| (i, j as Integer))
        .collect()
}

fn backdrop_from_indices(patch: Indices) -> Indices {
    if patch.is_empty() {
        panic!("backdrop received empty patch");
    }

    let (ui, uj) = ulcorner(patch.clone());
    let (li, lj) = lrcorner(patch);

    let mut result = Indices::new();

    for i in ui..=li {
        for j in uj..=lj {
            result.insert((i, j));
        }
    }

    result
}

#[primitive("backdrop")]
pub fn backdrop_object(object: Object) -> Indices {
    backdrop_from_indices(toindices_object(object))
}

#[primitive("backdrop")]
pub fn backdrop_indices(indices: Indices) -> Indices {
    backdrop_from_indices(indices)
}

fn delta_from_indices(patch: Indices) -> Indices {
    if patch.is_empty() {
        panic!("delta received empty patch");
    }

    let mut result = backdrop_from_indices(patch.clone());

    for index in patch {
        result.remove(&index);
    }

    result
}

#[primitive("delta")]
pub fn delta_object(object: Object) -> Indices {
    delta_from_indices(toindices_object(object))
}

#[primitive("delta")]
pub fn delta_indices(indices: Indices) -> Indices {
    delta_from_indices(indices)
}

fn gravitate_from_indices(
    source: Indices,
    destination: Indices,
) -> IntegerTuple {
    let (si, sj) = center_indices(source.clone());
    let (di, dj) = center_indices(destination.clone());

    let (step_i, step_j) = if vmatching_indices_indices(
        source.clone(),
        destination.clone(),
    ) {
        (if si < di { 1 } else { -1 }, 0)
    } else {
        (0, if sj < dj { 1 } else { -1 })
    };

    let mut current = source;
    let mut move_i = step_i;
    let mut move_j = step_j;
    let mut count = 0;

    while !adjacent_indices_indices(
        current.clone(),
        destination.clone(),
    ) && count < 42
    {
        count += 1;

        move_i += step_i;
        move_j += step_j;

        current = shift_indices(
            current,
            (step_i, step_j),
        );
    }

    (move_i - step_i, move_j - step_j)
}

#[primitive("gravitate")]
pub fn gravitate_object_object(
    source: Object,
    destination: Object,
) -> IntegerTuple {
    gravitate_from_indices(
        toindices_object(source),
        toindices_object(destination),
    )
}

#[primitive("gravitate")]
pub fn gravitate_object_indices(
    source: Object,
    destination: Indices,
) -> IntegerTuple {
    gravitate_from_indices(
        toindices_object(source),
        destination,
    )
}

#[primitive("gravitate")]
pub fn gravitate_indices_object(
    source: Indices,
    destination: Object,
) -> IntegerTuple {
    gravitate_from_indices(
        source,
        toindices_object(destination),
    )
}

#[primitive("gravitate")]
pub fn gravitate_indices_indices(
    source: Indices,
    destination: Indices,
) -> IntegerTuple {
    gravitate_from_indices(source, destination)
}

fn rectangle_outline(
    top: Integer,
    left: Integer,
    bottom: Integer,
    right: Integer,
) -> Indices {
    let mut result = Indices::new();

    if top > bottom || left > right {
        return result;
    }

    for j in left..=right {
        result.insert((top, j));
        result.insert((bottom, j));
    }

    for i in top..=bottom {
        result.insert((i, left));
        result.insert((i, right));
    }

    result
}

fn inbox_from_indices(patch: Indices) -> Indices {
    if patch.is_empty() {
        panic!("inbox received empty patch");
    }

    let ai = uppermost_indices(patch.clone()) + 1;
    let aj = leftmost_indices(patch.clone()) + 1;
    let bi = lowermost_indices(patch.clone()) - 1;
    let bj = rightmost_indices(patch);

    rectangle_outline(
        ai.min(bi),
        aj.min(bj),
        ai.max(bi),
        aj.max(bj),
    )
}

#[primitive("inbox")]
pub fn inbox_object(object: Object) -> Indices {
    inbox_from_indices(toindices_object(object))
}

#[primitive("inbox")]
pub fn inbox_indices(indices: Indices) -> Indices {
    inbox_from_indices(indices)
}

fn outbox_from_indices(patch: Indices) -> Indices {
    if patch.is_empty() {
        panic!("outbox received empty patch");
    }

    let ai = uppermost_indices(patch.clone()) + 1;
    let aj = leftmost_indices(patch.clone()) + 1;
    let bi = lowermost_indices(patch.clone()) - 1;
    let bj = rightmost_indices(patch);

    rectangle_outline(
        ai.min(bi),
        aj.min(bj),
        ai.max(bi),
        aj.max(bj),
    )
}

#[primitive("outbox")]
pub fn outbox_object(object: Object) -> Indices {
    outbox_from_indices(toindices_object(object))
}

#[primitive("outbox")]
pub fn outbox_indices(indices: Indices) -> Indices {
    outbox_from_indices(indices)
}

fn box_from_indices(patch: Indices) -> Indices {
    if patch.is_empty() {
        panic!("box received empty patch");
    }

    let (ai, aj) = ulcorner(patch.clone());
    let (bi, bj) = lrcorner(patch);

    rectangle_outline(
        ai.min(bi),
        aj.min(bj),
        ai.max(bi),
        aj.max(bj),
    )
}

#[primitive("box")]
pub fn box_object(object: Object) -> Indices {
    box_from_indices(toindices_object(object))
}

#[primitive("box")]
pub fn box_indices(indices: Indices) -> Indices {
    box_from_indices(indices)
}

#[primitive("shoot")]
pub fn shoot(
    start: IntegerTuple,
    direction: IntegerTuple,
) -> Indices {
    let (si, sj) = start;
    let (di, dj) = direction;

    if si.abs() > 100 || sj.abs() > 100 {
        panic!("shoot received wrong start");
    }

    if di * di + dj * dj > (2 * MAX_SIZE * MAX_SIZE).try_into().unwrap() {
        panic!("shoot received wrong direction");
    }

    let distance = SHOOT_DISTANCE as Integer;

    connect(
        start,
        (
            si + distance * di,
            sj + distance * dj,
        ),
    )
}

#[primitive("occurrences")]
pub fn occurrences(grid: Grid, object: Object) -> Indices {
    if grid.is_empty() || object.is_empty() {
        panic!("occurrences received empty grid or object");
    }

    let normalized = normalize_object(object.clone());

    let h = grid.len();
    let w = grid[0].len();

    if w == 0 || grid.iter().any(|row| row.len() != w) {
        panic!("occurrences received wrong grid");
    }

    let (oh, ow) = shape_object(object);

    let oh = oh as usize;
    let ow = ow as usize;

    if oh > h || ow > w {
        return Indices::new();
    }

    let mut result = Indices::new();

    for i in 0..=h - oh {
        for j in 0..=w - ow {
            let mut matches = true;

            for &(value, (oi, oj)) in &normalized {
                let a = (oi + i as Integer) as usize;
                let b = (oj + j as Integer) as usize;

                if grid[a][b] != value {
                    matches = false;
                    break;
                }
            }

            if matches {
                result.insert((i as Integer, j as Integer));
            }
        }
    }

    result
}

#[primitive("frontiers")]
pub fn frontiers(grid: Grid) -> Objects {
    if grid.is_empty() {
        panic!("frontiers received empty grid");
    }

    let h = grid.len();
    let w = grid[0].len();

    if w == 0 || grid.iter().any(|row| row.len() != w) {
        panic!("frontiers received wrong grid");
    }

    let mut result = Objects::new();

    // Uniform rows.
    for i in 0..h {
        let color = grid[i][0];

        if grid[i].iter().all(|&value| value == color) {
            let mut object = Object::new();

            for j in 0..w {
                object.insert((
                    grid[i][j],
                    (i as Integer, j as Integer),
                ));
            }

            result.insert(object);
        }
    }

    // Uniform columns.
    for j in 0..w {
        let color = grid[0][j];

        if (0..h).all(|i| grid[i][j] == color) {
            let mut object = Object::new();

            for i in 0..h {
                object.insert((
                    grid[i][j],
                    (i as Integer, j as Integer),
                ));
            }

            result.insert(object);
        }
    }

    result
}

#[primitive("compress")]
pub fn compress(grid: Grid) -> Grid {
    if grid.is_empty() {
        panic!("compress received empty grid");
    }

    let h = grid.len();
    let w = grid[0].len();

    if w == 0 || grid.iter().any(|row| row.len() != w) {
        panic!("compress received wrong grid");
    }

    let remove_rows: Vec<bool> = (0..h)
        .map(|i| {
            let color = grid[i][0];
            grid[i].iter().all(|&value| value == color)
        })
        .collect();

    let remove_cols: Vec<bool> = (0..w)
        .map(|j| {
            let color = grid[0][j];
            (0..h).all(|i| grid[i][j] == color)
        })
        .collect();

    let result: Grid = (0..h)
        .filter(|&i| !remove_rows[i])
        .map(|i| {
            (0..w)
                .filter(|&j| !remove_cols[j])
                .map(|j| grid[i][j])
                .collect()
        })
        .collect();

    if result == grid {
        panic!("compress produced identity");
    }

    result
}

#[primitive("move")]
pub fn move_object(
    grid: Grid,
    object: Object,
    offset: IntegerTuple,
) -> Grid {
    let covered = cover_object(grid, object.clone());
    let shifted = shift_object(object, offset);

    paint(covered, shifted)
}

#[primitive("subgrid")]
pub fn subgrid_object(patch: Object, grid: Grid) -> Grid {
    subgrid_from_indices(toindices_object(patch), grid)
}

#[primitive("subgrid")]
pub fn subgrid_indices(patch: Indices, grid: Grid) -> Grid {
    subgrid_from_indices(patch, grid)
}

fn subgrid_from_indices(patch: Indices, grid: Grid) -> Grid {
    let loc = ulcorner(patch.clone());
    let dimensions = shape_indices(patch);

    crop(grid, loc, dimensions)
}

fn hmatching_indices(a: Indices, b: Indices) -> Boolean {
    let rows: IntegerSet = a.into_iter().map(|(i, _)| i).collect();

    b.into_iter().any(|(i, _)| rows.contains(&i))
}

#[primitive("hmatching")]
pub fn hmatching_object_object(a: Object, b: Object) -> Boolean {
    hmatching_indices(toindices_object(a), toindices_object(b))
}

#[primitive("hmatching")]
pub fn hmatching_object_indices(a: Object, b: Indices) -> Boolean {
    hmatching_indices(toindices_object(a), b)
}

#[primitive("hmatching")]
pub fn hmatching_indices_object(a: Indices, b: Object) -> Boolean {
    hmatching_indices(a, toindices_object(b))
}

#[primitive("hmatching")]
pub fn hmatching_indices_indices(a: Indices, b: Indices) -> Boolean {
    hmatching_indices(a, b)
}

fn vmatching_indices(a: Indices, b: Indices) -> Boolean {
    let cols: IntegerSet = a.into_iter().map(|(_, j)| j).collect();

    b.into_iter().any(|(_, j)| cols.contains(&j))
}

#[primitive("vmatching")]
pub fn vmatching_object_object(a: Object, b: Object) -> Boolean {
    vmatching_indices(toindices_object(a), toindices_object(b))
}

#[primitive("vmatching")]
pub fn vmatching_object_indices(a: Object, b: Indices) -> Boolean {
    vmatching_indices(toindices_object(a), b)
}

#[primitive("vmatching")]
pub fn vmatching_indices_object(a: Indices, b: Object) -> Boolean {
    vmatching_indices(a, toindices_object(b))
}

#[primitive("vmatching")]
pub fn vmatching_indices_indices(a: Indices, b: Indices) -> Boolean {
    vmatching_indices(a, b)
}

fn manhattan_indices(a: Indices, b: Indices) -> Integer {
    let mut dmin = Integer::MAX;

    for (ai, aj) in a {
        for (bi, bj) in &b {
            let d = (ai - *bi).abs() + (aj - *bj).abs();
            dmin = dmin.min(d);
        }
    }

    dmin
}

#[primitive("manhattan")]
pub fn manhattan_object_object(a: Object, b: Object) -> Integer {
    manhattan_indices(toindices_object(a), toindices_object(b))
}

#[primitive("manhattan")]
pub fn manhattan_object_indices(a: Object, b: Indices) -> Integer {
    manhattan_indices(toindices_object(a), b)
}

#[primitive("manhattan")]
pub fn manhattan_indices_object(a: Indices, b: Object) -> Integer {
    manhattan_indices(a, toindices_object(b))
}

#[primitive("manhattan")]
pub fn manhattan_indices_indices(a: Indices, b: Indices) -> Integer {
    manhattan_indices(a, b)
}

#[primitive("adjacent")]
pub fn adjacent_object_object(a: Object, b: Object) -> Boolean {
    manhattan_object_object(a, b) == 1
}

#[primitive("adjacent")]
pub fn adjacent_object_indices(a: Object, b: Indices) -> Boolean {
    manhattan_object_indices(a, b) == 1
}

#[primitive("adjacent")]
pub fn adjacent_indices_object(a: Indices, b: Object) -> Boolean {
    manhattan_indices_object(a, b) == 1
}

#[primitive("adjacent")]
pub fn adjacent_indices_indices(a: Indices, b: Indices) -> Boolean {
    manhattan_indices_indices(a, b) == 1
}

fn bordering_from_indices(patch: Indices, grid: Grid) -> Boolean {
    if grid.is_empty() {
        panic!("bordering received empty grid");
    }

    let h = grid.len();
    let w = grid[0].len();

    if w == 0 || grid.iter().any(|row| row.len() != w) {
        panic!("bordering received wrong grid");
    }

    let um = uppermost_indices(patch.clone());
    let lm = leftmost_indices(patch.clone());
    let lrm = lowermost_indices(patch.clone());
    let rtm = rightmost_indices(patch);

    um == 0
        || lm == 0
        || lrm == h as Integer - 1
        || rtm == w as Integer - 1
}

#[primitive("bordering")]
pub fn bordering_object(patch: Object, grid: Grid) -> Boolean {
    bordering_from_indices(toindices_object(patch), grid)
}

#[primitive("bordering")]
pub fn bordering_indices(patch: Indices, grid: Grid) -> Boolean {
    bordering_from_indices(patch, grid)
}

fn centerofmass_from_indices(patch: Indices) -> IntegerTuple {
    let l = patch.len();

    if l == 0 {
        panic!("centerofmass received empty patch");
    }

    let (sum_i, sum_j) = patch.into_iter().fold(
        (0, 0),
        |(sum_i, sum_j), (i, j)| (sum_i + i, sum_j + j),
    );

    let l = l as Integer;

    (sum_i / l, sum_j / l)
}

#[primitive("centerofmass")]
pub fn centerofmass_object(object: Object) -> IntegerTuple {
    centerofmass_from_indices(toindices_object(object))
}

#[primitive("centerofmass")]
pub fn centerofmass_indices(patch: Indices) -> IntegerTuple {
    centerofmass_from_indices(patch)
}

#[primitive("palette")]
pub fn palette_grid(grid: Grid) -> IntegerSet {
    grid.into_iter()
        .flatten()
        .collect()
}

#[primitive("palette")]
pub fn palette_object(object: Object) -> IntegerSet {
    object
        .into_iter()
        .map(|(color, _)| color)
        .collect()
}

#[primitive("numcolors")]
pub fn numcolors_grid(grid: Grid) -> Integer {
    palette_grid(grid).len() as Integer
}

#[primitive("numcolors")]
pub fn numcolors_object(object: Object) -> Integer {
    palette_object(object).len() as Integer
}

#[primitive("square")]
pub fn square_grid(grid: Grid) -> Boolean {
    if grid.is_empty() {
        panic!("square received empty grid");
    }

    let width = grid[0].len();

    if grid.iter().any(|row| row.len() != width) {
        panic!("square received wrong grid");
    }

    grid.len() == width
}

fn square_patch(
    height: Integer,
    width: Integer,
    size: Integer,
) -> Boolean {
    height * width == size && height == width
}

#[primitive("square")]
pub fn square_object(object: Object) -> Boolean {
    let h = height_object(object.clone());
    let w = width_object(object.clone());
    let l = object.len() as Integer;

    square_patch(h, w, l)
}

#[primitive("square")]
pub fn square_indices(indices: Indices) -> Boolean {
    let h = height_indices(indices.clone());
    let w = width_indices(indices.clone());
    let l = indices.len() as Integer;

    square_patch(h, w, l)
}

fn vline_patch(
    height: Integer,
    width: Integer,
    size: Integer,
) -> Boolean {
    height == size && width == 1
}

#[primitive("vline")]
pub fn vline_object(object: Object) -> Boolean {
    let h = height_object(object.clone());
    let w = width_object(object.clone());
    let l = object.len() as Integer;

    vline_patch(h, w, l)
}

#[primitive("vline")]
pub fn vline_indices(indices: Indices) -> Boolean {
    let h = height_indices(indices.clone());
    let w = width_indices(indices.clone());
    let l = indices.len() as Integer;

    vline_patch(h, w, l)
}

fn hline_patch(
    height: Integer,
    width: Integer,
    size: Integer,
) -> Boolean {
    width == size && height == 1
}

#[primitive("hline")]
pub fn hline_object(object: Object) -> Boolean {
    let h = height_object(object.clone());
    let w = width_object(object.clone());
    let l = object.len() as Integer;

    hline_patch(h, w, l)
}

#[primitive("hline")]
pub fn hline_indices(indices: Indices) -> Boolean {
    let h = height_indices(indices.clone());
    let w = width_indices(indices.clone());
    let l = indices.len() as Integer;

    hline_patch(h, w, l)
}

#[primitive("dneighbors")]
pub fn dneighbors(loc: IntegerTuple) -> Indices {
    let (i, j) = loc;

    let mut result = Indices::new();

    result.insert((i - 1, j));
    result.insert((i + 1, j));
    result.insert((i, j - 1));
    result.insert((i, j + 1));

    result
}

#[primitive("ineighbors")]
pub fn ineighbors(loc: IntegerTuple) -> Indices {
    let (i, j) = loc;

    let mut result = Indices::new();

    result.insert((i - 1, j - 1));
    result.insert((i - 1, j + 1));
    result.insert((i + 1, j - 1));
    result.insert((i + 1, j + 1));

    result
}

#[primitive("neighbors")]
pub fn neighbors(loc: IntegerTuple) -> Indices {
    let mut result = dneighbors(loc);
    result.extend(ineighbors(loc));

    result
}

#[primitive("recolor")]
pub fn recolor_indices(
    value: Integer,
    patch: Indices,
) -> Object {
    patch
        .into_iter()
        .map(|index| (value, index))
        .collect()
}

#[primitive("recolor")]
pub fn recolor_object(
    value: Integer,
    patch: Object,
) -> Object {
    recolor_indices(value, toindices_object(patch))
}

#[primitive("asobject")]
pub fn asobject(grid: Grid) -> Object {
    let mut result = Object::new();

    for (i, row) in grid.iter().enumerate() {
        for (j, &color) in row.iter().enumerate() {
            result.insert((
                color,
                (i as Integer, j as Integer),
            ));
        }
    }

    result
}

#[primitive("toobject")]
pub fn toobject_indices(patch: Indices, grid: Grid) -> Object {
    let h = grid.len();

    if h == 0 {
        panic!("toobject received wrong height");
    }

    let w = grid[0].len();

    let mut result = Object::new();

    for &(i, j) in &patch {
        if i >= 0
            && (i as usize) < h
            && j >= 0
            && (j as usize) < w
        {
            result.insert((
                grid[i as usize][j as usize],
                (i, j),
            ));
        }
    }

    result
}

#[primitive("toobject")]
pub fn toobject_object(patch: Object, grid: Grid) -> Object {
    toobject_indices(toindices_object(patch), grid)
}

fn color_counts_grid(grid: &Grid) -> IntegerCountMap {
    let mut counts = IntegerCountMap::new();

    for row in grid {
        for &color in row {
            *counts.entry(color).or_insert(0) += 1;
        }
    }

    counts
}

fn color_counts_object(object: &Object) -> IntegerCountMap {
    let mut counts = IntegerCountMap::new();

    for &(color, _) in object {
        *counts.entry(color).or_insert(0) += 1;
    }

    counts
}

#[primitive("colorcount")]
pub fn colorcount_grid(grid: Grid, value: Integer) -> Integer {
    grid.iter()
        .flatten()
        .filter(|&&color| color == value)
        .count() as Integer
}

#[primitive("colorcount")]
pub fn colorcount_object(object: Object, value: Integer) -> Integer {
    object
        .iter()
        .filter(|&&(color, _)| color == value)
        .count() as Integer
}

fn most_common_color(counts: IntegerCountMap) -> Integer {
    counts
        .into_iter()
        .max_by(|(color_a, count_a), (color_b, count_b)| {
            count_a
                .cmp(count_b)
                .then_with(|| color_b.cmp(color_a))
        })
        .map(|(color, _)| color)
        .expect("mostcommon received wrong value")
}

fn least_common_color(counts: IntegerCountMap) -> Integer {
    counts
        .into_iter()
        .min_by(|(color_a, count_a), (color_b, count_b)| {
            count_a
                .cmp(count_b)
                .then_with(|| color_a.cmp(color_b))
        })
        .map(|(color, _)| color)
        .expect("leastcommon received wrong value")
}

#[primitive("mostcolor")]
pub fn mostcolor_grid(grid: Grid) -> Integer {
    most_common_color(color_counts_grid(&grid))
}

#[primitive("mostcolor")]
pub fn mostcolor_object(object: Object) -> Integer {
    most_common_color(color_counts_object(&object))
}

#[primitive("leastcolor")]
pub fn leastcolor_grid(grid: Grid) -> Integer {
    least_common_color(color_counts_grid(&grid))
}

#[primitive("leastcolor")]
pub fn leastcolor_object(object: Object) -> Integer {
    least_common_color(color_counts_object(&object))
}

fn in_bounds(
    i: Integer,
    j: Integer,
    h: usize,
    w: usize,
) -> Option<(usize, usize)> {
    if i < 0 || j < 0 {
        return None;
    }

    let i = i as usize;
    let j = j as usize;

    if i < h && j < w {
        Some((i, j))
    } else {
        None
    }
}

#[primitive("underfill")]
pub fn underfill_indices(
    grid: Grid,
    value: Integer,
    patch: Indices,
) -> Grid {
    let h = grid.len();

    if h == 0 {
        panic!("underfill received wrong height");
    }

    let w = grid[0].len();
    let bg = mostcolor_grid(grid.clone());

    let mut result = grid.clone();

    for &(i, j) in &patch {
        if let Some((i, j)) = in_bounds(i, j, h, w)
            && result[i as usize][j as usize] == bg
        {
            result[i as usize][j as usize] = value;
        }
    }

    if result == grid {
        panic!("underfill produced identity");
    }

    result
}

#[primitive("underpaint")]
pub fn underpaint(
    grid: Grid,
    object: Object,
) -> Grid {
    let h = grid.len();

    if h == 0 {
        panic!("underpaint received wrong height");
    }

    let w = grid[0].len();
    let bg = mostcolor_grid(grid.clone());

    let mut result = grid.clone();

    for &(value, (i, j)) in &object {
        if let Some((i, j)) = in_bounds(i, j, h, w)
            && result[i as usize][j as usize] == bg
        {
            result[i as usize][j as usize] = value;
        }
    }

    if result == grid {
        panic!("underpaint produced identity");
    }

    result
}

#[primitive("paint")]
pub fn paint(
    grid: Grid,
    object: Object,
) -> Grid {
    let h = grid.len();

    if h == 0 {
        panic!("paint received wrong height");
    }

    let w = grid[0].len();

    let mut result = grid.clone();

    for &(value, (i, j)) in &object {
        if let Some((i, j)) = in_bounds(i, j, h, w)
        {
            result[i as usize][j as usize] = value;
        }
    }

    if result == grid {
        panic!("paint produced identity");
    }

    result
}

#[primitive("fill")]
pub fn fill_indices(
    grid: Grid,
    value: Integer,
    patch: Indices,
) -> Grid {
    let h = grid.len();

    if h == 0 {
        panic!("fill received wrong height");
    }

    let w = grid[0].len();

    let mut result = grid.clone();

    for &(i, j) in &patch {
        if let Some((i, j)) = in_bounds(i, j, h, w)
        {
            result[i as usize][j as usize] = value;
        }
    }

    if result == grid {
        panic!("fill produced identity");
    }

    result
}

#[primitive("fill")]
pub fn fill_object(
    grid: Grid,
    value: Integer,
    object: Object,
) -> Grid {
    fill_indices(
        grid,
        value,
        toindices_object(object),
    )
}

fn cover_with_indices(grid: Grid, patch: Indices
) -> Grid {
    fill_indices(grid.clone(), mostcolor_grid(grid), patch)
}

#[primitive("cover")]
pub fn cover_indices(grid: Grid, patch: Indices) -> Grid {
    cover_with_indices(grid, patch)
}

#[primitive("cover")]
pub fn cover_object(grid: Grid, object: Object) -> Grid {
    cover_with_indices(grid, toindices_object(object))
}

#[primitive("connect")]
pub fn connect(
    a: IntegerTuple,
    b: IntegerTuple,
) -> Indices {
    let (ai, aj) = a;
    let (bi, bj) = b;

    if (bi - ai).abs() > (SHOOT_DISTANCE * MAX_SIZE).try_into().unwrap()
        || (bj - aj).abs() > (SHOOT_DISTANCE * MAX_SIZE).try_into().unwrap()
    {
        panic!("connect received wrong delta");
    }

    let (di, dj) = if ai == bi {
        (0, if bj > aj { 1 } else { -1 })
    } else if aj == bj {
        (if bi > ai { 1 } else { -1 }, 0)
    } else if (bi - ai).abs() == (bj - aj).abs() {
        (
            if bi > ai { 1 } else { -1 },
            if bj > aj { 1 } else { -1 },
        )
    } else {
        panic!("connect received wrong case");
    };

    let mut result = Indices::new();

    let mut i = ai;
    let mut j = aj;

    loop {
        result.insert((i, j));

        if i == bi && j == bj {
            break;
        }

        i += di;
        j += dj;
    }

    result
}

fn corners_from_indices(indices: Indices) -> Indices {
    let mut result = Indices::new();

    result.insert(ulcorner(indices.clone()));
    result.insert(urcorner(indices.clone()));
    result.insert(llcorner(indices.clone()));
    result.insert(lrcorner(indices));

    result
}

#[primitive("corners")]
pub fn corners_object(object: Object) -> Indices {
    corners_from_indices(toindices_object(object))
}

#[primitive("corners")]
pub fn corners_indices(indices: Indices) -> Indices {
    corners_from_indices(indices)
}

#[primitive("index")]
pub fn index(
    grid: Grid,
    loc: IntegerTuple,
) -> Integer {
    let (i, j) = loc;

    if i < 0 || j < 0 {
        panic!("index received negative i or j");
    }

    let i = i as usize;
    let j = j as usize;

    let h = grid.len();

    if h == 0 || i >= h {
        panic!("index received wrong height");
    }

    let w = grid[i].len();

    if j >= w {
        panic!("index received wrong width");
    }

    grid[i][j]
}

#[primitive("canvas")]
pub fn canvas(
    value: Integer,
    dimensions: IntegerTuple,
) -> Grid {
    let (h, w) = dimensions;

    if h <= 0 || w <= 0 {
        panic!("canvas received wrong height or width");
    }

    vec![vec![value; w as usize]; h as usize]
}

#[primitive("position")]
pub fn position_indices_object(
    a: Indices,
    b: Object,
) -> IntegerTuple {
    position_indices(
        a,
        toindices_object(b),
    )
}

#[primitive("position")]
pub fn position_object_indices(
    a: Object,
    b: Indices,
) -> IntegerTuple {
    position_indices(
        toindices_object(a),
        b,
    )
}

#[primitive("position")]
pub fn position_objects(
    a: Object,
    b: Object,
) -> IntegerTuple {
    position_indices(
        toindices_object(a),
        toindices_object(b),
    )
}

#[primitive("position")]
fn position_indices(
    a: Indices,
    b: Indices,
) -> IntegerTuple {
    let (ia, ja) = center_indices(a);
    let (ib, jb) = center_indices(b);

    if ia == ib {
        (0, if ja < jb { 1 } else { -1 })
    } else if ja == jb {
        (if ia < ib { 1 } else { -1 }, 0)
    } else if ia < ib {
        (1, if ja < jb { 1 } else { -1 })
    } else {
        (-1, if ja < jb { 1 } else { -1 })
    }
}

fn center_from_bounds(
    um: Integer,
    h: Integer,
    lm: Integer,
    w: Integer,
) -> IntegerTuple {
    (um + h / 2, lm + w / 2)
}

#[primitive("center")]
pub fn center_object(object: Object) -> IntegerTuple {
    center_from_bounds(
        uppermost_object(object.clone()),
        height_object(object.clone()),
        leftmost_object(object.clone()),
        width_object(object),
    )
}

#[primitive("center")]
pub fn center_indices(indices: Indices) -> IntegerTuple {
    center_from_bounds(
        uppermost_indices(indices.clone()),
        height_indices(indices.clone()),
        leftmost_indices(indices.clone()),
        width_indices(indices),
    )
}

#[primitive("hsplit")]
pub fn hsplit(grid: Grid, n: Integer) -> GridVector {
    if n <= 0 {
        panic!("hsplit received wrong n");
    }

    let h = grid.len();
    if h == 0 {
        panic!("hsplit received wrong height");
    }

    let w = grid[0].len();
    let n = n as usize;

    let part_width = w / n;
    let offset = usize::from(w % n != 0);

    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        let start_col = part_width * i + i * offset;

        result.push(crop(
            grid.clone(),
            (0, start_col as Integer),
            (h as Integer, part_width as Integer),
        ));
    }

    result
}

#[primitive("vsplit")]
pub fn vsplit(grid: Grid, n: Integer) -> GridVector {
    if n <= 0 {
        panic!("vsplit received wrong n");
    }

    let h = grid.len();
    if h == 0 {
        panic!("vsplit received wrong height");
    }

    let w = grid[0].len();
    let n = n as usize;

    let part_height = h / n;
    let offset = usize::from(h % n != 0);

    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        let start_row = part_height * i + i * offset;

        result.push(crop(
            grid.clone(),
            (start_row as Integer, 0),
            (part_height as Integer, w as Integer),
        ));
    }

    result
}

#[primitive("cellwise")]
pub fn cellwise(a: Grid, b: Grid, fallback: Integer) -> Grid {
    let h = a.len();

    if h == 0 || b.len() != h {
        panic!("cellwise received wrong height");
    }

    let w = a[0].len();

    if w == 0 || b.iter().any(|row| row.len() != w) {
        panic!("cellwise received wrong width");
    }

    let mut result = vec![vec![fallback; w]; h];

    for i in 0..h {
        for j in 0..w {
            result[i][j] = if a[i][j] == b[i][j] {
                a[i][j]
            } else {
                fallback
            };
        }
    }

    if result == a {
        panic!("cellwise produced identity");
    }

    result
}

#[primitive("replace")]
pub fn replace(
    grid: Grid,
    replacee: Integer,
    replacer: Integer,
) -> Grid {
    let mut result = grid.clone();

    for row in &mut result {
        for value in row {
            if *value == replacee {
                *value = replacer;
            }
        }
    }

    if result == grid {
        panic!("replace produced identity");
    }

    result
}

#[primitive("switch")]
pub fn switch(
    grid: Grid,
    a: Integer,
    b: Integer,
) -> Grid {
    let mut result = grid.clone();

    for row in &mut result {
        for value in row {
            if *value == a {
                *value = b;
            } else if *value == b {
                *value = a;
            }
        }
    }

    if result == grid {
        panic!("switch produced identity");
    }

    result
}

#[primitive("hupscale")]
pub fn hupscale(grid: Grid, factor: Integer) -> Grid {
    if factor <= 1 || factor > 10 {
        panic!("hupscale received wrong factor");
    }

    if grid.len() > MAX_SIZE
        || grid.first().map_or(false, |row| row.len() > MAX_SIZE)
    {
        panic!("hupscale received wrong grid");
    }

    let result: Grid = grid
        .iter()
        .map(|row| {
            row.iter()
                .flat_map(|&value| std::iter::repeat_n(value, factor as usize))
                .collect()
        })
        .collect();

    if result == grid {
        panic!("hupscale produced identity");
    }

    result
}

#[primitive("vupscale")]
pub fn vupscale(grid: Grid, factor: Integer) -> Grid {
    if factor <= 1 || factor > 10 {
        panic!("vupscale received wrong factor");
    }

    if grid.len() > MAX_SIZE
        || grid.first().map_or(false, |row| row.len() > MAX_SIZE)
    {
        panic!("vupscale received wrong grid");
    }

    let mut result =
        Vec::with_capacity(grid.len() * factor as usize);

    for row in &grid {
        for _ in 0..factor {
            result.push(row.clone());
        }
    }

    if result == grid {
        panic!("vupscale produced identity");
    }

    result
}

#[primitive("upscale")]
fn upscale_grid(grid: Grid, factor: Integer) -> Grid {
    if grid.len() > MAX_SIZE
        || grid.first().map_or(false, |row| row.len() > MAX_SIZE)
    {
        panic!("upscale received wrong grid");
    }

    let factor = factor as usize;

    let mut result =
        Vec::with_capacity(grid.len() * factor);

    for row in &grid {
        let mut new_row =
            Vec::with_capacity(row.len() * factor);

        for &value in row {
            for _ in 0..factor {
                new_row.push(value);
            }
        }

        for _ in 0..factor {
            result.push(new_row.clone());
        }
    }

    if result == grid {
        panic!("upscale produced identity");
    }

    result
}

#[primitive("upscale")]
fn upscale_object(
    object: Object,
    factor: Integer,
) -> Object {
    if object.is_empty() {
        return Object::new();
    }

    let (di_inv, dj_inv) = ulcorner(toindices_object(object.clone()));

    let di = -di_inv;
    let dj = -dj_inv;

    let normalized = shift_object(
        object,
        (di, dj),
    );

    let mut result = Object::new();

    for &(value, (i, j)) in &normalized {
        for io in 0..factor {
            for jo in 0..factor {
                result.insert((
                    value,
                    (
                        i * factor + io,
                        j * factor + jo,
                    ),
                ));
            }
        }
    }

    shift_object(
        result,
        (di_inv, dj_inv),
    )
}

#[primitive("downscale")]
pub fn downscale(grid: Grid, factor: Integer) -> Grid {
    if factor <= 1 {
        panic!("downscale received wrong factor");
    }

    let factor = factor as usize;

    let h = grid.len();

    if h == 0 {
        panic!("downscale received wrong height");
    }

    let w = grid[0].len();

    let mut result = Vec::new();

    for i in (0..h).step_by(factor) {
        let mut row = Vec::new();

        for j in (0..w).step_by(factor) {
            row.push(grid[i][j]);
        }

        result.push(row);
    }

    if result == grid {
        panic!("downscale produced identity");
    }

    result
}

#[primitive("tophalf")]
pub fn tophalf(grid: Grid) -> Grid {
    if grid.is_empty() {
        panic!("tophalf received empty grid");
    }

    let mid = grid.len() / 2;

    grid[..mid].to_vec()
}

#[primitive("bottomhalf")]
pub fn bottomhalf(grid: Grid) -> Grid {
    if grid.is_empty() {
        panic!("bottomhalf received empty grid");
    }

    let mid = grid.len().div_ceil(2);

    grid[mid..].to_vec()
}

#[primitive("lefthalf")]
pub fn lefthalf(grid: Grid) -> Grid {
    rot270(tophalf(rot90(grid.clone())))
}

#[primitive("righthalf")]
pub fn righthalf(grid: Grid) -> Grid {
    rot270(bottomhalf(rot90(grid)))
}

#[primitive("trim")]
pub fn trim(grid: Grid) -> Grid {
    if grid.len() < 3 {
        panic!("trim received wrong grid");
    }

    grid[1..grid.len() - 1]
        .iter()
        .map(|row| {
            if row.len() < 3 {
                panic!("trim received wrong grid");
            }

            row[1..row.len() - 1].to_vec()
        })
        .collect()
}

#[primitive("branch")]
pub fn branch_grid(condition: Boolean, a: Grid, b: Grid) -> Grid {
    if condition {
        a
    }
    else {
        b
    }
}

#[primitive("branch")]
pub fn branch_objects(condition: Boolean, a: Objects, b: Objects) -> Objects {
    if condition {
        a
    }
    else {
        b
    }
}

#[primitive("branch")]
pub fn branch_object(condition: Boolean, a: Object, b: Object) -> Object {
    if condition {
        a
    }
    else {
        b
    }
}

#[primitive("branch")]
pub fn branch_indices(condition: Boolean, a: Indices, b: Indices) -> Indices {
    if condition {
        a
    }
    else {
        b
    }
}

#[primitive("branch")]
pub fn branch_tuple(condition: Boolean, a: IntegerTuple, b: IntegerTuple) -> IntegerTuple {
    if condition {
        a
    }
    else {
        b
    }
}

#[primitive("branch")]
pub fn branch_integer(condition: Boolean, a: Integer, b: Integer) -> Integer {
    if condition {
        a
    }
    else {
        b
    }
}

#[primitive("branch")]
pub fn branch_boolean(condition: Boolean, a: Boolean, b: Boolean) -> Boolean {
    if condition {
        a
    }
    else {
        b
    }
}

#[primitive("pair")]
pub fn pair(a: IntegerVector, b: IntegerVector) -> Grid {
    if a.len() != b.len() {
        panic!("pair received wrong values");
    }

    a.into_iter()
        .zip(b)
        .map(|(x, y)| vec![x, y])
        .collect()
}

#[primitive("astuple")]
pub fn astuple(a: Integer, b: Integer) -> IntegerTuple {
    (a, b)
}

#[primitive("positive")]
pub fn positive(a: Integer) -> Boolean {
    a > 0
}

#[primitive("sign")]
pub fn sign_tuple(a: IntegerTuple) -> IntegerTuple {
    (sign(a.0), sign(a.1))
}

#[primitive("sign")]
pub fn sign(a: Integer) -> Integer {
    if a == 0 {
        0
    }
    else if a > 0 {
        1
    }
    else {
        -1
    }
}

#[primitive("crement")]
pub fn crement_tuple(a: IntegerTuple) -> IntegerTuple {
    (crement(a.0), crement(a.1))
}

#[primitive("crement")]
pub fn crement(a: Integer) -> Integer {
    if a == 0 {
        a
    }
    else if a > 0 {
        increment(a)
    }
    else {
        decrement(a)
    }
}

#[primitive("decrement")]
pub fn decrement_tuple(a: IntegerTuple) -> IntegerTuple {
    (a.0 - 1, a.1 - 1)
}

#[primitive("decrement")]
pub fn decrement(a: Integer) -> Integer {
    a - 1
}

#[primitive("increment")]
pub fn increment_tuple(a: IntegerTuple) -> IntegerTuple {
    (a.0 + 1, a.1 + 1)
}

#[primitive("increment")]
pub fn increment(a: Integer) -> Integer {
    a + 1
}

#[primitive("greater")]
pub fn greater(a: Integer, b: Integer) -> Boolean {
    a > b
}

#[primitive("halve")]
pub fn halve(a: Integer) -> Integer {
    a / 2
}

#[primitive("halve")]
pub fn halve_tuple(a: IntegerTuple) -> IntegerTuple {
    (a.0 / 2, a.1 / 2)
}

#[primitive("double")]
pub fn double(a: Integer) -> Integer {
    a * 2
}

#[primitive("double")]
pub fn double_tuple(a: IntegerTuple) -> IntegerTuple {
    (a.0 * 2, a.1 * 2)
}

#[primitive("even")]
pub fn even(a: Integer) -> Boolean {
    a % 2 == 0
}

#[primitive("invert")]
pub fn invert(a: Integer) -> Integer {
    -a
}

#[primitive("invert")]
pub fn invert_tuple(
    a: IntegerTuple,
) -> IntegerTuple {
    (-a.0, -a.0,)
}

#[primitive("toivec")]
pub fn toivec(
    i: Integer,
) -> IntegerTuple {
    (i, 0)
}

#[primitive("tojvec")]
pub fn tojvec(
    j: Integer,
) -> IntegerTuple {
    (0, j)
}

#[primitive("portrait")]
pub fn portrait_grid(
    grid: Grid,
) -> Boolean {
    width_grid(grid.clone()) < height_grid(grid)
}

#[primitive("portrait")]
pub fn portrait_object(
    object: Object,
) -> Boolean {
    width_object(object.clone()) < height_object(object)
}

#[primitive("portrait")]
pub fn portrait_indices(
    indices: Indices,
) -> Boolean {
    width_indices(indices.clone()) < height_indices(indices)
}

#[primitive("shape")]
pub fn shape_grid(
    grid: Grid,
) -> IntegerTuple {
    (width_grid(grid.clone()), height_grid(grid))
}

#[primitive("shape")]
pub fn shape_object(
    object: Object,
) -> IntegerTuple {
    (width_object(object.clone()), height_object(object))
}

#[primitive("shape")]
pub fn shape_indices(
    indices: Indices,
) -> IntegerTuple {
    (width_indices(indices.clone()), height_indices(indices))
}

#[primitive("both")]
fn both(a: Boolean, b: Boolean) -> Boolean {
    a && b
}

#[primitive("either")]
fn either(a: Boolean, b: Boolean) -> Boolean {
    a || b
}

#[primitive("identity")]
fn identity_integer_vector(value: IntegerVector) -> IntegerVector {
    value
}

#[primitive("identity")]
fn identity_objects(value: Objects) -> Objects {
    value
}

#[primitive("identity")]
fn identity_indices(value: Indices) -> Indices {
    value
}

#[primitive("identity")]
fn identity_tuple(value: IntegerTuple) -> IntegerTuple {
    value
}

#[primitive("identity")]
fn identity_object(value: Object) -> Object {
    value
}

#[primitive("identity")]
fn identity_integer(value: Integer) -> Integer {
    value
}

#[primitive("identity")]
fn identity_grid(value: Grid) -> Grid {
    value
}

#[primitive("rot270")]
pub fn rot270(
    grid: Grid,
) -> Grid {
    let rows = grid.len();

    if rows == 0 {
        panic!("rot270: empty Grid");
    }

    let cols = grid[0].len();

    if cols == 0 || grid.iter().any(|row| row.len() != cols) {
        panic!("rot270: invalid Grid");
    }

    let mut result = vec![vec![0; rows]; cols];

    for i in 0..rows {
        for j in 0..cols {
            result[cols - 1 - j][i] = grid[i][j];
        }
    }

    if result == grid {
        panic!("rot270 produced identity");
    }

    result
}

#[primitive("rot180")]
pub fn rot180(
    grid: Grid,
) -> Grid {
    let rows = grid.len();

    if rows == 0 {
        panic!("rot180: empty Grid");
    }

    let cols = grid[0].len();

    if cols == 0 || grid.iter().any(|row| row.len() != cols) {
        panic!("rot180: invalid Grid");
    }

    let mut result = vec![vec![0; cols]; rows];

    for i in 0..rows {
        for j in 0..cols {
            result[rows - 1 - i][cols - 1 - j] = grid[i][j];
        }
    }

    if result == grid {
        panic!("rot180 produced identity");
    }

    result
}

#[primitive("rot90")]
pub fn rot90(
    grid: Grid,
) -> Grid {
    let rows = grid.len();

    if rows == 0 {
        panic!("rot90: empty Grid");
    }

    let cols = grid[0].len();

    if cols == 0 || grid.iter().any(|row| row.len() != cols) {
        panic!("rot90: invalid Grid");
    }

    let mut result = vec![vec![0; rows]; cols];

    for i in 0..rows {
        for j in 0..cols {
            result[j][rows - 1 - i] = grid[i][j];
        }
    }

    if result == grid {
        panic!("rot90 produced identity");
    }

    result
}

#[primitive("lrcorner")]
pub fn lrcorner_object(
    object: Object,
) -> IntegerTuple {
    lrcorner_indices(toindices_object(object))
}

#[primitive("lrcorner")]
pub fn lrcorner_indices(
    indices: Indices,
) -> IntegerTuple {
    let max_y = indices
        .iter()
        .map(|(y, _)| *y)
        .max()
        .expect("lrcorner: empty Indices");

    let max_x = indices
        .iter()
        .map(|(_, x)| *x)
        .max()
        .expect("lrcorner: empty Indices");

    (max_y, max_x)
}

#[primitive("llcorner")]
pub fn llcorner_object(
    object: Object,
) -> IntegerTuple {
    llcorner_indices(toindices_object(object))
}

#[primitive("llcorner")]
pub fn llcorner_indices(
    indices: Indices,
) -> IntegerTuple {
    let max_y = indices
        .iter()
        .map(|(y, _)| *y)
        .max()
        .expect("llcorner: empty Indices");

    let min_x = indices
        .iter()
        .map(|(_, x)| *x)
        .min()
        .expect("llcorner: empty Indices");

    (max_y, min_x)
}

#[primitive("urcorner")]
pub fn urcorner_object(
    object: Object,
) -> IntegerTuple {
    urcorner_indices(toindices_object(object))
}

#[primitive("urcorner")]
pub fn urcorner_indices(
    indices: Indices,
) -> IntegerTuple {
    let min_y = indices
        .iter()
        .map(|(y, _)| *y)
        .min()
        .expect("urcorner: empty Indices");

    let max_x = indices
        .iter()
        .map(|(_, x)| *x)
        .max()
        .expect("urcorner: empty Indices");

    (min_y, max_x)
}

#[primitive("ulcorner")]
pub fn ulcorner_object(
    object: Object,
) -> IntegerTuple {
     ulcorner_indices(toindices_object(object))
}

#[primitive("ulcorner")]
pub fn ulcorner_indices(
    indices: Indices,
) -> IntegerTuple {
    (
        uppermost_indices(indices.clone()),
        leftmost_indices(indices),
    )
}

#[primitive("vperiod")]
pub fn vperiod(
    object: Object,
) -> Integer {
    if object.is_empty() {
        panic!("vperiod: empty Object");
    }

    let normalized = normalize_object(object);
    let height = height_object(normalized.clone());

    for p in 1..height {
        let offsetted = shift_object(
            normalized.clone(),
            (-p, 0),
        );

        let pruned: Object = offsetted
            .into_iter()
            .filter(|(_, (i, _))| *i >= 0)
            .collect();

        if pruned.iter().all(|cell| normalized.contains(cell)) {
            return p;
        }
    }

    height
}

#[primitive("hperiod")]
pub fn hperiod(
    object: Object,
) -> Integer {
    if object.is_empty() {
        panic!("hperiod: empty Object");
    }

    let normalized = normalize_object(object);
    let width = width_object(normalized.clone());

    for p in 1..width {
        let offsetted = shift_object(
            normalized.clone(),
            (0, -p),
        );

        let pruned: Object = offsetted
            .into_iter()
            .filter(|(_, (_, j))| *j >= 0)
            .collect();

        if pruned.iter().all(|cell| normalized.contains(cell)) {
            return p;
        }
    }

    width
}

#[primitive("width")]
pub fn width_grid(
    grid: Grid,
) -> Integer {
    grid.first()
        .map(|row| row.len() as Integer)
        .expect("width: empty Grid")
}

#[primitive("width")]
pub fn width_object(
    object: Object,
) -> Integer {
    rightmost_object(object.clone())
        - leftmost_object(object)
        + 1
}

#[primitive("width")]
pub fn width_indices(
    indices: Indices,
) -> Integer {
    rightmost_indices(indices.clone())
        - leftmost_indices(indices)
        + 1
}

#[primitive("height")]
pub fn height_grid(
    grid: Grid,
) -> Integer {
    grid.len() as Integer
}

#[primitive("height")]
pub fn height_object(
    object: Object,
) -> Integer {
    lowermost_object(object.clone())
        - uppermost_object(object)
        + 1
}

#[primitive("height")]
pub fn height_indices(
    indices: Indices,
) -> Integer {
    lowermost_indices(indices.clone())
        - uppermost_indices(indices)
        + 1
}

#[primitive("normalize")]
pub fn normalize_object(
    object: Object,
) -> Object {
    if object.is_empty() {
        return object;
    }

    let di = -uppermost_object(object.clone());
    let dj = -leftmost_object(object.clone());

    shift_object(object, (di, dj))
}

#[primitive("normalize")]
pub fn normalize_indices(
    indices: Indices,
) -> Indices {
    if indices.is_empty() {
        panic!("normalize: empty Indices");
    }

    let di = -uppermost_indices(indices.clone());
    let dj = -leftmost_indices(indices.clone());

    shift_indices(indices, (di, dj))
}

#[primitive("shift")]
pub fn shift_object(
    object: Object,
    direction: IntegerTuple,
) -> Object {
    let (di, dj) = direction;

    object
        .into_iter()
        .map(|(color, (i, j))| {
            (color, (i + di, j + dj))
        })
        .collect()
}

#[primitive("shift")]
pub fn shift_indices(
    indices: Indices,
    direction: IntegerTuple,
) -> Indices {
    let (di, dj) = direction;

    indices
        .into_iter()
        .map(|(i, j)| (i + di, j + dj))
        .collect()
}

#[primitive("rightmost")]
pub fn rightmost_object(
    object: Object,
) -> Integer {
    let indices = toindices_object(object);

    indices
        .iter()
        .map(|(_, j)| *j)
        .max()
        .expect("rightmost: empty Object")
}

#[primitive("rightmost")]
pub fn rightmost_indices(
    indices: Indices,
) -> Integer {
    indices
        .iter()
        .map(|(_, j)| *j)
        .max()
        .expect("rightmost: empty Indices")
}

#[primitive("leftmost")]
pub fn leftmost_object(
    object: Object,
) -> Integer {
    let indices = toindices_object(object);

    indices
        .iter()
        .map(|(_, j)| *j)
        .min()
        .expect("leftmost: empty Object")
}

#[primitive("leftmost")]
pub fn leftmost_indices(
    indices: Indices,
) -> Integer {
    indices
        .iter()
        .map(|(_, j)| *j)
        .min()
        .expect("leftmost: empty Indices")
}

#[primitive("lowermost")]
pub fn lowermost_object(
    object: Object,
) -> Integer {
    let indices = toindices_object(object);

    indices
        .iter()
        .map(|(i, _)| *i)
        .max()
        .expect("lowermost: empty Object")
}

#[primitive("lowermost")]
pub fn lowermost_indices(
    indices: Indices,
) -> Integer {
    indices
        .iter()
        .map(|(i, _)| *i)
        .max()
        .expect("lowermost: empty Indices")
}

#[primitive("uppermost")]
pub fn uppermost_object(
    object: Object,
) -> Integer {
    let indices = toindices_object(object);
    indices
        .iter()
        .map(|(i, _)| *i)
        .min()
        .expect("uppermost: empty Object")
}

#[primitive("uppermost")]
pub fn uppermost_indices(
    indices: Indices,
) -> Integer {
    indices
        .iter()
        .map(|(i, _)| *i)
        .min()
        .expect("uppermost: empty Indices")
}

#[primitive("toindices")]
pub fn toindices_object(
    object: Object,
) -> Indices {
    if object.is_empty() {
        panic!("toindices: empty Object");
    }

    object
        .iter()
        .map(|(_, position)| *position)
        .collect()
}

#[primitive("toindices")]
pub fn toindices_indices(
    indices: Indices,
) -> Indices {
    indices
}

#[primitive("size")]
pub fn size_integer_tuple(
    _value: IntegerTuple,
) -> Integer {
    2
}

#[primitive("size")]
pub fn size_indices(
    value: Indices,
) -> Integer {
    value.len() as Integer
}

#[primitive("size")]
pub fn size_object(
    value: Object,
) -> Integer {
    value.len() as Integer
}

#[primitive("size")]
pub fn size_objects(
    value: Objects,
) -> Integer {
    value.len() as Integer
}

#[primitive("size")]
pub fn size_integer_vector(
    value: IntegerVector,
) -> Integer {
    value.len() as Integer
}

#[primitive("size")]
pub fn size_grid(
    value: Grid,
) -> Integer {
    value.len() as Integer
}

#[primitive("size")]
pub fn size_object_vector(
    value: ObjectVector,
) -> Integer {
    value.len() as Integer
}

#[primitive("equality")]
pub fn equality_boolean(a: Boolean, b: Boolean) -> Boolean {
    a == b
}

#[primitive("equality")]
pub fn equality_integer(a: Integer, b: Integer) -> Boolean {
    a == b
}

#[primitive("equality")]
pub fn equality_tuple(
    a: IntegerTuple,
    b: IntegerTuple,
) -> Boolean {
    a == b
}

#[primitive("equality")]
pub fn equality_grid(a: Grid, b: Grid) -> Boolean {
    a == b
}

#[primitive("equality")]
pub fn equality_object(a: Object, b: Object) -> Boolean {
    a == b
}

#[primitive("equality")]
pub fn equality_objects(a: Objects, b: Objects) -> Boolean {
    a == b
}

#[primitive("equality")]
pub fn equality_indices(a: Indices, b: Indices) -> Boolean {
    a == b
}

#[primitive("equality")]
pub fn equality_integer_vector(
    a: IntegerVector,
    b: IntegerVector,
) -> Boolean {
    a == b
}

#[primitive("color")]
pub fn color(object: Object) -> Integer {
    object
        .iter()
        .next()
        .map(|(color, _)| *color)
        .expect("empty Object")
}

#[primitive("flip")]
pub fn flip(value: Boolean) -> Boolean {
    !value
}

#[primitive("add")]
pub fn add(a: Integer, b: Integer) -> Integer {
    let c = a + b;

    if c == a || c == b {
        panic!("add produced identity");
    }

    c
}

#[primitive("add")]
pub fn add_tuple_tuple(
    a: IntegerTuple,
    b: IntegerTuple,
) -> IntegerTuple {
    let result = (
        a.0 + b.0,
        a.1 + b.1,
    );

    if result == a || result == b {
        panic!("add produced identity");
    }

    result
}

#[primitive("add")]
pub fn add_integer_tuple(
    a: Integer,
    b: IntegerTuple,
) -> IntegerTuple {
    let result = (
        a + b.0,
        a + b.1,
    );

    if result == b {
        panic!("add produced identity");
    }

    result
}

#[primitive("add")]
pub fn add_tuple_integer(
    a: IntegerTuple,
    b: Integer,
) -> IntegerTuple {
    let result = (
        a.0 + b,
        a.1 + b,
    );

    if result == a {
        panic!("add produced identity");
    }

    result
}

#[primitive("subtract")]
pub fn subtract(a: Integer, b: Integer) -> Integer {
    let c: i16 = a - b;

    if c == a || c == b {
        panic!("subtract produced identity");
    }

    c
}

#[primitive("subtract")]
pub fn subtract_tuple_tuple(
    a: IntegerTuple,
    b: IntegerTuple,
) -> IntegerTuple {
    let result = (
        a.0 - b.0,
        a.1 - b.1,
    );

    if result == a || result == b {
        panic!("subtract produced identity");
    }

    result
}

#[primitive("subtract")]
pub fn subtract_integer_tuple(
    a: Integer,
    b: IntegerTuple,
) -> IntegerTuple {
    let result = (
        a - b.0,
        a - b.1,
    );

    if result == b {
        panic!("subtract produced identity");
    }

    result
}

#[primitive("subtract")]
pub fn subtract_tuple_integer(
    a: IntegerTuple,
    b: Integer,
) -> IntegerTuple {
    let result = (
        a.0 - b,
        a.1 - b,
    );

    if result == a {
        panic!("subtract produced identity");
    }

    result
}

#[primitive("multiply")]
pub fn multiply(a: Integer, b: Integer) -> Integer {
    let c = a * b;

    if c == a || c == b {
        panic!("multiply produced identity");
    }

    c
}

#[primitive("multiply")]
pub fn multiply_tuple_tuple(
    a: IntegerTuple,
    b: IntegerTuple,
) -> IntegerTuple {
    let result = (
        a.0 * b.0,
        a.1 * b.1,
    );

    if result == a || result == b {
        panic!("multiply produced identity");
    }

    result
}

#[primitive("multiply")]
pub fn multiply_integer_tuple(
    a: Integer,
    b: IntegerTuple,
) -> IntegerTuple {
    let result = (
        a * b.0,
        a * b.1,
    );

    if result == b {
        panic!("multiply produced identity");
    }

    result
}

#[primitive("multiply")]
pub fn multiply_tuple_integer(
    a: IntegerTuple,
    b: Integer,
) -> IntegerTuple {
    let result = (
        a.0 * b,
        a.1 * b,
    );

    if result == a {
        panic!("multiply produced identity");
    }

    result
}

#[primitive("divide")]
pub fn divide(a: Integer, b: Integer) -> Integer {
    if b == 0 {
        panic!("division by zero");
    }

    let c = a / b;

    if c == a || c == b {
        panic!("divide produced identity");
    }

    c
}

#[primitive("divide")]
pub fn divide_tuple_tuple(
    a: IntegerTuple,
    b: IntegerTuple,
) -> IntegerTuple {
    if b.0 == 0 || b.1 == 0 {
        panic!("division by zero");
    }

    let result = (
        a.0 / b.0,
        a.1 / b.1,
    );

    if result == a || result == b {
        panic!("divide produced identity");
    }

    result
}

#[primitive("divide")]
pub fn divide_integer_tuple(
    a: Integer,
    b: IntegerTuple,
) -> IntegerTuple {
    if b.0 == 0 || b.1 == 0 {
        panic!("division by zero");
    }

    let result = (
        a / b.0,
        a / b.1,
    );

    if result == b {
        panic!("divide produced identity");
    }

    result
}

#[primitive("divide")]
pub fn divide_tuple_integer(
    a: IntegerTuple,
    b: Integer,
) -> IntegerTuple {
    if b == 0 {
        panic!("division by zero");
    }

    let result = (
        a.0 / b,
        a.1 / b,
    );

    if result == a {
        panic!("divide produced identity");
    }

    result
}

#[primitive("hmirror")]
pub fn hmirror(mut grid: Grid) -> Grid {
    let original = grid.clone();

    for row in &mut grid {
        row.reverse();
    }

    if grid == original {
        panic!("hmirror produced identity");
    }

    grid
}

#[primitive("hmirror")]
pub fn hmirror_object(object: Object) -> Object {
    let (y_min, y_max) = object
        .iter()
        .map(|(_, (y, _))| *y)
        .fold((Integer::MAX, Integer::MIN), |(min_y, max_y), y| {
            (min_y.min(y), max_y.max(y))
        });

    let d = y_min + y_max;

    let result: Object = object
        .iter()
        .map(|(value, (y, x))| (*value, (d - y, *x)))
        .collect();

    if result == object {
        panic!("hmirror produced identity");
    }

    result
}

#[primitive("hmirror")]
pub fn hmirror_indices(indices: Indices) -> Indices {
    let (y_min, y_max) = indices
        .iter()
        .map(|(y, _)| *y)
        .fold((Integer::MAX, Integer::MIN), |(min_y, max_y), y| {
            (min_y.min(y), max_y.max(y))
        });

    let d = y_min + y_max;

    let result: Indices = indices
        .iter()
        .map(|(y, x)| (d - y, *x))
        .collect();

    if result == indices {
        panic!("hmirror produced identity");
    }

    result
}

#[primitive("vmirror")]
pub fn vmirror(mut grid: Grid) -> Grid {
    let original = grid.clone();

    grid.reverse();

    if grid == original {
        panic!("vmirror produced identity");
    }

    grid
}

#[primitive("vmirror")]
pub fn vmirror_object(object: Object) -> Object {
    let (x_min, x_max) = object
        .iter()
        .map(|(_, (_, x))| *x)
        .fold((Integer::MAX, Integer::MIN), |(min_x, max_x), x| {
            (min_x.min(x), max_x.max(x))
        });

    let d = x_min + x_max;

    let result: Object = object
        .iter()
        .map(|(value, (y, x))| (*value, (*y, d - x)))
        .collect();

    if result == object {
        panic!("vmirror produced identity");
    }

    result
}

#[primitive("vmirror")]
pub fn vmirror_indices(indices: Indices) -> Indices {
    let (x_min, x_max) = indices
        .iter()
        .map(|(_, x)| *x)
        .fold((Integer::MAX, Integer::MIN), |(min_x, max_x), x| {
            (min_x.min(x), max_x.max(x))
        });

    let d = x_min + x_max;

    let result: Indices = indices
        .iter()
        .map(|(y, x)| (*y, d - x))
        .collect();

    if result == indices {
        panic!("vmirror produced identity");
    }

    result
}

#[primitive("dmirror")]
pub fn dmirror(grid: Grid) -> Grid {
    let original = grid.clone();

    let height = grid.len();
    let width = grid.first().map_or(0, |row| row.len());

    let result: Grid = (0..width)
        .map(|x| {
            (0..height)
                .map(|y| grid[y][x])
                .collect()
        })
        .collect();

    if result == original {
        panic!("dmirror produced identity");
    }

    result
}

#[primitive("dmirror")]
pub fn dmirror_object(object: Object) -> Object {
    let (y_min, _y_max) = object
        .iter()
        .map(|(_, (y, _))| *y)
        .fold((Integer::MAX, Integer::MIN), |(min_y, max_y), y| {
            (min_y.min(y), max_y.max(y))
        });

    let (x_min, _x_max) = object
        .iter()
        .map(|(_, (_, x))| *x)
        .fold((Integer::MAX, Integer::MIN), |(min_x, max_x), x| {
            (min_x.min(x), max_x.max(x))
        });

    let result: Object = object
        .iter()
        .map(|(value, (y, x))| {
            (
                *value,
                (
                    y_min + (x - x_min),
                    x_min + (y - y_min),
                ),
            )
        })
        .collect();

    if result == object {
        panic!("dmirror produced identity");
    }

    result
}

#[primitive("dmirror")]
pub fn dmirror_indices(indices: Indices) -> Indices {
    let (y_min, _y_max) = indices
        .iter()
        .map(|(y, _)| *y)
        .fold((Integer::MAX, Integer::MIN), |(min_y, max_y), y| {
            (min_y.min(y), max_y.max(y))
        });

    let (x_min, _x_max) = indices
        .iter()
        .map(|(_, x)| *x)
        .fold((Integer::MAX, Integer::MIN), |(min_x, max_x), x| {
            (min_x.min(x), max_x.max(x))
        });

    let result: Indices = indices
        .iter()
        .map(|(y, x)| {
            (
                y_min + (x - x_min),
                x_min + (y - y_min),
            )
        })
        .collect();

    if result == indices {
        panic!("dmirror produced identity");
    }

    result
}

#[primitive("cmirror")]
pub fn cmirror(grid: Grid) -> Grid {
    let original = grid.clone();

    let height = grid.len();
    let width = grid.first().map_or(0, |row| row.len());

    let result: Grid = (0..width)
        .map(|x| {
            (0..height)
                .rev()
                .map(|y| grid[y][width - 1 - x])
                .collect()
        })
        .collect();

    if result == original {
        panic!("cmirror produced identity");
    }

    result
}

#[primitive("cmirror")]
pub fn cmirror_object(object: Object) -> Object {
    let (y_min, y_max) = object
        .iter()
        .map(|(_, (y, _))| *y)
        .fold((Integer::MAX, Integer::MIN), |(min_y, max_y), y| {
            (min_y.min(y), max_y.max(y))
        });

    let (x_min, x_max) = object
        .iter()
        .map(|(_, (_, x))| *x)
        .fold((Integer::MAX, Integer::MIN), |(min_x, max_x), x| {
            (min_x.min(x), max_x.max(x))
        });

    let result: Object = object
        .iter()
        .map(|(value, (y, x))| {
            (
                *value,
                (
                    y_max - (x - x_min),
                    x_max - (y - y_min),
                ),
            )
        })
        .collect();

    if result == object {
        panic!("cmirror produced identity");
    }

    result
}

#[primitive("cmirror")]
pub fn cmirror_indices(indices: Indices) -> Indices {
    let (y_min, y_max) = indices
        .iter()
        .map(|(y, _)| *y)
        .fold((Integer::MAX, Integer::MIN), |(min_y, max_y), y| {
            (min_y.min(y), max_y.max(y))
        });

    let (x_min, x_max) = indices
        .iter()
        .map(|(_, x)| *x)
        .fold((Integer::MAX, Integer::MIN), |(min_x, max_x), x| {
            (min_x.min(x), max_x.max(x))
        });

    let result: Indices = indices
        .iter()
        .map(|(y, x)| {
            (
                y_max - (x - x_min),
                x_max - (y - y_min),
            )
        })
        .collect();

    if result == indices {
        panic!("cmirror produced identity");
    }

    result
}

#[primitive("vconcat")]
pub fn vconcat(mut top: Grid, bottom: Grid) -> Grid {
    top.extend(bottom);
    top
}

#[primitive("hconcat")]
pub fn hconcat(mut left: Grid, right: Grid) -> Grid {
    for (left_row, right_row) in left.iter_mut().zip(right) {
        left_row.extend(right_row);
    }

    left
}

#[primitive("ulcorner")]
pub fn ulcorner(indices: Indices) -> IntegerTuple {
    indices
        .iter()
        .copied()
        .reduce(|(min_y, min_x), (y, x)| {
            (min_y.min(y), min_x.min(x))
        })
        .expect("empty Indices")
}

#[primitive("urcorner")]
pub fn urcorner(indices: Indices) -> IntegerTuple {
    indices
        .iter()
        .copied()
        .reduce(|(min_y, max_x), (y, x)| {
            (min_y.min(y), max_x.max(x))
        })
        .expect("empty Indices")
}

#[primitive("llcorner")]
pub fn llcorner(indices: Indices) -> IntegerTuple {
    indices
        .iter()
        .copied()
        .reduce(|(max_y, min_x), (y, x)| {
            (max_y.max(y), min_x.min(x))
        })
        .expect("empty Indices")
}

#[primitive("lrcorner")]
pub fn lrcorner(indices: Indices) -> IntegerTuple {
    indices
        .iter()
        .copied()
        .reduce(|(max_y, max_x), (y, x)| {
            (max_y.max(y), max_x.max(x))
        })
        .expect("empty Indices")
}

#[primitive("crop")]
pub fn crop(
    grid: Grid,
    ul: IntegerTuple,
    lr: IntegerTuple,
) -> Grid {
    let (y_min, x_min) = ul;
    let (y_max, x_max) = lr;

    grid
        .into_iter()
        .enumerate()
        .filter_map(|(y, row)| {
            let y = y as Integer;

            if y < y_min || y > y_max {
                return None;
            }

            let cropped = row
                .into_iter()
                .enumerate()
                .filter_map(|(x, value)| {
                    let x = x as Integer;

                    if x < x_min || x > x_max {
                        None
                    } else {
                        Some(value)
                    }
                })
                .collect();

            Some(cropped)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use crate::registry::{PRIMITIVES, Type};

    #[test]
    fn corners_of_indices() {
        let indices: Indices = BTreeSet::from([
            (2, 5),
            (4, 1),
            (7, 9),
        ]);

        assert_eq!(ulcorner(indices.clone()), (2, 1));
        assert_eq!(urcorner(indices.clone()), (2, 9));
        assert_eq!(llcorner(indices.clone()), (7, 1));
        assert_eq!(lrcorner(indices), (7, 9));
    }

    #[test]
    fn crop_extracts_inclusive_rectangle() {
        let grid = vec![
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 0],
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 0],
        ];

        let result = crop(
            grid,
            (1, 1),
            (2, 3),
        );

        assert_eq!(
            result,
            vec![
                vec![7, 8, 9],
                vec![2, 3, 4],
            ]
        );
    }

    #[test]
    fn hmirror_object_mirrors_rows() {
        let object: Object = BTreeSet::from([
            (1, (2, 5)),
            (2, (4, 1)),
            (3, (7, 9)),
        ]);

        let result = hmirror_object(object);

        assert_eq!(
            result,
            BTreeSet::from([
                (1, (7, 5)),
                (2, (5, 1)),
                (3, (2, 9)),
            ])
        );
    }

    #[test]
    fn hmirror_indices_mirrors_rows() {
        let indices: Indices = BTreeSet::from([
            (2, 5),
            (4, 1),
            (7, 9),
        ]);

        let result = hmirror_indices(indices);

        assert_eq!(
            result,
            BTreeSet::from([
                (7, 5),
                (5, 1),
                (2, 9),
            ])
        );
    }

    #[test]
    #[should_panic(expected = "hmirror produced identity")]
    fn hmirror_indices_rejects_identity() {
        let indices: Indices = BTreeSet::from([
            (2, 5),
            (4, 5),
        ]);

        hmirror_indices(indices);
    }

    #[test]
    #[should_panic(expected = "hmirror produced identity")]
    fn hmirror_object_rejects_identity() {
        let object: Object = BTreeSet::from([
            (1, (2, 5)),
            (1, (4, 5)),
        ]);

        hmirror_object(object);
    }

    #[test]
    #[should_panic(expected = "hmirror produced identity")]
    fn hmirror_grid_rejects_identity() {
        let grid = vec![
            vec![1, 2, 1],
            vec![3, 4, 3],
        ];

        hmirror(grid);
    }

    #[test]
    fn test_add_integer_integer() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_tuple_tuple() {
        assert_eq!(
            add_tuple_tuple((2, 3), (4, 5)),
            (6, 8),
        );
    }

    #[test]
    fn test_add_integer_tuple() {
        assert_eq!(
            add_integer_tuple(2, (4, 5)),
            (6, 7),
        );
    }

    #[test]
    fn test_add_tuple_integer() {
        assert_eq!(
            add_tuple_integer((4, 5), 2),
            (6, 7),
        );
    }

    #[test]
    #[should_panic(expected = "add produced identity")]
    fn add_integer_integer_rejects_identity() {
        add(0, 5);
    }

    #[test]
    #[should_panic(expected = "add produced identity")]
    fn add_tuple_tuple_rejects_identity() {
        add_tuple_tuple((0, 0), (2, 3));
    }

    #[test]
    #[should_panic(expected = "add produced identity")]
    fn add_integer_tuple_rejects_identity() {
        add_integer_tuple(0, (2, 3));
    }

    #[test]
    #[should_panic(expected = "add produced identity")]
    fn add_tuple_integer_rejects_identity() {
        add_tuple_integer((2, 3), 0);
    }

    #[test]
fn color_returns_first_object_color() {
    let object: Object = BTreeSet::from([
        (3, (4, 5)),
        (7, (1, 2)),
    ]);

    assert_eq!(color(object), 3);
}

    #[test]
    #[should_panic(expected = "empty Object")]
    fn color_rejects_empty_object() {
        color(BTreeSet::new());
    }

    #[test]
    fn equality_compares_integers() {
        assert!(equality_integer(3, 3));
        assert!(!equality_integer(3, 4));
    }

    #[test]
    fn equality_compares_grids() {
        let a: Grid = vec![vec![1, 2], vec![3, 4]];
        let b: Grid = vec![vec![1, 2], vec![3, 4]];
        let c: Grid = vec![vec![1, 2], vec![4, 3]];

        assert!(equality_grid(a.clone(), b));
        assert!(!equality_grid(a, c));
    }

    #[test]
    fn equality_compares_objects() {
        let a: Object = std::collections::BTreeSet::from([
            (1, (2, 3)),
            (4, (5, 6)),
        ]);

        let b = a.clone();

        let c: Object = std::collections::BTreeSet::from([
            (1, (2, 3)),
        ]);

        assert!(equality_object(a.clone(), b));
        assert!(!equality_object(a, c));
    }

    #[test]
    fn equality_compares_integer_vectors() {
        assert!(equality_integer_vector(
            vec![1, 2, 3],
            vec![1, 2, 3],
        ));

        assert!(!equality_integer_vector(
            vec![1, 2, 3],
            vec![1, 3, 2],
        ));
    }

    #[test]
    fn arithmetic_callable_families_exist() {
        for name in ["add", "subtract", "multiply", "divide"] {
            assert!(
                PRIMITIVES.iter().any(|primitive| {
                    primitive.name == name
                        && primitive.inputs.is_empty()
                        && primitive.output == Type::Callable
                }),
                "missing Callable primitive family: {}",
                name
            );
        }
    }

    #[test]
    fn size_matches_historical_cardinality() {
        assert_eq!(size_integer_tuple((4, 7)), 2);

        assert_eq!(
            size_integer_vector(vec![1, 2, 3]),
            3,
        );

        assert_eq!(
            size_grid(vec![
                vec![1, 2],
                vec![3, 4],
                vec![5, 6],
            ]),
            3,
        );

        let object: Object = std::collections::BTreeSet::from([
            (1, (2, 3)),
            (4, (5, 6)),
        ]);

        assert_eq!(size_object(object.clone()), 2);

        let objects: Objects =
            std::collections::BTreeSet::from([
                object.clone(),
                std::collections::BTreeSet::from([
                    (7, (1, 2)),
                ]),
            ]);

        assert_eq!(size_objects(objects), 2);

        let indices: Indices =
            std::collections::BTreeSet::from([
                (1, 2),
                (3, 4),
            ]);

        assert_eq!(size_indices(indices), 2);

        assert_eq!(
            size_object_vector(vec![object]),
            1,
        );
    }

    #[test]
    fn toindices_matches_historical_behavior() {
        let object: Object = std::collections::BTreeSet::from([
            (1, (2, 3)),
            (4, (5, 6)),
            (7, (2, 3)),
        ]);

        let expected: Indices =
            std::collections::BTreeSet::from([
                (2, 3),
                (5, 6),
            ]);

        assert_eq!(toindices_object(object), expected);

        let indices: Indices =
            std::collections::BTreeSet::from([
                (1, 2),
                (3, 4),
            ]);

        assert_eq!(
            toindices_indices(indices.clone()),
            indices,
        );

        assert_eq!(
            toindices_indices(Indices::new()),
            Indices::new(),
        );
    }

    #[test]
    fn uppermost_matches_historical_behavior() {
        let object: Object = std::collections::BTreeSet::from([
            (1, (4, 7)),
            (2, (2, 5)),
            (3, (6, 1)),
        ]);

        assert_eq!(uppermost_object(object), 2);

        let indices: Indices =
            std::collections::BTreeSet::from([
                (8, 3),
                (5, 9),
                (7, 1),
            ]);

        assert_eq!(uppermost_indices(indices), 5);
    }

    #[test]
    fn lowermost_matches_historical_behavior() {
        let object: Object = std::collections::BTreeSet::from([
            (1, (4, 7)),
            (2, (2, 5)),
            (3, (6, 1)),
        ]);

        assert_eq!(lowermost_object(object), 6);

        let indices: Indices =
            std::collections::BTreeSet::from([
                (8, 3),
                (5, 9),
                (7, 1),
            ]);

        assert_eq!(lowermost_indices(indices), 8);
    }

    #[test]
    fn leftmost_matches_historical_behavior() {
        let object: Object = std::collections::BTreeSet::from([
            (1, (4, 7)),
            (2, (2, 5)),
            (3, (6, 1)),
        ]);

        assert_eq!(leftmost_object(object), 1);

        let indices: Indices =
            std::collections::BTreeSet::from([
                (8, 3),
                (5, 9),
                (7, 1),
            ]);

        assert_eq!(leftmost_indices(indices), 1);
    }

    #[test]
    fn rightmost_matches_historical_behavior() {
        let object: Object = std::collections::BTreeSet::from([
            (1, (4, 7)),
            (2, (2, 5)),
            (3, (6, 1)),
        ]);

        assert_eq!(rightmost_object(object), 7);

        let indices: Indices =
            std::collections::BTreeSet::from([
                (8, 3),
                (5, 9),
                (7, 1),
            ]);

        assert_eq!(rightmost_indices(indices), 9);
    }

    #[test]
    fn shift_matches_historical_behavior() {
        let object: Object = std::collections::BTreeSet::from([
            (1, (2, 3)),
            (4, (5, 6)),
        ]);

        let expected_object: Object =
            std::collections::BTreeSet::from([
                (1, (4, 2)),
                (4, (7, 5)),
            ]);

        assert_eq!(
            shift_object(object, (2, -1)),
            expected_object,
        );

        let indices: Indices =
            std::collections::BTreeSet::from([
                (2, 3),
                (5, 6),
            ]);

        let expected_indices: Indices =
            std::collections::BTreeSet::from([
                (4, 2),
                (7, 5),
            ]);

        assert_eq!(
            shift_indices(indices, (2, -1)),
            expected_indices,
        );

        assert_eq!(
            shift_object(Object::new(), (3, 4)),
            Object::new(),
        );

        assert_eq!(
            shift_indices(Indices::new(), (3, 4)),
            Indices::new(),
        );
    }

    #[test]
    fn normalize_matches_historical_behavior() {
        let object: Object = std::collections::BTreeSet::from([
            (1, (4, 7)),
            (2, (6, 9)),
            (3, (5, 8)),
        ]);

        let expected: Object =
            std::collections::BTreeSet::from([
                (1, (0, 0)),
                (2, (2, 2)),
                (3, (1, 1)),
            ]);

        assert_eq!(
            normalize_object(object),
            expected,
        );

        let indices: Indices =
            std::collections::BTreeSet::from([
                (4, 7),
                (6, 9),
                (5, 8),
            ]);

        let expected_indices: Indices =
            std::collections::BTreeSet::from([
                (0, 0),
                (2, 2),
                (1, 1),
            ]);

        assert_eq!(
            normalize_indices(indices),
            expected_indices,
        );

        assert_eq!(
            normalize_object(Object::new()),
            Object::new(),
        );
    }

    #[test]
    fn height_matches_historical_behavior() {
        let grid: Grid = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
        ];

        assert_eq!(height_grid(grid), 3);

        let object: Object = std::collections::BTreeSet::from([
            (1, (2, 7)),
            (2, (4, 5)),
            (3, (3, 9)),
        ]);

        assert_eq!(height_object(object), 3);

        let indices: Indices =
            std::collections::BTreeSet::from([
                (2, 7),
                (4, 5),
                (3, 9),
            ]);

        assert_eq!(height_indices(indices), 3);

        assert_eq!(height_grid(Grid::new()), 0);
    }

    #[test]
    fn width_matches_historical_behavior() {
        let grid: Grid = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
        ];

        assert_eq!(width_grid(grid), 4);

        let object: Object = std::collections::BTreeSet::from([
            (1, (2, 7)),
            (2, (4, 5)),
            (3, (3, 9)),
        ]);

        assert_eq!(width_object(object), 5);

        let indices: Indices =
            std::collections::BTreeSet::from([
                (2, 7),
                (4, 5),
                (3, 9),
            ]);

        assert_eq!(width_indices(indices), 5);
    }

    #[test]
    fn hperiod_matches_historical_behavior() {
        let periodic: Object =
            std::collections::BTreeSet::from([
                (1, (0, 0)),
                (1, (0, 2)),
            ]);

        assert_eq!(hperiod(periodic), 2);

        let non_periodic: Object =
            std::collections::BTreeSet::from([
                (1, (0, 0)),
                (1, (0, 3)),
            ]);

        assert_eq!(hperiod(non_periodic), 3);
    }

    #[test]
    fn vperiod_matches_historical_behavior() {
        let periodic: Object =
            std::collections::BTreeSet::from([
                (1, (0, 0)),
                (1, (2, 0)),
            ]);

        assert_eq!(vperiod(periodic), 2);

        let non_periodic: Object =
            std::collections::BTreeSet::from([
                (1, (0, 0)),
                (1, (3, 0)),
            ]);

        assert_eq!(vperiod(non_periodic), 3);
    }

    #[test]
    fn ulcorner_matches_historical_behavior() {
        let object: Object =
            std::collections::BTreeSet::from([
                (1, (4, 7)),
                (2, (2, 5)),
                (3, (6, 9)),
            ]);

        assert_eq!(
            ulcorner_object(object),
            (2, 5),
        );

        let indices: Indices =
            std::collections::BTreeSet::from([
                (4, 7),
                (2, 5),
                (6, 9),
            ]);

        assert_eq!(
            ulcorner_indices(indices),
            (2, 5),
        );
    }

    #[test]
    fn urcorner_matches_historical_behavior() {
        let object: Object =
            std::collections::BTreeSet::from([
                (1, (4, 7)),
                (2, (2, 5)),
                (3, (6, 9)),
            ]);

        assert_eq!(
            urcorner_object(object),
            (2, 9),
        );

        let indices: Indices =
            std::collections::BTreeSet::from([
                (4, 7),
                (2, 5),
                (6, 9),
            ]);

        assert_eq!(
            urcorner_indices(indices),
            (2, 9),
        );
    }

    #[test]
    fn llcorner_matches_historical_behavior() {
        let object: Object =
            std::collections::BTreeSet::from([
                (1, (4, 7)),
                (2, (2, 5)),
                (3, (6, 9)),
            ]);

        assert_eq!(
            llcorner_object(object),
            (6, 5),
        );

        let indices: Indices =
            std::collections::BTreeSet::from([
                (4, 7),
                (2, 5),
                (6, 9),
            ]);

        assert_eq!(
            llcorner_indices(indices),
            (6, 5),
        );
    }

    #[test]
    fn lrcorner_matches_historical_behavior() {
        let object: Object =
            std::collections::BTreeSet::from([
                (1, (4, 7)),
                (2, (2, 5)),
                (3, (6, 9)),
            ]);

        assert_eq!(
            lrcorner_object(object),
            (6, 9),
        );

        let indices: Indices =
            std::collections::BTreeSet::from([
                (4, 7),
                (2, 5),
                (6, 9),
            ]);

        assert_eq!(
            lrcorner_indices(indices),
            (6, 9),
        );
    }

    #[test]
    fn rot90_matches_historical_behavior() {
        let grid = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
        ];

        assert_eq!(
            rot90(grid),
            vec![
                vec![4, 1],
                vec![5, 2],
                vec![6, 3],
            ],
        );

        let square = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        assert_eq!(
            rot90(square),
            vec![
                vec![3, 1],
                vec![4, 2],
            ],
        );
    }

    #[test]
    fn rot180_matches_historical_behavior() {
        let grid = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
        ];

        assert_eq!(
            rot180(grid),
            vec![
                vec![6, 5, 4],
                vec![3, 2, 1],
            ],
        );

        let square = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        assert_eq!(
            rot180(square),
            vec![
                vec![4, 3],
                vec![2, 1],
            ],
        );
    }

    #[test]
    fn rot270_matches_historical_behavior() {
        let grid = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
        ];

        assert_eq!(
            rot270(grid),
            vec![
                vec![3, 6],
                vec![2, 5],
                vec![1, 4],
            ],
        );

        let square = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        assert_eq!(
            rot270(square),
            vec![
                vec![2, 4],
                vec![1, 3],
            ],
        );
    }
}
