use std::collections::BTreeSet;

pub type Boolean = bool;
pub type Integer = i16;
pub type IntegerTuple = (Integer, Integer);
pub type Grid = Vec<Vec<Integer>>;
pub type Cell = (Integer, IntegerTuple);
pub type Object = BTreeSet<Cell>;
pub type Indices = BTreeSet<IntegerTuple>;

pub type IntegerVector = Vec<Integer>;
pub type GridVector = Vec<Grid>;
pub type TupleVector = Vec<IntegerTuple>;
pub type ObjectVector = Vec<Object>;
pub type BooleanVector = Vec<Boolean>;
pub type ObjectsVector = Vec<Objects>;
pub type IndicesVector = Vec<Indices>;

pub type Objects = BTreeSet<Object>;
pub type IntegerSet = BTreeSet<Integer>;
pub type IndicesSet = BTreeSet<Indices>;
