use crate::types::*;
use primitive_macro::primitive;

#[primitive("equality")]
pub fn equality_boolean(a: Boolean, b: Boolean) -> Boolean {
    a == b
}

#[primitive("equality")]
pub fn equality_integer(a: Integer, b: Integer) -> Boolean {
    a == b
}

#[primitive("equality")]
pub fn equality_integer_tuple(
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
}
