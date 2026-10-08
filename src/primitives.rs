use crate::types::*;
use primitive_macro::primitive;

const MAX_SIZE: usize = 30;

#[primitive("hsplit")]
pub fn hsplit(grid: Grid, n: Integer) -> GridVector {
    if n <= 0 {
        panic!("Wrong value");
    }

    let h = grid.len();
    if h == 0 {
        panic!("Wrong value");
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
        panic!("Wrong value");
    }

    let h = grid.len();
    if h == 0 {
        panic!("Wrong value");
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
        panic!("Wrong value");
    }

    let w = a[0].len();

    if w == 0 || b.iter().any(|row| row.len() != w) {
        panic!("Wrong value");
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
        panic!("Wrong value");
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
        panic!("Wrong value");
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
        panic!("Wrong value");
    }

    result
}

#[primitive("hupscale")]
pub fn hupscale(grid: Grid, factor: Integer) -> Grid {
    if factor <= 1 || factor > 10 {
        panic!("Wrong value");
    }

    if grid.len() > MAX_SIZE
        || grid.first().map_or(false, |row| row.len() > MAX_SIZE)
    {
        panic!("Wrong value");
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
        panic!("Wrong value");
    }

    result
}

#[primitive("vupscale")]
pub fn vupscale(grid: Grid, factor: Integer) -> Grid {
    if factor <= 1 || factor > 10 {
        panic!("Wrong value");
    }

    if grid.len() > MAX_SIZE
        || grid.first().map_or(false, |row| row.len() > MAX_SIZE)
    {
        panic!("Wrong value");
    }

    let mut result =
        Vec::with_capacity(grid.len() * factor as usize);

    for row in &grid {
        for _ in 0..factor {
            result.push(row.clone());
        }
    }

    if result == grid {
        panic!("Wrong value");
    }

    result
}

#[primitive("upscale")]
fn upscale_grid(grid: Grid, factor: Integer) -> Grid {
    if grid.len() > MAX_SIZE
        || grid.first().map_or(false, |row| row.len() > MAX_SIZE)
    {
        panic!("Wrong value");
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
        panic!("Wrong value");
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
        panic!("Wrong value");
    }

    let factor = factor as usize;

    let h = grid.len();

    if h == 0 {
        panic!("Wrong value");
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
        panic!("Wrong value");
    }

    result
}

#[primitive("tophalf")]
pub fn tophalf(grid: Grid) -> Grid {
    if grid.is_empty() {
        panic!("Wrong value");
    }

    let mid = grid.len() / 2;

    grid[..mid].to_vec()
}

#[primitive("bottomhalf")]
pub fn bottomhalf(grid: Grid) -> Grid {
    if grid.is_empty() {
        panic!("Wrong value");
    }

    let mid = grid.len().div_ceil(2);

    grid[mid..].to_vec()
}

#[primitive("lefthalf")]
pub fn lefthalf(grid: Grid) -> Grid {
    let result = rot270(tophalf(rot90(grid.clone())));

    if grid == result {
        panic!("Wrong value");
    }

    result
}

#[primitive("righthalf")]
pub fn righthalf(grid: Grid) -> Grid {
    rot270(bottomhalf(rot90(grid)))
}

#[primitive("trim")]
pub fn trim(grid: Grid) -> Grid {
    if grid.len() < 3 {
        panic!("Wrong value");
    }

    grid[1..grid.len() - 1]
        .iter()
        .map(|row| {
            if row.len() < 3 {
                panic!("Wrong value");
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
        panic!("Wrong value");
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
