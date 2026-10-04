use linkme::distributed_slice;

use crate::primitives;
use crate::types::{Grid, Integer};

pub trait HasType {
    const TYPE: Type;
}

impl HasType for Integer {
    const TYPE: Type = Type::Integer;
}

impl HasType for Grid {
    const TYPE: Type = Type::Grid;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Integer,
    Grid,
}


#[derive(Debug, Clone)]
pub enum Value {
    Integer(Integer),
    Grid(Grid),
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

register_binary_primitive!(
    ADD,
    add_dyn_generated,
    "add",
    primitives::add
);

register_unary_primitive!(
    HMIRROR,
    hmirror_dyn_generated,
    "hmirror",
    primitives::hmirror
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

#[cfg(test)]
mod tests
{
    use super::{
        PrimitiveEntry,
        Type,
        unary_descriptor,
        binary_descriptor,
        call_unary,
        Value,
        PRIMITIVES,
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
        assert_eq!(PRIMITIVES.len(), 4);

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
        let primitive = PRIMITIVES
            .iter()
            .find(|primitive| primitive.name == "hmirror")
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
}
