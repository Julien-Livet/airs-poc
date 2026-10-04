use linkme::distributed_slice;

use crate::primitives;
use crate::types::{Grid, Indices, Integer, IntegerTuple, Object};

pub trait HasType {
    const TYPE: Type;
}

impl HasType for Integer {
    const TYPE: Type = Type::Integer;
}

impl HasType for Grid {
    const TYPE: Type = Type::Grid;
}

impl HasType for IntegerTuple {
    const TYPE: Type = Type::IntegerTuple;
}

impl HasType for Indices {
    const TYPE: Type = Type::Indices;
}

impl HasType for Object {
    const TYPE: Type = Type::Object;
}

macro_rules! register_primitive {
    (
        $static_name:ident,
        $entry:expr
    ) => {
        #[distributed_slice(PRIMITIVES)]
        static $static_name: PrimitiveEntry = $entry;
    };
}

macro_rules! register_unary_primitive {
    (
        $static_name:ident,
        $wrapper_name:ident,
        $name:expr,
        $function:path
    ) => {
        fn $wrapper_name(
            args: &[Value],
        ) -> Result<Value, String> {
            call_unary(args, $function)
        }

        register_primitive!(
            $static_name,
            PrimitiveEntry::from_descriptor(
                unary_descriptor($name, $function),
                $wrapper_name,
            )
        );
    };
}

macro_rules! register_binary_primitive {
    (
        $static_name:ident,
        $wrapper_name:ident,
        $name:expr,
        $function:path
    ) => {
        fn $wrapper_name(
            args: &[Value],
        ) -> Result<Value, String> {
            call_binary(args, $function)
        }

        register_primitive!(
            $static_name,
            PrimitiveEntry::from_descriptor(
                binary_descriptor($name, $function),
                $wrapper_name,
            )
        );
    };
}

macro_rules! register_ternary_primitive {
    (
        $static_name:ident,
        $wrapper_name:ident,
        $name:expr,
        $function:path
    ) => {
        fn $wrapper_name(
            args: &[Value],
        ) -> Result<Value, String> {
            call_ternary(args, $function)
        }

        register_primitive!(
            $static_name,
            PrimitiveEntry::from_descriptor(
                ternary_descriptor($name, $function),
                $wrapper_name,
            )
        );
    };
}

pub struct PrimitiveDescriptor {
    pub name: &'static str,
    pub inputs: &'static [Type],
    pub output: Type,
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

pub const fn unary_descriptor<A, R>(
    name: &'static str,
    _function: fn(A) -> R,
) -> PrimitiveDescriptor
where
    A: HasType,
    R: HasType,
{
    PrimitiveDescriptor {
        name,
        inputs: &[A::TYPE],
        output: R::TYPE,
    }
}

pub const fn binary_descriptor<A, B, R>(
    name: &'static str,
    _function: fn(A, B) -> R,
) -> PrimitiveDescriptor
where
    A: HasType,
    B: HasType,
    R: HasType,
{
    PrimitiveDescriptor {
        name,
        inputs: &[A::TYPE, B::TYPE],
        output: R::TYPE,
    }
}

pub const fn ternary_descriptor<A, B, C, R>(
    name: &'static str,
    _function: fn(A, B, C) -> R,
) -> PrimitiveDescriptor
where
    A: HasType,
    B: HasType,
    C: HasType,
    R: HasType,
{
    PrimitiveDescriptor {
        name,
        inputs: &[A::TYPE, B::TYPE, C::TYPE],
        output: R::TYPE,
    }
}

pub fn call_unary<A, R>(
    args: &[Value],
    function: fn(A) -> R,
) -> Result<Value, String>
where
    A: FromValue,
    R: IntoValue,
{
    if args.len() != 1 {
        return Err(format!(
            "expected 1 argument, got {}",
            args.len()
        ));
    }

    let a = A::from_value(&args[0])?;

    Ok(function(a).into_value())
}

pub fn call_binary<A, B, R>(
    args: &[Value],
    function: fn(A, B) -> R,
) -> Result<Value, String>
where
    A: FromValue,
    B: FromValue,
    R: IntoValue,
{
    if args.len() != 2 {
        return Err(format!(
            "expected 2 arguments, got {}",
            args.len()
        ));
    }

    let a = A::from_value(&args[0])?;
    let b = B::from_value(&args[1])?;

    Ok(function(a, b).into_value())
}

pub fn call_ternary<A, B, C, R>(
    args: &[Value],
    function: fn(A, B, C) -> R,
) -> Result<Value, String>
where
    A: FromValue,
    B: FromValue,
    C: FromValue,
    R: IntoValue,
{
    if args.len() != 3 {
        return Err(format!(
            "expected 3 arguments, got {}",
            args.len()
        ));
    }

    let a = A::from_value(&args[0])?;
    let b = B::from_value(&args[1])?;
    let c = C::from_value(&args[2])?;

    Ok(function(a, b, c).into_value())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Integer,
    Grid,
    IntegerTuple,
    Indices,
    Object,
}


#[derive(Debug, Clone)]
pub enum Value {
    Integer(Integer),
    Grid(Grid),
    IntegerTuple(IntegerTuple),
    Indices(Indices),
    Object(Object),
}


pub struct PrimitiveEntry {
    pub name: &'static str,
    pub inputs: &'static [Type],
    pub output: Type,
    pub apply: fn(&[Value]) -> Result<Value, String>,
}

impl PrimitiveEntry {
    pub const fn new(
        name: &'static str,
        inputs: &'static [Type],
        output: Type,
        apply: fn(&[Value]) -> Result<Value, String>,
    ) -> Self {
        Self {
            name,
            inputs,
            output,
            apply,
        }
    }

    pub const fn from_descriptor(
        descriptor: PrimitiveDescriptor,
        apply: fn(&[Value]) -> Result<Value, String>,
    ) -> Self {
        Self::new(
            descriptor.name,
            descriptor.inputs,
            descriptor.output,
            apply,
        )
    }

    pub fn accepts(&self, inputs: &[Type]) -> bool {
        self.inputs == inputs
    }

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

register_binary_primitive!(
    ADD,
    add_dyn_generated,
    "add",
    primitives::add
);

register_binary_primitive!(
    ADD_TUPLE_TUPLE,
    add_tuple_tuple_dyn_generated,
    "add",
    primitives::add_tuple_tuple
);

register_binary_primitive!(
    ADD_INTEGER_TUPLE,
    add_integer_tuple_dyn_generated,
    "add",
    primitives::add_integer_tuple
);

register_binary_primitive!(
    ADD_TUPLE_INTEGER,
    add_tuple_integer_dyn_generated,
    "add",
    primitives::add_tuple_integer
);

register_unary_primitive!(
    HMIRROR,
    hmirror_dyn_generated,
    "hmirror",
    primitives::hmirror
);

register_unary_primitive!(
    HMIRROR_INDICES,
    hmirror_indices_dyn_generated,
    "hmirror",
    primitives::hmirror_indices
);

register_unary_primitive!(
    VMIRROR,
    vmirror_dyn_generated,
    "vmirror",
    primitives::vmirror
);

register_binary_primitive!(
    VCONCAT,
    vconcat_dyn_generated,
    "vconcat",
    primitives::vconcat
);

register_unary_primitive!(
    ULCORNER,
    ulcorner_dyn_generated,
    "ulcorner",
    primitives::ulcorner
);

register_unary_primitive!(
    URCORNER,
    urcorner_dyn_generated,
    "urcorner",
    primitives::urcorner
);

register_unary_primitive!(
    LLCORNER,
    llcorner_dyn_generated,
    "llcorner",
    primitives::llcorner
);

register_unary_primitive!(
    LRCORNER,
    lrcorner_dyn_generated,
    "lrcorner",
    primitives::lrcorner
);

register_ternary_primitive!(
    CROP,
    crop_dyn_generated,
    "crop",
    primitives::crop
);

#[cfg(test)]
mod tests
{
    use super::{
        PrimitiveEntry,
        Type,
        unary_descriptor,
        binary_descriptor,
        ternary_descriptor,
        call_unary,
        call_ternary,
        Value,
        PRIMITIVES,
        find_compatible,
        find_by_name_and_inputs,
    };
    use crate::{primitives, registry::hmirror_dyn_generated};

    #[test]
    fn unary_descriptor_infers_types() {
        let descriptor = unary_descriptor(
            "hmirror",
            primitives::hmirror,
        );

        assert_eq!(descriptor.name, "hmirror");
        assert_eq!(descriptor.inputs, &[Type::Grid]);
        assert_eq!(descriptor.output, Type::Grid);
    }

    #[test]
    fn binary_descriptor_infers_types() {
        let descriptor = binary_descriptor(
            "vconcat",
            primitives::vconcat,
        );

        assert_eq!(descriptor.name, "vconcat");
        assert_eq!(
            descriptor.inputs,
            &[Type::Grid, Type::Grid],
        );
        assert_eq!(
            descriptor.output,
            Type::Grid,
        );
    }

    #[test]
    fn unary_call_adapter_can_be_generated_from_function() {
        let result = call_unary(
            &[Value::Grid(vec![
                vec![1, 2],
                vec![3, 4],
            ])],
            primitives::hmirror,
        )
        .unwrap();

        assert_eq!(
            result.output_type(),
            Type::Grid,
        );
    }

    #[test]
    fn primitive_entry_can_be_built_from_descriptor() {
        let descriptor = unary_descriptor(
            "hmirror",
            primitives::hmirror,
        );

        let entry = PrimitiveEntry::from_descriptor(
            descriptor,
            hmirror_dyn_generated,
        );

        assert_eq!(entry.name, "hmirror");
        assert_eq!(entry.inputs, &[Type::Grid]);
        assert_eq!(entry.output, Type::Grid);
    }

    #[test]
    fn registry_contains_expected_primitives() {
        assert_eq!(PRIMITIVES.len(), 13);

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
        let primitive = PRIMITIVES
            .iter()
            .find(|primitive| primitive.name == "ulcorner")
            .expect("ulcorner should be registered");

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
    fn ternary_descriptor_infers_types() {
        let descriptor = ternary_descriptor(
            "crop",
            primitives::crop,
        );

        assert_eq!(descriptor.name, "crop");
        assert_eq!(
            descriptor.inputs,
            &[Type::Grid, Type::IntegerTuple, Type::IntegerTuple]
        );
        assert_eq!(descriptor.output, Type::Grid);
    }

    #[test]
    fn ternary_call_adapter_can_be_generated_from_function() {
        let grid = vec![
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 0],
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 0],
        ];

        let result = call_ternary(
            &[
                Value::Grid(grid),
                Value::IntegerTuple((1, 1)),
                Value::IntegerTuple((2, 3)),
            ],
            primitives::crop,
        )
        .expect("crop should apply successfully");

        assert!(matches!(
            result,
            Value::Grid(grid)
                if grid == vec![
                    vec![7, 8, 9],
                    vec![2, 3, 4],
                ]
        ));
    }

    #[test]
    fn crop_is_found_by_input_signature() {
        let matches = find_compatible(&[
            Type::Grid,
            Type::IntegerTuple,
            Type::IntegerTuple,
        ]);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "crop");
        assert_eq!(matches[0].output, Type::Grid);
    }

    #[test]
    fn registry_contains_grid_and_indices_hmirror() {
        let grid = find_compatible(&[Type::Grid]);
        assert!(
            grid.iter().any(|primitive| {
                primitive.name == "hmirror"
                    && primitive.output == Type::Grid
            })
        );

        let indices = find_compatible(&[Type::Indices]);
        assert!(
            indices.iter().any(|primitive| {
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
}
