use crate::types::{Grid, Indices, Integer, IntegerTuple};

pub fn add(a: Integer, b: Integer) -> Integer {
    a + b
}

pub fn hmirror(mut grid: Grid) -> Grid {
    for row in &mut grid {
        row.reverse();
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
}
