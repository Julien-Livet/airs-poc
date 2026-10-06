use linkme::distributed_slice;
use std::collections::HashMap;

use crate::connection::Connection;
use crate::types::{Grid, Indices, Integer, IntegerTuple, Object, Boolean};
use crate::function::Function;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicPrimitive {
    Lbind,
    Apply,
    Rbind,
}

impl DynamicPrimitive {
    pub fn name(self) -> &'static str {
        match self {
            DynamicPrimitive::Lbind => "lbind",
            DynamicPrimitive::Rbind => "rbind",
            DynamicPrimitive::Apply => "apply",
        }
    }
}

pub fn dynamic_primitives() -> &'static [DynamicPrimitive] {
    &[
        DynamicPrimitive::Lbind,
        DynamicPrimitive::Apply,
    ]
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

    pub fn can_lbind(
        &self,
        function_type: Type,
        fixed_type: Type,
    ) -> bool {
        let id = match function_type {
            Type::Function(id) => id,
            _ => return false,
        };

        let function = match self.get(id) {
            Some(function) => function,
            None => return false,
        };

        !function.inputs.is_empty()
            && function.inputs[0] == fixed_type
    }

    pub fn can_apply(
        &self,
        function_type: Type,
        argument_types: &[Type],
    ) -> bool {
        self.apply_type(
            function_type,
            argument_types,
        ).is_ok()
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
}

impl FunctionTypeRegistry {
    pub fn function_type(
        &self,
        ty: Type,
    ) -> Option<&FunctionType> {
        match ty {
            Type::Function(id) => self.get(id),
            _ => None,
        }
    }
}

impl FunctionTypeRegistry {
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
}

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

impl HasType for Boolean {
    const TYPE: Type = Type::Boolean;
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
    Function(FunctionTypeId),
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
}

#[derive(Debug)]
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{primitives};
    use crate::connection::Connection;
    use crate::signature::InputSpec;
    use crate::function::Function;

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
    fn registry_contains_expected_primitives() {
        assert_eq!(PRIMITIVES.len(), 36);

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
    fn function_type_can_be_resolved_from_type() {
        let mut registry = FunctionTypeRegistry::new();

        let ty = registry.type_of(
            &[Type::Integer],
            Type::Integer,
        );

        let function_type = registry
            .function_type(ty)
            .expect("function type must resolve");

        assert_eq!(
            function_type,
            &FunctionType {
                inputs: vec![Type::Integer],
                output: Type::Integer,
            }
        );

        assert_eq!(
            registry.function_type(Type::Integer),
            None
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

        let result_type = registry
            .function_type(result)
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

        let function_type = registry
            .function_type(ty)
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

        assert_eq!(
            registry.function_type(bound_integer)
                .expect("bound integer must be a function")
                .inputs,
            vec![Type::Integer],
        );

        assert_eq!(
            registry.function_type(bound_boolean)
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
    fn can_lbind_checks_function_and_fixed_type() {
        let mut registry = FunctionTypeRegistry::new();

        let function_type = registry.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        assert!(
            registry.can_lbind(
                function_type,
                Type::Integer,
            )
        );

        assert!(
            !registry.can_lbind(
                function_type,
                Type::Grid,
            )
        );

        assert!(
            !registry.can_lbind(
                Type::Integer,
                Type::Integer,
            )
        );
    }

    #[test]
    fn can_apply_checks_function_and_argument_types() {
        let mut registry = FunctionTypeRegistry::new();

        let function_type = registry.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        assert!(
            registry.can_apply(
                function_type,
                &[Type::Integer, Type::Integer],
            )
        );

        assert!(
            !registry.can_apply(
                function_type,
                &[Type::Integer],
            )
        );

        assert!(
            !registry.can_apply(
                function_type,
                &[Type::Integer, Type::Grid],
            )
        );

        assert!(
            !registry.can_apply(
                Type::Integer,
                &[Type::Integer],
            )
        );
    }

    #[test]
    fn dynamic_primitives_contains_lbind_and_apply() {
        let primitives = dynamic_primitives();

        assert!(
            primitives.contains(&DynamicPrimitive::Lbind)
        );

        assert!(
            primitives.contains(&DynamicPrimitive::Apply)
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

        let result_type = registry
            .function_type(result)
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
}
