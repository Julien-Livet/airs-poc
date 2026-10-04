use crate::types::{Grid, Integer};

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
