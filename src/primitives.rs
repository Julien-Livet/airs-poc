use crate::types::{Grid, Indices, Integer, IntegerTuple, Object};

pub fn flip(value: bool) -> bool {
    !value
}

pub fn add(a: Integer, b: Integer) -> Integer {
    let c = a + b;

    if c == a || c == b {
        panic!("add produced identity");
    }

    c
}

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

pub fn vmirror(mut grid: Grid) -> Grid {
    grid.reverse();
    grid
}

pub fn vconcat(mut top: Grid, bottom: Grid) -> Grid {
    top.extend(bottom);
    top
}

pub fn ulcorner(indices: Indices) -> IntegerTuple {
    indices
        .iter()
        .copied()
        .reduce(|(min_y, min_x), (y, x)| {
            (min_y.min(y), min_x.min(x))
        })
        .expect("empty Indices")
}

pub fn urcorner(indices: Indices) -> IntegerTuple {
    indices
        .iter()
        .copied()
        .reduce(|(min_y, max_x), (y, x)| {
            (min_y.min(y), max_x.max(x))
        })
        .expect("empty Indices")
}

pub fn llcorner(indices: Indices) -> IntegerTuple {
    indices
        .iter()
        .copied()
        .reduce(|(max_y, min_x), (y, x)| {
            (max_y.max(y), min_x.min(x))
        })
        .expect("empty Indices")
}

pub fn lrcorner(indices: Indices) -> IntegerTuple {
    indices
        .iter()
        .copied()
        .reduce(|(max_y, max_x), (y, x)| {
            (max_y.max(y), max_x.max(x))
        })
        .expect("empty Indices")
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

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
}
