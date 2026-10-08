use linkme::distributed_slice;
use std::collections::{BTreeSet, HashMap};

use crate::connection::Connection;
use crate::types::*;
use crate::function::Function;

macro_rules! define_callable_primitive {
    (
        $entry:ident,
        $apply:ident,
        $name:literal
    ) => {
        #[linkme::distributed_slice(PRIMITIVES)]
        #[allow(non_upper_case_globals)]
        pub static $entry: PrimitiveEntry = PrimitiveEntry {
            name: $name,
            inputs: &[],
            output: Type::Callable,
            apply: $apply,
        };

        fn $apply(
            _values: &[Value],
        ) -> Result<Value, String> {
            Ok(Value::Function(Box::new(
                Function::primitive_family(&$entry),
            )))
        }
    };
}

define_callable_primitive!(halve_callable_entry, halve_callable_apply, "halve"); //scaling by one half
define_callable_primitive!(double_callable_entry, double_callable_apply, "double"); //scaling by two
define_callable_primitive!(even_callable_entry, even_callable_apply, "even"); //evenness
define_callable_primitive!(add_callable_entry, add_callable_apply, "add"); //addition
define_callable_primitive!(subtract_callable_entry, subtract_callable_apply, "subtract"); //subtraction
define_callable_primitive!(multiply_callable_entry, multiply_callable_apply, "multiply"); //multiplication
define_callable_primitive!(divide_callable_entry, divide_callable_apply, "divide"); //floor division
define_callable_primitive!(invert_callable_entry, invert_callable_apply, "invert"); //inversion with respect to addition
define_callable_primitive!(tojvec_callable_entry, tojvec_callable_apply, "tojvec"); //vector pointing horizontally
define_callable_primitive!(toivec_callable_entry, toivec_callable_apply, "toivec"); //vector pointing vertically
define_callable_primitive!(portrait_callable_entry, portrait_callable_apply, "portrait"); //whether height is greater than width
define_callable_primitive!(shape_callable_entry, shape_callable_apply, "shape"); //height and width of grid or patch 
define_callable_primitive!(either_callable_entry, either_callable_apply, "either"); //logical or
define_callable_primitive!(both_callable_entry, both_callable_apply, "both"); //logical and
define_callable_primitive!(vconcat_callable_entry, vconcat_callable_apply, "vconcat"); //concatenate two grids vertically
define_callable_primitive!(hconcat_callable_entry, hconcat_callable_apply, "hconcat"); //concatenate two grids horizontally
define_callable_primitive!(flip_callable_entry, flip_callable_apply, "flip"); //logical not
define_callable_primitive!(color_callable_entry, color_callable_apply, "color"); //color of object
define_callable_primitive!(toindices_callable_entry, toindices_callable_apply, "toindices"); //indices of object cells
define_callable_primitive!(uppermost_callable_entry, uppermost_callable_apply, "uppermost"); //row index of uppermost occupied cell
define_callable_primitive!(lowermost_callable_entry, lowermost_callable_apply, "lowermost"); //row index of lowermost occupied cell
define_callable_primitive!(leftmost_callable_entry, leftmost_callable_apply, "leftmost"); //column index of leftmost occupied cell
define_callable_primitive!(rightmost_callable_entry, rightmost_callable_apply, "rightmost"); //column index of rightmost occupied cell
define_callable_primitive!(shift_callable_entry, shift_callable_apply, "shift"); //shift patch
define_callable_primitive!(normalize_callable_entry, normalize_callable_apply, "normalize"); //moves upper left corner to origin
define_callable_primitive!(height_callable_entry, height_callable_apply, "height"); //height of grid or patch
define_callable_primitive!(width_callable_entry, width_callable_apply, "width"); //width of grid or patch
define_callable_primitive!(hperiod_callable_entry, hperiod_callable_apply, "hperiod"); //horizontal periodicity
define_callable_primitive!(vperiod_callable_entry, vperiod_callable_apply, "vperiod"); //vertical periodicity
define_callable_primitive!(ulcorner_callable_entry, ulcorner_callable_apply, "ulcorner"); //index of upper left corner
define_callable_primitive!(urcorner_callable_entry, urcorner_callable_apply, "urcorner"); //index of upper right corner
define_callable_primitive!(llcorner_callable_entry, llcorner_callable_apply, "llcorner"); //index of lower left corner
define_callable_primitive!(lrcorner_callable_entry, lrcorner_callable_apply, "lrcorner"); //index of lower right corner
define_callable_primitive!(crop_callable_entry, crop_callable_apply, "crop"); //subgrid specified by start and dimension
define_callable_primitive!(rot270_callable_entry, rot270_callable_apply, "rot270"); //quarter anticlockwise rotation
define_callable_primitive!(rot180_callable_entry, rot180_callable_apply, "rot180"); //half rotation
define_callable_primitive!(rot90_callable_entry, rot90_callable_apply, "rot90"); //quarter clockwise rotation
define_callable_primitive!(dmirror_callable_entry, dmirror_callable_apply, "dmirror"); //mirroring along diagonal
define_callable_primitive!(cmirror_callable_entry, cmirror_callable_apply, "cmirror"); //mirroring along counterdiagonal
define_callable_primitive!(hmirror_callable_entry, hmirror_callable_apply, "hmirror"); //mirroring along horizontal
define_callable_primitive!(vmirror_callable_entry, vmirror_callable_apply, "vmirror"); //mirroring along vertical
define_callable_primitive!(size_callable_entry, size_callable_apply, "size"); //cardinality
define_callable_primitive!(equality_callable_entry, equality_callable_apply, "equality"); //equality
define_callable_primitive!(identity_callable_entry, identity_callable_apply, "identity"); //identity function

pub fn apply_primitive_family(
    name: &str,
    arguments: &[Value],
) -> Result<Value, String> {
    for primitive in primitives_by_name(name) {
        let input_types = arguments
            .iter()
            .map(Value::output_type)
            .collect::<Vec<_>>();

        if !primitive.accepts(&input_types) {
            continue;
        }

        return (primitive.apply)(arguments);
    }

    Err(format!(
        "no overload of '{}' accepts the given arguments",
        name
    ))
}

pub fn primitives_by_name(
    name: &str,
) -> Vec<&'static PrimitiveEntry> {
    PRIMITIVES
        .iter()
        .filter(|primitive| primitive.name == name)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicPrimitive {
    Lbind,
    Apply,
    Rbind,
    HistoricalApply,
}

impl DynamicPrimitive {
    pub fn name(self) -> &'static str {
        match self {
            DynamicPrimitive::Lbind => "lbind",
            DynamicPrimitive::Rbind => "rbind",
            DynamicPrimitive::Apply => "apply",
            DynamicPrimitive::HistoricalApply => "apply",
        }
    }
}

pub fn lbind_value(
    function: &Value,
    fixed: &Value,
) -> Result<Value, String> {
    let function = match function {
        Value::Function(function) => function,
        _ => {
            return Err(
                "lbind expects a Function".to_string()
            );
        }
    };

    let fixed = Connection::terminal(
        fixed.clone(),
    );

    Ok(Value::Function(Box::new(
        function.lbind(fixed)?,
    )))
}

pub fn rbind_value(
    function: &Value,
    fixed: &Value,
) -> Result<Value, String> {
    let function = match function {
        Value::Function(function) => function,
        _ => {
            return Err(
                "rbind expects a Function".to_string()
            );
        }
    };

    let fixed = Connection::terminal(
        fixed.clone(),
    );

    Ok(Value::Function(Box::new(
        function.rbind(fixed)?,
    )))
}

pub fn apply_value(
    function: &Value,
    argument: &Value,
) -> Result<Value, String> {
    apply_values(
        function,
        vec![argument.clone()],
    )
}

pub fn apply_values(
    function: &Value,
    arguments: Vec<Value>,
) -> Result<Value, String> {
    let function = match function {
        Value::Function(function) => function,
        _ => {
            return Err(
                "apply expects a Function".to_string()
            );
        }
    };

    function.apply_values(arguments)
}

pub fn map_apply_value(
    function: &Value,
    container: &Value,
) -> Result<Value, String> {
    if let Value::IntegerTuple((a, b)) = container {
        let first = apply_value(
            function,
            &Value::Integer(*a),
        )?;

        let second = apply_value(
            function,
            &Value::Integer(*b),
        )?;

        let first = Integer::from_value(&first)?;
        let second = Integer::from_value(&second)?;

        return Ok(Value::IntegerVector(vec![
            first,
            second,
        ]));
    }

    if let Value::IntegerVector(values) = container {
        let mut result = Vec::with_capacity(values.len());

        for value in values {
            let mapped = apply_value(
                function,
                &Value::Integer(*value),
            )?;

            result.push(Integer::from_value(&mapped)?);
        }

        return Ok(Value::IntegerVector(result));
    }

    if let Value::GridVector(values) = container {
        let mut integers = Vec::with_capacity(values.len());
        let mut grids = Vec::with_capacity(values.len());

        for value in values {
            let mapped = apply_value(
                function,
                &Value::Grid(value.clone()),
            )?;

            match mapped {
                Value::Integer(value) => {
                    integers.push(value);
                }

                Value::Grid(value) => {
                    grids.push(value);
                }

                other => {
                    return Err(format!(
                        "historical apply: unsupported mapped output {:?}",
                        other
                    ));
                }
            }
        }

        if integers.len() == values.len() {
            return Ok(Value::IntegerVector(integers));
        }

        if grids.len() == values.len() {
            return Ok(Value::GridVector(grids));
        }

        return Err(
            "historical apply: mixed mapped output types"
                .to_string(),
        );
    }

    if let Value::ObjectVector(values) = container {
        let mut result = Vec::with_capacity(values.len());

        for value in values {
            let mapped = apply_value(
                function,
                &Value::Object(value.clone()),
            )?;

            result.push(Object::from_value(&mapped)?);
        }

        return Ok(Value::ObjectVector(result));
    }

    if let Value::Objects(values) = container {
        let mut objects = BTreeSet::new();
        let mut integers = Vec::with_capacity(values.len());

        for value in values {
            let mapped = apply_value(
                function,
                &Value::Object(value.clone()),
            )?;

            match mapped {
                Value::Object(value) => {
                    objects.insert(value);
                }

                Value::Integer(value) => {
                    integers.push(value);
                }

                other => {
                    return Err(format!(
                        "historical apply: unsupported mapped output {:?}",
                        other
                    ));
                }
            }
        }

        if objects.len() == values.len() {
            return Ok(Value::Objects(objects));
        }

        if integers.len() == values.len() {
            return Ok(Value::IntegerVector(integers));
        }

        return Err(
            "historical apply: mixed mapped output types"
                .to_string(),
        );
    }

    Err(
        "historical apply: unsupported container"
            .to_string(),
    )
}

#[derive(Debug, Default)]
pub struct FunctionTypeRegistry {
    types: Vec<FunctionType>,
    ids: HashMap<FunctionType, FunctionTypeId>,
}

impl FunctionTypeRegistry {
    pub fn inputs(&self, id: FunctionTypeId) -> Option<&[Type]> {
        self.get(id).map(|function| {
            function.inputs.as_slice()
        })
    }

    pub fn output(&self, id: FunctionTypeId) -> Option<Type> {
        self.get(id).map(|function| function.output)
    }

    pub fn all_types(&self) -> impl Iterator<Item = (FunctionTypeId, &FunctionType)> {
        self.types
            .iter()
            .enumerate()
            .map(|(index, function_type)| {
                (
                    FunctionTypeId(index as u32),
                    function_type,
                )
            })
    }

    pub fn type_of(
        &mut self,
        inputs: &[Type],
        output: Type,
    ) -> Type {
        Type::Function(self.intern(inputs, output))
    }

    pub fn lbind_type(
        &mut self,
        function_type: Type,
        fixed_type: Type,
    ) -> Result<Type, String> {
        let id = match function_type {
            Type::Function(id) => id,
            other => {
                return Err(format!(
                    "lbind expects a function, got {:?}",
                    other
                ));
            }
        };

        let (remaining, output) = {
            let function = self
                .get(id)
                .ok_or_else(|| "unknown function type".to_string())?;

            if function.inputs.is_empty() {
                return Err(
                    "cannot lbind a function with no inputs".to_string()
                );
            }

            if function.inputs[0] != fixed_type {
                return Err(format!(
                    "lbind type mismatch: expected {:?}, got {:?}",
                    function.inputs[0],
                    fixed_type
                ));
            }

            let remaining = &function.inputs[1..];

            (remaining.to_vec(), function.output)
        };

        Ok(self.type_of(&remaining, output))
    }

    pub fn rbind_type(
        &mut self,
        function_type: Type,
        fixed_type: Type,
    ) -> Result<Type, String> {
        let id = match function_type {
            Type::Function(id) => id,
            other => {
                return Err(format!(
                    "rbind expects a function, got {:?}",
                    other
                ));
            }
        };

        let (remaining, output) = {
            let function = self
                .get(id)
                .ok_or_else(|| "unknown function type".to_string())?;

            if function.inputs.is_empty() {
                return Err(
                    "cannot rbind a function with no inputs".to_string()
                );
            }

            let last = function.inputs.len() - 1;

            if function.inputs[last] != fixed_type {
                return Err(format!(
                    "rbind type mismatch: expected {:?}, got {:?}",
                    function.inputs[last],
                    fixed_type
                ));
            }

            let remaining = function.inputs[..last].to_vec();

            (remaining, function.output)
        };

        Ok(self.type_of(&remaining, output))
    }

    pub fn apply_type(
        &self,
        function_type: Type,
        argument_types: &[Type],
    ) -> Result<Type, String> {
        let id = match function_type {
            Type::Function(id) => id,

            Type::Callable => {
                return Ok(Type::Callable);
            }

            other => {
                return Err(format!(
                    "apply expects a function, got {:?}",
                    other
                ));
            }
        };

        let function = self
            .get(id)
            .ok_or_else(|| "unknown function type".to_string())?;

        if function.inputs.len() != argument_types.len() {
            return Err(format!(
                "apply argument count mismatch: expected {}, got {}",
                function.inputs.len(),
                argument_types.len()
            ));
        }

        for (expected, actual) in
            function.inputs.iter().zip(argument_types.iter())
        {
            if expected != actual {
                return Err(format!(
                    "apply type mismatch: expected {:?}, got {:?}",
                    expected,
                    actual
                ));
            }
        }

        Ok(function.output)
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(
        &mut self,
        inputs: &[Type],
        output: Type,
    ) -> FunctionTypeId {
        let function_type = FunctionType {
            inputs: inputs.to_vec(),
            output,
        };

        if let Some(&id) = self.ids.get(&function_type) {
            return id;
        }

        let id = FunctionTypeId(self.types.len() as u32);

        self.types.push(function_type.clone());
        self.ids.insert(function_type, id);

        id
    }

    pub fn get(
        &self,
        id: FunctionTypeId,
    ) -> Option<&FunctionType> {
        self.types.get(id.0 as usize)
    }


    pub fn apply_callable_family(
        &mut self,
        name: &str,
        arguments: &[Type],
    ) -> Result<Type, String> {
        for primitive in crate::registry::primitives_by_name(name) {
            if primitive.accepts(arguments) {
                return Ok(primitive.output);
            }
        }

        Err(format!(
            "no overload of '{}' accepts the given arguments",
            name
        ))
    }
}

pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self, String>;
}

pub trait IntoValue {
    fn into_value(self) -> Value;
}

impl FromValue for Integer {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Integer(value) => Ok(*value),
            _ => Err("expected Integer".to_string()),
        }
    }
}

impl IntoValue for Integer {
    fn into_value(self) -> Value {
        Value::Integer(self)
    }
}

impl FromValue for Grid {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Grid(grid) => Ok(grid.clone()),
            _ => Err("expected Grid".to_string()),
        }
    }
}

impl IntoValue for Grid {
    fn into_value(self) -> Value {
        Value::Grid(self)
    }
}

impl FromValue for IntegerTuple {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::IntegerTuple(value) => Ok(*value),
            _ => Err("expected IntegerTuple".to_string()),
        }
    }
}

impl IntoValue for IntegerTuple {
    fn into_value(self) -> Value {
        Value::IntegerTuple(self)
    }
}

impl FromValue for Indices {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Indices(value) => Ok(value.clone()),
            _ => Err("expected Indices".to_string()),
        }
    }
}

impl IntoValue for Indices {
    fn into_value(self) -> Value {
        Value::Indices(self)
    }
}

impl FromValue for Object {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Object(value) => Ok(value.clone()),
            _ => Err("expected Object".to_string()),
        }
    }
}

impl IntoValue for Object {
    fn into_value(self) -> Value {
        Value::Object(self)
    }
}

impl FromValue for Boolean {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Boolean(value) => Ok(*value),
            _ => Err("expected Boolean".to_string()),
        }
    }
}

impl IntoValue for Boolean {
    fn into_value(self) -> Value {
        Value::Boolean(self)
    }
}

impl FromValue for Function {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Function(function) => Ok((**function).clone()),
            _ => Err("expected Function".to_string()),
        }
    }
}

impl IntoValue for Function {
    fn into_value(self) -> Value {
        Value::Function(Box::new(self))
    }
}

impl FromValue for IntegerVector {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::IntegerVector(value) => Ok(value.clone()),
            _ => Err("expected IntegerVector".to_string()),
        }
    }
}

impl IntoValue for IntegerVector {
    fn into_value(self) -> Value {
        Value::IntegerVector(self)
    }
}

impl FromValue for GridVector {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::GridVector(value) => Ok(value.clone()),
            _ => Err("expected GridVector".to_string()),
        }
    }
}

impl IntoValue for GridVector {
    fn into_value(self) -> Value {
        Value::GridVector(self)
    }
}

impl FromValue for TupleVector {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::TupleVector(value) => Ok(value.clone()),
            _ => Err("expected TupleVector".to_string()),
        }
    }
}

impl IntoValue for TupleVector {
    fn into_value(self) -> Value {
        Value::TupleVector(self)
    }
}

impl FromValue for ObjectVector {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::ObjectVector(value) => Ok(value.clone()),
            _ => Err("expected ObjectVector".to_string()),
        }
    }
}

impl IntoValue for ObjectVector {
    fn into_value(self) -> Value {
        Value::ObjectVector(self)
    }
}

impl FromValue for Objects {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Objects(value) => Ok(value.clone()),
            _ => Err("expected Objects".to_string()),
        }
    }
}

impl IntoValue for Objects {
    fn into_value(self) -> Value {
        Value::Objects(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub inputs: Vec<Type>,
    pub output: Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionTypeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Boolean,
    Integer,
    Grid,
    IntegerTuple,
    Indices,
    Object,
    IntegerVector,
    GridVector,
    TupleVector,
    Function(FunctionTypeId),
    Callable,
    ObjectVector,
    Objects,
}

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(Boolean),
    Integer(Integer),
    Grid(Grid),
    IntegerTuple(IntegerTuple),
    Indices(Indices),
    Object(Object),
    Function(Box<Function>),
    IntegerVector(IntegerVector),
    GridVector(GridVector),
    TupleVector(TupleVector),
    ObjectVector(ObjectVector),
    Objects(Objects),
}

#[derive(Debug)]
pub struct PrimitiveEntry {
    pub name: &'static str,
    pub inputs: &'static [Type],
    pub output: Type,
    pub apply: fn(&[Value]) -> Result<Value, String>,
}

impl PrimitiveEntry {
    pub fn accepts(&self, inputs: &[Type]) -> bool {
        self.inputs == inputs
    }

    #[allow(dead_code)]
    pub fn produces(&self, output: Type) -> bool {
        self.output == output
    }
}

#[distributed_slice]
pub static PRIMITIVES: [PrimitiveEntry];

pub fn find_compatible(inputs: &[Type]) -> Vec<&'static PrimitiveEntry> {
    PRIMITIVES
        .iter()
        .filter(|primitive| primitive.accepts(inputs))
        .collect()
}

#[cfg(test)]
pub fn find_by_name_and_inputs(
    name: &str,
    inputs: &[Type],
) -> Option<&'static PrimitiveEntry> {
    PRIMITIVES
        .iter()
        .find(|primitive| {
            primitive.name == name
                && primitive.accepts(inputs)
        })
}

pub fn find_primitive(
    name: &str,
    inputs: &[Type],
    output: Type,
) -> &'static PrimitiveEntry {
    PRIMITIVES
        .iter()
        .find(|primitive| {
            primitive.name == name
                && primitive.inputs == inputs
                && primitive.output == output
        })
        .expect("primitive should be registered")
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::connection::Connection;
    use crate::signature::InputSpec;
    use crate::function::Function;

    #[test]
    fn registry_contains_expected_primitives() {
        assert_eq!(PRIMITIVES.len(), 153);

        assert!(
            PRIMITIVES.iter().any(|primitive| {
                primitive.name == "crop"
                    && primitive.inputs
                        == [
                            Type::Grid,
                            Type::IntegerTuple,
                            Type::IntegerTuple,
                        ]
                    && primitive.output == Type::Grid
            })
        );

        assert!(
            PRIMITIVES.iter().any(|primitive| {
                primitive.name == "ulcorner"
                    && primitive.inputs == [Type::Indices]
                    && primitive.output == Type::IntegerTuple
            })
        );

        assert!(
            PRIMITIVES.iter().any(|primitive| {
                primitive.name == "urcorner"
                    && primitive.inputs == [Type::Indices]
                    && primitive.output == Type::IntegerTuple
            })
        );

        assert!(
            PRIMITIVES.iter().any(|primitive| {
                primitive.name == "llcorner"
                    && primitive.inputs == [Type::Indices]
                    && primitive.output == Type::IntegerTuple
            })
        );

        assert!(
            PRIMITIVES.iter().any(|primitive| {
                primitive.name == "lrcorner"
                    && primitive.inputs == [Type::Indices]
                    && primitive.output == Type::IntegerTuple
            })
        );

        assert!(PRIMITIVES.iter().any(|p| {
            p.name == "add"
                && p.inputs == &[Type::Integer, Type::Integer]
                && p.output == Type::Integer
        }));

        assert!(PRIMITIVES.iter().any(|p| {
            p.name == "hmirror"
                && p.inputs == &[Type::Grid]
                && p.output == Type::Grid
        }));

        assert!(PRIMITIVES.iter().any(|p| {
            p.name == "vmirror"
                && p.inputs == &[Type::Grid]
                && p.output == Type::Grid
        }));

        assert!(PRIMITIVES.iter().any(|p| {
            p.name == "vconcat"
                && p.inputs == &[Type::Grid, Type::Grid]
                && p.output == Type::Grid
        }));
    }

    #[test]
    fn registered_primitive_can_be_applied_dynamically() {
        let primitive = find_by_name_and_inputs(
            "hmirror",
            &[Type::Grid],
        )
        .unwrap();

        let result = (primitive.apply)(&[
            Value::Grid(vec![
                vec![1, 2],
                vec![3, 4],
            ]),
        ])
        .unwrap();

        assert_eq!(
            result.output_type(),
            Type::Grid,
        );

        match result {
            Value::Grid(grid) => {
                assert_eq!(
                    grid,
                    vec![
                        vec![2, 1],
                        vec![4, 3],
                    ],
                );
            }
            _ => panic!("expected Grid"),
        }
    }

    #[test]
    fn registered_corner_can_be_applied_dynamically() {
        let primitive = find_primitive(
            "ulcorner",
            &[Type::Indices],
            Type::IntegerTuple,
        );

        let indices = std::collections::BTreeSet::from([
            (2, 5),
            (4, 1),
            (7, 9),
        ]);

        let result = (primitive.apply)(&[
            Value::Indices(indices),
        ])
        .expect("ulcorner should apply successfully");

        assert!(matches!(
            result,
            Value::IntegerTuple((2, 1))
        ));
    }

    #[test]
    fn registry_contains_grid_and_indices_hmirror() {
        let mut grid = PRIMITIVES
            .iter()
            .filter(|primitive| primitive.accepts(&[Type::Grid]));

        assert!(
            grid.any(|primitive| {
                primitive.name == "hmirror"
                    && primitive.output == Type::Grid
            })
        );

        let mut indices = PRIMITIVES
            .iter()
            .filter(|primitive| primitive.accepts(&[Type::Indices]));

        assert!(
            indices.any(|primitive| {
                primitive.name == "hmirror"
                    && primitive.output == Type::Indices
            })
        );
    }

    #[test]
    fn registry_resolves_all_add_overloads_by_input_signature() {
        let integer_integer =
            find_by_name_and_inputs(
                "add",
                &[Type::Integer, Type::Integer],
            )
            .unwrap();

        assert_eq!(integer_integer.inputs, &[Type::Integer, Type::Integer]);
        assert_eq!(integer_integer.output, Type::Integer);

        let tuple_tuple =
            find_by_name_and_inputs(
                "add",
                &[Type::IntegerTuple, Type::IntegerTuple],
            )
            .unwrap();

        assert_eq!(
            tuple_tuple.inputs,
            &[Type::IntegerTuple, Type::IntegerTuple]
        );
        assert_eq!(tuple_tuple.output, Type::IntegerTuple);

        let integer_tuple =
            find_by_name_and_inputs(
                "add",
                &[Type::Integer, Type::IntegerTuple],
            )
            .unwrap();

        assert_eq!(
            integer_tuple.inputs,
            &[Type::Integer, Type::IntegerTuple]
        );
        assert_eq!(integer_tuple.output, Type::IntegerTuple);

        let tuple_integer =
            find_by_name_and_inputs(
                "add",
                &[Type::IntegerTuple, Type::Integer],
            )
            .unwrap();

        assert_eq!(
            tuple_integer.inputs,
            &[Type::IntegerTuple, Type::Integer]
        );
        assert_eq!(tuple_integer.output, Type::IntegerTuple);
    }

    #[test]
    fn value_reports_its_type() {
        assert_eq!(Value::Integer(2).ty(), Type::Integer);
        assert_eq!(
            Value::IntegerTuple((1, 2)).ty(),
            Type::IntegerTuple
        );
    }

    #[test]
    fn function_types_are_interned() {
        let mut registry = FunctionTypeRegistry::new();

        let integer_to_integer =
            registry.intern(&[Type::Integer], Type::Integer);

        let same =
            registry.intern(&[Type::Integer], Type::Integer);

        let integer_to_tuple =
            registry.intern(
                &[Type::Integer],
                Type::IntegerTuple,
            );

        assert_eq!(integer_to_integer, same);
        assert_ne!(integer_to_integer, integer_to_tuple);

        assert_eq!(
            registry.get(integer_to_integer),
            Some(&FunctionType {
                inputs: vec![Type::Integer],
                output: Type::Integer,
            })
        );
    }

    #[test]
    fn function_type_can_be_built_from_a_function() {
        let input = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let x = input.connection();

        let two = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("add(Integer, Integer) must exist");

        let body = Connection::new(
            add,
            vec![Box::new(two), Box::new(x)],
        )
        .expect("add(TWO, X) must be valid");

        let function = Function::new(
            vec![input],
            body,
        );

        let mut registry = FunctionTypeRegistry::new();

        let ty = registry.type_of(
            &[function.input_type()],
            function.body_type(),
        );

        assert_eq!(
            registry.get(match ty {
                Type::Function(id) => id,
                other => panic!("expected function type, got {:?}", other),
            }),
            Some(&FunctionType {
                inputs: vec![Type::Integer],
                output: Type::Integer,
            })
        );
    }

    #[test]
    fn function_value_has_function_type() {
        let input = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let x = input.connection();

        let two = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("add(Integer, Integer) must exist");

        let body = Connection::new(
            add,
            vec![Box::new(two), Box::new(x)],
        )
        .expect("add(TWO, X) must be valid");

        let function = Function::new(
            vec![input],
            body,
        );

        let value = Value::Function(Box::new(function));

        let mut registry = FunctionTypeRegistry::new();

        let ty = value.output_type_with(&mut registry);

        let function_type_id = match ty {
            Type::Function(id) => id,
            other => panic!("expected function type, got {:?}", other),
        };

        assert_eq!(
            registry.get(function_type_id),
            Some(&FunctionType {
                inputs: vec![Type::Integer],
                output: Type::Integer,
            })
        );
    }

    #[test]
    fn lbind_type_removes_first_argument() {
        let mut registry = FunctionTypeRegistry::new();

        let function_type = registry.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let result = registry
            .lbind_type(
                function_type,
                Type::Integer,
            )
            .expect("lbind type must succeed");

        let result_id = match result {
            Type::Function(id) => id,
            _ => panic!("result must be a function type"),
        };

        let result_type = registry
            .get(result_id)
            .expect("result must be a function type");

        assert_eq!(
            result_type,
            &FunctionType {
                inputs: vec![Type::Integer],
                output: Type::Integer,
            }
        );
    }

        #[test]
        fn lbind_type_rejects_wrong_fixed_type() {
            let mut registry = FunctionTypeRegistry::new();

            let function_type = registry.type_of(
                &[Type::Integer, Type::Integer],
                Type::Integer,
            );

            let result = registry.lbind_type(
                function_type,
                Type::Boolean,
            );

            assert!(result.is_err());
        }

    #[test]
    fn apply_type_returns_function_output() {
        let mut registry = FunctionTypeRegistry::new();

        let function_type = registry.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let result = registry
            .apply_type(
                function_type,
                &[Type::Integer, Type::Integer],
            )
            .expect("apply type must succeed");

        assert_eq!(result, Type::Integer);
    }

    #[test]
    fn apply_type_rejects_wrong_argument_type() {
        let mut registry = FunctionTypeRegistry::new();

        let function_type = registry.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let result = registry.apply_type(
            function_type,
            &[Type::Integer, Type::Boolean],
        );

        assert!(result.is_err());
    }

    #[test]
    fn lbind_then_apply_type_checks() {
        let mut registry = FunctionTypeRegistry::new();

        let function_type = registry.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let bound_type = registry
            .lbind_type(
                function_type,
                Type::Integer,
            )
            .expect("lbind type must succeed");

        let result = registry
            .apply_type(
                bound_type,
                &[Type::Integer],
            )
            .expect("apply type must succeed");

        assert_eq!(result, Type::Integer);
    }

    #[test]
    fn function_value_can_be_converted_back_to_function() {
        let input = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let connection = input.connection();

        let function = Function::new(
            vec![input],
            connection,
        );

        let value = function.clone().into_value();

        let recovered = Function::from_value(&value)
            .expect("function value must convert back");

        assert_eq!(
            recovered.inputs.len(),
            1,
        );

        assert_eq!(
            recovered.inputs[0].ty,
            Type::Integer,
        );
    }

    #[test]
    fn function_value_can_be_applied_to_value() {
        let input = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let two = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input.connection()),
                Box::new(two),
            ],
        )
        .expect("add connection must be valid");

        let function = Function::new(
            vec![input],
            body,
        );

        let function_value = Value::Function(
            Box::new(function)
        );

        let function = match function_value {
            Value::Function(function) => function,
            _ => unreachable!(),
        };

        let result = function
            .apply_values(vec![
                Value::Integer(3),
            ])
            .expect("function application must succeed");

        assert_eq!(
            result.output_type(),
            Type::Integer,
        );
    }

    #[test]
    fn function_value_type_can_be_registered() {
        let mut registry = FunctionTypeRegistry::new();

        let input = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let body = input.connection();

        let function = Function::new(
            vec![input],
            body,
        );

        let value = Value::Function(
            Box::new(function)
        );

        let ty = value.output_type_with(
            &mut registry,
        );

        let function_type_id = match ty {
            Type::Function(id) => id,
            _ => panic!("value type must be a function type"),
        };

        let function_type = registry
            .get(function_type_id)
            .expect("value type must be a function type");

        assert_eq!(
            function_type.inputs,
            vec![Type::Integer],
        );

        assert_eq!(
            function_type.output,
            Type::Integer,
        );
    }

    #[test]
    fn lbind_type_depends_on_function_type() {
        let mut registry = FunctionTypeRegistry::new();

        let integer_function = registry.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let boolean_function = registry.type_of(
            &[Type::Boolean, Type::Integer],
            Type::Integer,
        );

        let bound_integer = registry
            .lbind_type(
                integer_function,
                Type::Integer,
            )
            .expect("integer lbind must succeed");

        let bound_boolean = registry
            .lbind_type(
                boolean_function,
                Type::Boolean,
            )
            .expect("boolean lbind must succeed");

        assert_eq!(
            bound_integer,
            bound_boolean,
        );

        let bound_integer_id = match bound_integer {
            Type::Function(id) => id,
            _ => panic!("bound integer must be a function"),
        };

        assert_eq!(
            registry.get(bound_integer_id)
                .expect("bound integer must be a function")
                .inputs,
            vec![Type::Integer],
        );

        let bound_boolean_id = match bound_boolean {
            Type::Function(id) => id,
            _ => panic!("bound boolean must be a function"),
        };

        assert_eq!(
            registry.get(bound_boolean_id)
                .expect("bound boolean must be a function")
                .inputs,
            vec![Type::Integer],
        );
    }

    #[test]
    fn lbind_value_returns_function() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let result = lbind_value(
            &function,
            &Value::Integer(2),
        )
        .expect("lbind must succeed");

        match result {
            Value::Function(function) => {
                assert_eq!(function.inputs.len(), 1);
                assert_eq!(
                    function.inputs[0].ty,
                    Type::Integer,
                );
            }
            _ => panic!("lbind must return a Function"),
        }
    }

    #[test]
    fn lbind_value_result_can_be_applied() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let bound = lbind_value(
            &function,
            &Value::Integer(2),
        )
        .expect("lbind must succeed");

        let bound_function = match bound {
            Value::Function(function) => function,
            _ => panic!("lbind must return a Function"),
        };

        let result = bound_function
            .apply_values(vec![
                Value::Integer(3),
            ])
            .expect("application must succeed");

        match result {
            Value::Integer(value) => assert_eq!(value, 5),
            _ => panic!("expected Integer"),
        }
    }

    #[test]
    fn lbind_value_rejects_non_function() {
        let result = lbind_value(
            &Value::Integer(1),
            &Value::Integer(2),
        );

        assert!(result.is_err());
    }

    #[test]
    fn lbind_value_rejects_wrong_fixed_type() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let result = lbind_value(
            &function,
            &Value::Grid(vec![vec![1]]),
        );

        let error = result.expect_err(
            "lbind must reject the wrong fixed type",
        );

        assert!(
            error.contains("lbind type mismatch"),
            "unexpected error: {error}",
        );
    }

    #[test]
    fn lbind_value_works_for_three_arguments() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let input_z = InputSpec {
            name: "Z".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let xy = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("first add must be valid");

        let body = Connection::new(
            add,
            vec![
                Box::new(xy),
                Box::new(input_z.connection()),
            ],
        )
        .expect("second add must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y, input_z],
                body,
            )
        ));

        let bound = lbind_value(
            &function,
            &Value::Integer(1),
        )
        .expect("lbind must succeed");

        let bound_function = match bound {
            Value::Function(function) => function,
            _ => panic!("lbind must return a Function"),
        };

        let result = bound_function
            .apply_values(vec![
                Value::Integer(2),
                Value::Integer(3),
            ])
            .expect("application must succeed");

        match result {
            Value::Integer(value) => assert_eq!(value, 6),
            _ => panic!("expected Integer"),
        }
    }

    #[test]
    fn apply_value_applies_function_to_value() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let bound = lbind_value(
            &function,
            &Value::Integer(2),
        )
        .expect("lbind must succeed");

        let result = apply_value(
            &bound,
            &Value::Integer(3),
        )
        .expect("apply must succeed");

        match result {
            Value::Integer(value) => assert_eq!(value, 5),
            _ => panic!("expected Integer"),
        }
    }

    #[test]
    fn apply_value_rejects_non_function() {
        let result = apply_value(
            &Value::Integer(1),
            &Value::Integer(2),
        );

        assert!(result.is_err());
    }

    #[test]
    fn apply_value_rejects_wrong_argument_type() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let bound = lbind_value(
            &function,
            &Value::Integer(2),
        )
        .expect("lbind must succeed");

        let result = apply_value(
            &bound,
            &Value::Grid(vec![vec![1]]),
        );

        let error = result.expect_err(
            "apply must reject the wrong argument type",
        );

        assert!(
            error.contains("apply type mismatch"),
            "unexpected error: {error}",
        );
    }

    #[test]
    fn apply_value_requires_unary_function() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let result = apply_value(
            &function,
            &Value::Integer(2),
        );

        assert!(result.is_err());
    }

    #[test]
    fn apply_values_applies_multi_argument_function() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let result = apply_values(
            &function,
            vec![
                Value::Integer(2),
                Value::Integer(3),
            ],
        )
        .expect("apply must succeed");

        match result {
            Value::Integer(value) => assert_eq!(value, 5),
            _ => panic!("expected Integer"),
        }
    }

    #[test]
    fn apply_values_rejects_wrong_argument_count() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let result = apply_values(
            &function,
            vec![
                Value::Integer(2),
            ],
        );

        let error = result.expect_err(
            "apply must reject the wrong argument count",
        );

        assert!(
            error.contains("wrong number of arguments"),
            "unexpected error: {error}",
        );
    }

    #[test]
    fn lbind_value_supports_unary_function() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x.clone()],
                input_x.connection(),
            )
        ));

        let result = lbind_value(
            &function,
            &Value::Integer(1),
        )
        .expect("lbind must succeed");

        match result {
            Value::Function(function) => {
                assert!(function.inputs.is_empty());
            }
            _ => panic!("lbind must return a Function"),
        }
    }

    #[test]
    fn lbind_type_supports_zero_argument_function() {
        let mut registry = FunctionTypeRegistry::new();

        let function_type = registry.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        let result = registry.lbind_type(
            function_type,
            Type::Integer,
        );

        assert!(
            result.is_ok(),
            "lbind type should support a zero-argument function",
        );
    }

    #[test]
    fn apply_values_executes_zero_argument_function() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x.clone()],
                input_x.connection(),
            )
        ));

        let bound = lbind_value(
            &function,
            &Value::Integer(1),
        )
        .expect("lbind must succeed");

        let result = apply_values(
            &bound,
            vec![],
        )
        .expect("zero-argument application must succeed");

        match result {
            Value::Integer(value) => assert_eq!(value, 1),
            _ => panic!("expected Integer"),
        }
    }

    #[test]
    fn lbind_value_has_expected_function_type() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let bound = lbind_value(
            &function,
            &Value::Integer(2),
        )
        .expect("lbind must succeed");

        let mut function_types = FunctionTypeRegistry::new();

        let bound_type = bound.output_type_with(
            &mut function_types,
        );

        let expected = function_types.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        assert_eq!(bound_type, expected);
    }

    #[test]
    fn apply_values_result_has_expected_type() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x.clone()],
                input_x.connection(),
            )
        ));

        let result = apply_values(
            &function,
            vec![
                Value::Integer(42),
            ],
        )
        .expect("application must succeed");

        let mut function_types = FunctionTypeRegistry::new();

        let result_type = result.output_type_with(
            &mut function_types,
        );

        assert_eq!(
            result_type,
            Type::Integer,
        );
    }

    #[test]
    fn lbind_then_apply_values_composes_at_value_level() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let input_z = InputSpec {
            name: "Z".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let xy = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("first add must be valid");

        let body = Connection::new(
            add,
            vec![
                Box::new(xy),
                Box::new(input_z.connection()),
            ],
        )
        .expect("second add must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y, input_z],
                body,
            )
        ));

        let bound = lbind_value(
            &function,
            &Value::Integer(10),
        )
        .expect("lbind must succeed");

        let result = apply_values(
            &bound,
            vec![
                Value::Integer(20),
                Value::Integer(30),
            ],
        )
        .expect("application must succeed");

        match result {
            Value::Integer(value) => assert_eq!(value, 60),
            _ => panic!("expected Integer"),
        }
    }

    #[test]
    fn apply_values_rejects_wrong_second_argument_type() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let result = apply_values(
            &function,
            vec![
                Value::Integer(2),
                Value::Grid(vec![vec![1]]),
            ],
        );

        let error = result.expect_err(
            "apply must reject the wrong second argument type",
        );

        assert!(
            error.contains("apply type mismatch"),
            "unexpected error: {error}",
        );
    }

    #[test]
    fn apply_values_rejects_too_many_arguments() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let result = apply_values(
            &function,
            vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ],
        );

        let error = result.expect_err(
            "apply must reject too many arguments",
        );

        assert!(
            error.contains("wrong number of arguments"),
            "unexpected error: {error}",
        );
    }

    #[test]
    fn apply_value_types_match_runtime_application() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let mut function_types = FunctionTypeRegistry::new();

        let function_type = function.output_type_with(
            &mut function_types,
        );

        let result_type = function_types
            .apply_type(
                function_type,
                &[Type::Integer, Type::Integer],
            )
            .expect("apply type must succeed");

        assert_eq!(
            result_type,
            Type::Integer,
        );
    }

    #[test]
    fn lbind_value_type_matches_runtime_binding() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y],
                body,
            )
        ));

        let fixed = Value::Integer(10);

        let mut function_types = FunctionTypeRegistry::new();

        let function_type = function.output_type_with(
            &mut function_types,
        );

        let bound_type = function_types
            .lbind_type(
                function_type,
                Type::Integer,
            )
            .expect("lbind type must succeed");

        let bound = lbind_value(
            &function,
            &fixed,
        )
        .expect("lbind must succeed");

        let runtime_bound_type =
            bound.output_type_with(&mut function_types);

        assert_eq!(
            bound_type,
            runtime_bound_type,
        );
    }

    #[test]
    fn lbind_value_can_be_applied_successively() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let input_z = InputSpec {
            name: "Z".to_string(),
            ty: Type::Integer,
        };

        let add = find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("integer add must exist");

        let xy = Connection::new(
            add,
            vec![
                Box::new(input_x.connection()),
                Box::new(input_y.connection()),
            ],
        )
        .expect("inner add connection must be valid");

        let xyz = Connection::new(
            add,
            vec![
                Box::new(xy),
                Box::new(input_z.connection()),
            ],
        )
        .expect("outer add connection must be valid");

        let function = Value::Function(Box::new(
            Function::new(
                vec![input_x, input_y, input_z],
                xyz,
            ),
        ));

        let bound_x = lbind_value(
            &function,
            &Value::Integer(10),
        )
        .expect("first lbind must succeed");

        let bound_y = lbind_value(
            &bound_x,
            &Value::Integer(20),
        )
        .expect("second lbind must succeed");

        let result = apply_value(
            &bound_y,
            &Value::Integer(30),
        )
        .expect("final apply must succeed");

        match result {
            Value::Integer(value) => {
                assert_eq!(value, 60);
            }
            other => {
                panic!("expected Integer, got {:?}", other);
            }
        }
    }

    #[test]
    fn dynamic_primitive_names_are_stable() {
        assert_eq!(
            DynamicPrimitive::Lbind.name(),
            "lbind",
        );

        assert_eq!(
            DynamicPrimitive::Apply.name(),
            "apply",
        );
    }

    #[test]
    fn all_types_returns_interned_function_types() {
        let mut registry = FunctionTypeRegistry::new();

        let id = registry.intern(
            &[Type::Integer],
            Type::Integer,
        );

        let types = registry
            .all_types()
            .collect::<Vec<_>>();

        assert_eq!(types.len(), 1);
        assert_eq!(types[0].0, id);
        assert_eq!(
            types[0].1.inputs,
            vec![Type::Integer],
        );
        assert_eq!(
            types[0].1.output,
            Type::Integer,
        );
    }

    #[test]
    fn function_type_accessors_return_signature() {
        let mut registry = FunctionTypeRegistry::new();

        let id = registry.intern(
            &[Type::Integer, Type::Grid],
            Type::Object,
        );

        assert_eq!(
            registry.inputs(id),
            Some(&[
                Type::Integer,
                Type::Grid,
            ][..]),
        );

        assert_eq!(
            registry.output(id),
            Some(Type::Object),
        );
    }

    #[test]
    fn primitive_macro_generates_entry() {
        assert_eq!(crate::primitives::add_entry.name, "add");
        assert_eq!(
            crate::primitives::add_entry.inputs,
            &[Type::Integer, Type::Integer]
        );
        assert_eq!(
            crate::primitives::add_entry.output,
            Type::Integer
        );
    }

    #[test]
    fn rbind_type_removes_last_argument() {
        let mut registry = FunctionTypeRegistry::new();

        let function_type = registry.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let result = registry
            .rbind_type(
                function_type,
                Type::Integer,
            )
            .expect("rbind type must succeed");

        let result_id = match result {
            Type::Function(id) => id,
            _ => panic!("result must be a function type"),
        };

        let result_type = registry
            .get(result_id)
            .expect("result must be a function type");

        assert_eq!(
            result_type,
            &FunctionType {
                inputs: vec![Type::Integer],
                output: Type::Integer,
            }
        );
    }

    #[test]
    fn rbind_type_rejects_wrong_fixed_type() {
        let mut registry = FunctionTypeRegistry::new();

        let function_type = registry.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let result = registry.rbind_type(
            function_type,
            Type::Boolean,
        );

        assert!(result.is_err());
    }

    #[test]
    fn primitives_by_name_finds_all_add_overloads() {
        let primitives = primitives_by_name("add");

        assert_eq!(primitives.len(), 5);

        assert_eq!(
            primitives
                .iter()
                .filter(|entry| {
                    entry.output == Type::Callable
                })
                .count(),
            1
        );

        assert!(primitives.iter().all(|primitive| {
            primitive.name == "add"
        }));
    }

    #[test]
    fn apply_primitive_family_dispatches_add_integer() {
        let result = apply_primitive_family(
            "add",
            &[
                Value::Integer(2),
                Value::Integer(3),
            ],
        )
        .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }

    #[test]
    fn apply_primitive_family_rejects_wrong_arguments() {
        let result = apply_primitive_family(
            "add",
            &[
                Value::Boolean(true),
                Value::Integer(3),
            ],
        );

        assert!(result.is_err());
    }

    #[test]
    fn lbind_primitive_family_should_bind_left_argument() {
        let connection =
            crate::connection::Connection::callable_primitive(
                &crate::primitives::add_entry,
            );

        let environment =
            crate::environment::InputEnvironment::new();

        let function = connection
            .output_with_inputs(&environment)
            .unwrap();

        let bound = lbind_value(
            &function,
            &Value::Integer(2),
        )
        .unwrap();

        let result = apply_values(
            &bound,
            vec![Value::Integer(3)],
        )
        .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }

    #[test]
    fn rbind_primitive_family_should_bind_right_argument() {
        let connection =
            crate::connection::Connection::callable_primitive(
                &crate::primitives::add_entry,
            );

        let environment =
            crate::environment::InputEnvironment::new();

        let function = connection
            .output_with_inputs(&environment)
            .unwrap();

        let bound = rbind_value(
            &function,
            &Value::Integer(3),
        )
        .unwrap();

        let result = apply_values(
            &bound,
            vec![Value::Integer(2)],
        )
        .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }

    #[test]
    fn bound_primitive_family_can_be_bound_again() {
        let connection =
            crate::connection::Connection::callable_primitive(
                &crate::primitives::add_entry,
            );

        let environment =
            crate::environment::InputEnvironment::new();

        let function = connection
            .output_with_inputs(&environment)
            .unwrap();

        let left_bound = lbind_value(
            &function,
            &Value::Integer(2),
        )
        .unwrap();

        let right_bound = rbind_value(
            &left_bound,
            &Value::Integer(3),
        )
        .unwrap();

        let result = apply_values(
            &right_bound,
            vec![],
        )
        .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }

    #[test]
    fn bound_primitive_family_can_be_bound_in_reverse_order() {
        let connection =
            crate::connection::Connection::callable_primitive(
                &crate::primitives::add_entry,
            );

        let environment =
            crate::environment::InputEnvironment::new();

        let function = connection
            .output_with_inputs(&environment)
            .unwrap();

        let right_bound = rbind_value(
            &function,
            &Value::Integer(3),
        )
        .unwrap();

        let left_bound = lbind_value(
            &right_bound,
            &Value::Integer(2),
        )
        .unwrap();

        let result = apply_values(
            &left_bound,
            vec![],
        )
        .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }

    #[test]
    fn apply_primitive_family_applies_arguments() {
        let connection =
            crate::connection::Connection::callable_primitive(
                &crate::primitives::add_entry,
            );

        let environment =
            crate::environment::InputEnvironment::new();

        let function = connection
            .output_with_inputs(&environment)
            .unwrap();

        let result = apply_values(
            &function,
            vec![
                Value::Integer(2),
                Value::Integer(3),
            ],
        )
        .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }

    #[test]
    fn apply_bound_primitive_family() {
        let connection =
            crate::connection::Connection::callable_primitive(
                &crate::primitives::add_entry,
            );

        let environment =
            crate::environment::InputEnvironment::new();

        let function = connection
            .output_with_inputs(&environment)
            .unwrap();

        let bound = lbind_value(
            &function,
            &Value::Integer(2),
        )
        .unwrap();

        let result = apply_values(
            &bound,
            vec![Value::Integer(3)],
        )
        .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }

    #[test]
    fn callable_can_apply_integer() {
        let registry =
            FunctionTypeRegistry::new();

        let result = registry.apply_type(
            Type::Callable,
            &[Type::Integer],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn callable_apply_integer_can_resolve_known_function_type() {
        let mut registry =
            FunctionTypeRegistry::new();

        let integer_to_integer =
            registry.type_of(
                &[Type::Integer],
                Type::Integer,
            );

        let result = registry.apply_type(
            integer_to_integer,
            &[Type::Integer],
        );

        assert_eq!(
            result.unwrap(),
            Type::Integer
        );
    }

    #[test]
    fn add_family_contains_integer_integer_signature() {
        let entries =
            crate::registry::primitives_by_name("add");

        assert!(entries.iter().any(|entry| {
            entry.inputs == &[Type::Integer, Type::Integer]
                && entry.output == Type::Integer
        }));
    }

    #[test]
    fn apply_callable_family_resolves_add() {
        let mut registry =
            FunctionTypeRegistry::new();

        let output = registry
            .apply_callable_family(
                "add",
                &[Type::Integer, Type::Integer],
            )
            .unwrap();

        assert_eq!(
            output,
            Type::Integer
        );
    }

    #[test]
    fn historical_apply_integer_tuple_to_integer_vector() {
        let function = Value::Function(Box::new(
            Function::primitive_family(
                crate::registry::primitives_by_name("add")
                    .into_iter()
                    .find(|primitive| {
                        primitive.inputs
                            == &[Type::Integer, Type::Integer]
                            && primitive.output
                                == Type::Integer
                    })
                    .expect("expected integer add"),
            ),
        ));

        let result = map_apply_value(
            &function,
            &Value::IntegerTuple((2, 3)),
        );

        assert!(result.is_err());
    }

    #[test]
    fn historical_apply_maps_identity_over_integer_tuple() {
        let function = Value::Function(Box::new(
            Function::primitive_family(
                &identity_callable_entry,
            ),
        ));

        let container =
            Value::IntegerTuple((2, 3));

        let result = map_apply_value(
            &function,
            &container,
        )
        .expect("expected historical apply");

        assert!(matches!(
            result,
            Value::IntegerVector(values)
                if values == vec![2, 3]
        ));
    }

    #[test]
    fn historical_apply_maps_identity_over_integer_vector() {
        let function = Value::Function(Box::new(
            Function::primitive_family(
                &identity_callable_entry,
            ),
        ));

        let container =
            Value::IntegerVector(vec![2, 3, 4]);

        let result = map_apply_value(
            &function,
            &container,
        )
        .expect("expected historical apply");

        assert!(matches!(
            result,
            Value::IntegerVector(values)
                if values == vec![2, 3, 4]
        ));
    }

    #[test]
    fn historical_apply_maps_hmirror_over_grid_vector() {
        let primitive = primitives_by_name("hmirror")
            .into_iter()
            .find(|primitive| {
                primitive.inputs == &[Type::Grid]
                    && primitive.output == Type::Grid
            })
            .expect("expected hmirror(Grid) primitive");

        let function = Value::Function(Box::new(
            Function::primitive_family(primitive),
        ));

        let grid1 = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        let grid2 = vec![
            vec![5, 6, 7],
            vec![8, 9, 10],
        ];

        let container = Value::GridVector(vec![
            grid1,
            grid2,
        ]);

        let result = map_apply_value(
            &function,
            &container,
        )
        .expect("expected historical apply");

        assert!(matches!(
            result,
            Value::GridVector(values)
                if values.len() == 2
        ));
    }

    #[test]
    fn historical_apply_maps_hmirror_over_object_vector() {
        let primitive = crate::registry::PRIMITIVES
            .iter()
            .find(|primitive| {
                primitive.name == "hmirror"
                    && primitive.inputs == &[Type::Object]
                    && primitive.output == Type::Object
            })
            .expect("expected hmirror(Object) primitive");

        let function =
            Value::Function(Box::new(Function::primitive_family(
                primitive,
            )));

        let object1: Object = BTreeSet::from([
            (1, (2, 3)),
            (2, (4, 5)),
        ]);

        let object2: Object = BTreeSet::from([
            (3, (1, 2)),
            (4, (5, 6)),
        ]);

        let container =
            Value::ObjectVector(vec![object1, object2]);

        let result =
            map_apply_value(&function, &container)
                .expect("historical apply should succeed");

        match result {
            Value::ObjectVector(values) => {
                assert_eq!(values.len(), 2);

                assert_eq!(
                    values[0],
                    BTreeSet::from([
                        (1, (4, 3)),
                        (2, (2, 5)),
                    ])
                );

                assert_eq!(
                    values[1],
                    BTreeSet::from([
                        (3, (5, 2)),
                        (4, (1, 6)),
                    ])
                );
            }

            other => {
                panic!(
                    "expected ObjectVector, got {:?}",
                    other
                );
            }
        }
    }

    #[test]
    fn historical_apply_maps_hmirror_over_objects() {
        let primitive = crate::registry::PRIMITIVES
            .iter()
            .find(|primitive| {
                primitive.name == "hmirror"
                    && primitive.inputs == &[Type::Object]
                    && primitive.output == Type::Object
            })
            .expect("expected hmirror(Object) primitive");

        let function =
            Value::Function(Box::new(Function::primitive_family(
                primitive,
            )));

        let object1: Object = BTreeSet::from([
            (1, (2, 3)),
            (2, (4, 5)),
        ]);

        let object2: Object = BTreeSet::from([
            (3, (1, 2)),
            (4, (5, 6)),
        ]);

        let container = Value::Objects(BTreeSet::from([
            object1,
            object2,
        ]));

        let result =
            map_apply_value(&function, &container)
                .expect("historical apply should succeed");

        match result {
            Value::Objects(values) => {
                assert_eq!(values.len(), 2);

                assert!(
                    values.contains(&BTreeSet::from([
                        (1, (4, 3)),
                        (2, (2, 5)),
                    ]))
                );

                assert!(
                    values.contains(&BTreeSet::from([
                        (3, (5, 2)),
                        (4, (1, 6)),
                    ]))
                );
            }

            other => {
                panic!(
                    "expected Objects, got {:?}",
                    other
                );
            }
        }
    }

    #[test]
    fn historical_apply_color_over_objects() {
        let primitive = crate::registry::PRIMITIVES
            .iter()
            .find(|primitive| {
                primitive.name == "color"
                    && primitive.inputs == &[Type::Object]
                    && primitive.output == Type::Integer
            })
            .expect("expected color(Object) primitive");

        let function =
            Value::Function(Box::new(Function::primitive_family(
                primitive,
            )));

        let object1: Object = BTreeSet::from([
            (3, (4, 5)),
            (7, (1, 2)),
        ]);

        let object2: Object = BTreeSet::from([
            (5, (2, 3)),
            (8, (4, 6)),
        ]);

        let container = Value::Objects(BTreeSet::from([
            object1,
            object2,
        ]));

        let result =
            map_apply_value(&function, &container)
                .expect("historical apply should succeed");

        match result {
            Value::IntegerVector(values) => {
                assert_eq!(values, vec![3, 5]);
            }

            other => {
                panic!(
                    "expected IntegerVector, got {:?}",
                    other
                );
            }
        }
    }
}
