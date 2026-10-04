use std::collections::BTreeSet;

pub type Boolean = bool;
pub type Integer = i16;
pub type IntegerTuple = (Integer, Integer);

pub type Grid = Vec<Vec<Integer>>;

pub type Cell = (Integer, IntegerTuple);

pub type Object = BTreeSet<Cell>;
pub type Indices = BTreeSet<IntegerTuple>;
