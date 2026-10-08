use crate::registry::{PrimitiveEntry, Type, Value, FunctionTypeRegistry, DynamicPrimitive};
use crate::types::*;
use crate::function::Function;

use serde::{Deserialize, Serialize};

impl TryFrom<ConnectionJson> for Connection {
    type Error = String;

    fn try_from(value: ConnectionJson) -> Result<Self, Self::Error> {
        match value {
            ConnectionJson::Input { name, ty } => {
                Ok(Connection::Input {
                    name,
                    ty: ty.try_into()?,
                })
            }

            ConnectionJson::Constant { name, value } => {
                Ok(Connection::Constant {
                    name,
                    value: value.try_into()?,
                })
            }

            ConnectionJson::Primitive {
                primitive,
                inputs,
            } => {
                // Deserialize inputs first, so we know their actual types.
                let inputs = inputs
                    .into_iter()
                    .map(Connection::try_from)
                    .map(|input| input.map(Box::new))
                    .collect::<Result<Vec<_>, _>>()?;

                let input_types: Vec<Type> = inputs
                    .iter()
                    .map(|input| input.output_type())
                    .collect();

                let primitive_entry =
                    crate::registry::PRIMITIVES
                        .iter()
                        .find(|entry| {
                            entry.name == primitive
                                && entry.inputs == input_types.as_slice()
                        })
                        .ok_or_else(|| {
                            format!(
                                "unknown primitive '{}' with signature {:?}",
                                primitive,
                                input_types
                            )
                        })?;

                Connection::new(
                    primitive_entry,
                    inputs,
                )
            }

            ConnectionJson::CallablePrimitive { primitive } => {
                let primitive_entry =
                    crate::registry::PRIMITIVES
                        .iter()
                        .find(|entry| {
                            entry.name == primitive
                                && entry.output == Type::Callable
                        })
                        .ok_or_else(|| {
                            format!(
                                "unknown callable primitive '{}'",
                                primitive
                            )
                        })?;

                Ok(Connection::CallablePrimitive {
                    primitive: primitive_entry,
                })
            }

            ConnectionJson::Dynamic {
                primitive,
                inputs,
                output,
                family,
                fixed,
                left,
            } => {
                let inputs = inputs
                    .into_iter()
                    .map(Connection::try_from)
                    .map(|input| input.map(Box::new))
                    .collect::<Result<Vec<_>, _>>()?;

                let fixed = fixed
                    .map(|fixed| Connection::try_from(*fixed))
                    .transpose()?
                    .map(Box::new);

                Ok(Connection::Dynamic {
                    primitive: primitive.into(),
                    inputs,
                    output: output.try_into()?,
                    family: family
                        .map(|family| {
                            Box::leak(
                                family.into_boxed_str()
                            ) as &'static str
                        }),
                    fixed,
                    left,
                })
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ConnectionJson {
    Input {
        name: String,
        ty: TypeJson,
    },

    Constant {
        name: String,
        value: ValueJson,
    },

    CallablePrimitive {
        primitive: String,
    },

    Primitive {
        primitive: String,
        inputs: Vec<ConnectionJson>,
    },

    Dynamic {
        primitive: DynamicPrimitiveJson,
        inputs: Vec<ConnectionJson>,
        output: TypeJson,
        family: Option<String>,
        fixed: Option<Box<ConnectionJson>>,
        left: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TypeJson {
    Boolean,
    Integer,
    Grid,
    IntegerTuple,
    Indices,
    Object,
    IntegerVector,
    GridVector,
    TupleVector,
    Callable,
    ObjectVector,
    Objects,
}

impl TryFrom<Type> for TypeJson {
    type Error = String;

    fn try_from(value: Type) -> Result<Self, Self::Error> {
        match value {
            Type::Boolean => Ok(Self::Boolean),
            Type::Integer => Ok(Self::Integer),
            Type::Grid => Ok(Self::Grid),
            Type::IntegerTuple => Ok(Self::IntegerTuple),
            Type::Indices => Ok(Self::Indices),
            Type::Object => Ok(Self::Object),
            Type::IntegerVector => Ok(Self::IntegerVector),
            Type::GridVector => Ok(Self::GridVector),
            Type::TupleVector => Ok(Self::TupleVector),
            Type::Callable => Ok(Self::Callable),
            Type::ObjectVector => Ok(Self::ObjectVector),
            Type::Objects => Ok(Self::Objects),

            Type::Function(_) => Err(
                "Function types require signature serialization"
                    .to_string(),
            ),
        }
    }
}

impl TryFrom<TypeJson> for Type {
    type Error = String;

    fn try_from(value: TypeJson) -> Result<Self, Self::Error> {
        match value {
            TypeJson::Boolean => Ok(Self::Boolean),
            TypeJson::Integer => Ok(Self::Integer),
            TypeJson::Grid => Ok(Self::Grid),
            TypeJson::IntegerTuple => Ok(Self::IntegerTuple),
            TypeJson::Indices => Ok(Self::Indices),
            TypeJson::Object => Ok(Self::Object),
            TypeJson::IntegerVector => Ok(Self::IntegerVector),
            TypeJson::GridVector => Ok(Self::GridVector),
            TypeJson::TupleVector => Ok(Self::TupleVector),
            TypeJson::Callable => Ok(Self::Callable),
            TypeJson::ObjectVector => Ok(Self::ObjectVector),
            TypeJson::Objects => Ok(Self::Objects),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DynamicPrimitiveJson {
    Lbind,
    Apply,
    Rbind,
    HistoricalApply,
}

impl From<DynamicPrimitive> for DynamicPrimitiveJson {
    fn from(value: DynamicPrimitive) -> Self {
        match value {
            DynamicPrimitive::Lbind => Self::Lbind,
            DynamicPrimitive::Apply => Self::Apply,
            DynamicPrimitive::Rbind => Self::Rbind,
            DynamicPrimitive::HistoricalApply => {
                Self::HistoricalApply
            }
        }
    }
}

impl From<DynamicPrimitiveJson> for DynamicPrimitive {
    fn from(value: DynamicPrimitiveJson) -> Self {
        match value {
            DynamicPrimitiveJson::Lbind => Self::Lbind,
            DynamicPrimitiveJson::Apply => Self::Apply,
            DynamicPrimitiveJson::Rbind => Self::Rbind,
            DynamicPrimitiveJson::HistoricalApply => {
                Self::HistoricalApply
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ValueJson {
    Boolean(Boolean),
    Integer(Integer),
    Grid(Grid),
    IntegerTuple(IntegerTuple),
    Indices(Indices),
    Object(Object),
    IntegerVector(IntegerVector),
    GridVector(GridVector),
    TupleVector(TupleVector),
    ObjectVector(ObjectVector),
    Objects(Objects),
}

impl TryFrom<Value> for ValueJson {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(value) => Ok(Self::Boolean(value)),
            Value::Integer(value) => Ok(Self::Integer(value)),
            Value::Grid(value) => Ok(Self::Grid(value)),
            Value::IntegerTuple(value) => {
                Ok(Self::IntegerTuple(value))
            }
            Value::Indices(value) => Ok(Self::Indices(value)),
            Value::Object(value) => Ok(Self::Object(value)),
            Value::IntegerVector(value) => {
                Ok(Self::IntegerVector(value))
            }
            Value::GridVector(value) => {
                Ok(Self::GridVector(value))
            }
            Value::TupleVector(value) => {
                Ok(Self::TupleVector(value))
            }
            Value::ObjectVector(value) => {
                Ok(Self::ObjectVector(value))
            }
            Value::Objects(value) => Ok(Self::Objects(value)),

            Value::Function(_) => Err(
                "Function values require function serialization"
                    .to_string(),
            ),
        }
    }
}

impl TryFrom<ValueJson> for Value {
    type Error = String;

    fn try_from(value: ValueJson) -> Result<Self, Self::Error> {
        match value {
            ValueJson::Boolean(value) => Ok(Self::Boolean(value)),
            ValueJson::Integer(value) => Ok(Self::Integer(value)),
            ValueJson::Grid(value) => Ok(Self::Grid(value)),
            ValueJson::IntegerTuple(value) => {
                Ok(Self::IntegerTuple(value))
            }
            ValueJson::Indices(value) => Ok(Self::Indices(value)),
            ValueJson::Object(value) => Ok(Self::Object(value)),
            ValueJson::IntegerVector(value) => {
                Ok(Self::IntegerVector(value))
            }
            ValueJson::GridVector(value) => {
                Ok(Self::GridVector(value))
            }
            ValueJson::TupleVector(value) => {
                Ok(Self::TupleVector(value))
            }
            ValueJson::ObjectVector(value) => {
                Ok(Self::ObjectVector(value))
            }
            ValueJson::Objects(value) => Ok(Self::Objects(value)),
        }
    }
}

pub fn apply_container_type(
    container_type: Type,
    mapped_type: Type,
) -> Result<Type, String> {
    match container_type {
        Type::IntegerTuple | Type::IntegerVector => {
            if mapped_type == Type::Integer {
                Ok(Type::IntegerVector)
            } else {
                Err(format!(
                    "historical apply: {:?} cannot be mapped from {:?}",
                    mapped_type,
                    container_type
                ))
            }
        }

        Type::GridVector => match mapped_type {
            Type::Integer => Ok(Type::IntegerVector),
            Type::Grid => Ok(Type::GridVector),
            other => Err(format!(
                "historical apply: unsupported GridVector mapped output {:?}",
                other
            )),
        },

        Type::ObjectVector => {
            if mapped_type == Type::Object {
                Ok(Type::ObjectVector)
            } else {
                Err(format!(
                    "historical apply: unsupported ObjectVector mapped output {:?}",
                    mapped_type
                ))
            }
        }

        Type::Objects => match mapped_type {
            Type::Object => Ok(Type::Objects),
            Type::Integer => Ok(Type::IntegerVector),
            other => Err(format!(
                "historical apply: unsupported Objects mapped output {:?}",
                other
            )),
        },

        other => Err(format!(
            "historical apply: unsupported container type {:?}",
            other
        )),
    }
}

#[derive(Debug, Clone)]
pub struct NamedTerminal {
    pub name: &'static str,
    pub value: Value,
}

impl NamedTerminal {
    pub fn new(
        name: &'static str,
        value: Value,
    ) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Clone)]
pub struct Terminals {
    pub values: Vec<NamedTerminal>,
}

impl Terminals {
    pub fn arc_agi() -> Self {
        Self {
            values: vec![
                NamedTerminal::new("F", Value::Boolean(false)),
                NamedTerminal::new("T", Value::Boolean(true)),

                NamedTerminal::new("ZERO", Value::Integer(0)),
                NamedTerminal::new("ONE", Value::Integer(1)),
                NamedTerminal::new("TWO", Value::Integer(2)),
                NamedTerminal::new("THREE", Value::Integer(3)),
                NamedTerminal::new("FOUR", Value::Integer(4)),
                NamedTerminal::new("FIVE", Value::Integer(5)),
                NamedTerminal::new("SIX", Value::Integer(6)),
                NamedTerminal::new("SEVEN", Value::Integer(7)),
                NamedTerminal::new("EIGHT", Value::Integer(8)),
                NamedTerminal::new("NINE", Value::Integer(9)),
                NamedTerminal::new("TEN", Value::Integer(10)),
                NamedTerminal::new("NEG_ONE", Value::Integer(-1)),
                NamedTerminal::new("NEG_TWO", Value::Integer(-2)),

                NamedTerminal::new("DOWN", Value::IntegerTuple((-1, 0))),
                NamedTerminal::new("RIGHT", Value::IntegerTuple((0, 1))),
                NamedTerminal::new("UP", Value::IntegerTuple((1, 0))),
                NamedTerminal::new("LEFT", Value::IntegerTuple((0, -1))),

                NamedTerminal::new("ORIGIN", Value::IntegerTuple((0, 0))),
                NamedTerminal::new("UNITY", Value::IntegerTuple((1, 1))),
                NamedTerminal::new("NEG_UNITY", Value::IntegerTuple((-1, -1))),
                NamedTerminal::new("UP_RIGHT", Value::IntegerTuple((-1, 1))),
                NamedTerminal::new("DOWN_LEFT", Value::IntegerTuple((1, -1))),

                NamedTerminal::new("ZERO_BY_TWO", Value::IntegerTuple((0, 2))),
                NamedTerminal::new("TWO_BY_ZERO", Value::IntegerTuple((2, 0))),
                NamedTerminal::new("TWO_BY_TWO", Value::IntegerTuple((2, 2))),
                NamedTerminal::new("THREE_BY_THREE", Value::IntegerTuple((3, 3))),
            ],
        }
    }
}

use crate::environment::InputEnvironment;

pub struct Dataset {
    pub environments: Vec<InputEnvironment>,
}

impl Dataset {
    pub fn from_grids(grids: Vec<Grid>) -> Self {
        let environments = grids
            .into_iter()
            .map(|grid| {
                let mut environment = InputEnvironment::new();

                environment.insert(
                    "I".to_string(),
                    Value::Grid(grid),
                );

                environment
            })
            .collect();

        Self { environments }
    }
}

#[derive(Debug, Clone)]
pub enum Connection {
    Input {
        name: String,
        ty: Type,
    },

    Constant {
        name: String,
        value: Value,
    },

    CallablePrimitive {
        primitive: &'static PrimitiveEntry,
    },

    Primitive {
        primitive: &'static PrimitiveEntry,
        inputs: Vec<Box<Connection>>,
    },

    Dynamic {
        primitive: DynamicPrimitive,
        inputs: Vec<Box<Connection>>,
        output: Type,
        family: Option<&'static str>,
        fixed: Option<Box<Connection>>,
        left: bool,
    },
}

impl Connection {
    pub fn new(
        primitive: &'static PrimitiveEntry,
        inputs: Vec<Box<Connection>>,
    ) -> Result<Self, String> {
        if primitive.inputs.len() != inputs.len() {
            return Err(format!(
                "{} expects {} inputs, got {}",
                primitive.name,
                primitive.inputs.len(),
                inputs.len()
            ));
        }

        for (input, expected_type) in inputs.iter().zip(primitive.inputs.iter()) {
            let actual_type = input.output_type();

            if actual_type != *expected_type {
                return Err(format!(
                    "{} expects {:?}, got {:?}",
                    primitive.name,
                    expected_type,
                    actual_type
                ));
            }
        }

        Ok(Self::Primitive {
            primitive,
            inputs,
        })
    }

    pub fn historical_apply(
        inputs: Vec<Box<Connection>>,
        output: Type,
    ) -> Self {
        Self::Dynamic {
            primitive: DynamicPrimitive::HistoricalApply,
            inputs,
            output,
            family: None,
            fixed: None,
            left: false,
        }
    }

    pub fn output_type_with(
        &self,
        function_types: &mut crate::registry::FunctionTypeRegistry,
    ) -> Type {
        match self {
            Connection::Input { ty, .. } => *ty,

            Connection::Constant { value, .. } => {
                value.output_type_with(function_types)
            }

            Connection::CallablePrimitive { .. } => Type::Callable,

            Connection::Primitive { primitive, .. } => {
                primitive.output
            }

            Connection::Dynamic { output, .. } => *output,
        }
    }

    pub fn dynamic(
        primitive: DynamicPrimitive,
        inputs: Vec<Box<Connection>>,
        output: Type,
    ) -> Self {
        Self::Dynamic {
            primitive,
            inputs,
            output,
            family: None,
            fixed: None,
            left: false,
        }
    }

    pub fn dynamic_family(
        primitive: DynamicPrimitive,
        inputs: Vec<Box<Connection>>,
        output: Type,
        family: &'static str,
        fixed: Connection,
        left: bool,
    ) -> Self {
        Self::Dynamic {
            primitive,
            inputs,
            output,
            family: Some(family),
            fixed: Some(Box::new(fixed)),
            left,
        }
    }

    pub fn apply(
        function: Connection,
        arguments: Vec<Connection>,
        function_types: &mut crate::registry::FunctionTypeRegistry,
    ) -> Result<Self, String> {
        let function_type =
            function.output_type_with(function_types);

        let argument_types = arguments
            .iter()
            .map(|argument| {
                argument.output_type_with(function_types)
            })
            .collect::<Vec<_>>();

        if let Connection::CallablePrimitive { primitive } = &function {
            let output = function_types
                .apply_callable_family(
                    primitive.name,
                    &argument_types,
                )?;

            let mut inputs = Vec::with_capacity(
                1 + arguments.len()
            );

            inputs.push(Box::new(function));

            inputs.extend(
                arguments.into_iter().map(Box::new)
            );

            return Ok(Self::dynamic(
                DynamicPrimitive::Apply,
                inputs,
                output,
            ));
        }

        if let Connection::Dynamic {
            family: Some(family),
            output: Type::Callable,
            fixed,
            left,
            ..
        } = &function
        {
            let mut all_argument_types =
                Vec::with_capacity(argument_types.len() + 1);

            if *left {
                all_argument_types.push(
                    fixed
                        .as_ref()
                        .expect("bound callable has no fixed argument")
                        .output_type_with(function_types),
                );

                all_argument_types.extend(
                    argument_types.iter().copied()
                );
            } else {
                all_argument_types.extend(
                    argument_types.iter().copied()
                );

                all_argument_types.push(
                    fixed
                        .as_ref()
                        .expect("bound callable has no fixed argument")
                        .output_type_with(function_types),
                );
            }

            let output = function_types
                .apply_callable_family(
                    family,
                    &all_argument_types,
                )?;

            return Ok(Self::dynamic(
                DynamicPrimitive::Apply,
                {
                    let mut inputs =
                        Vec::with_capacity(1 + arguments.len());

                    inputs.push(Box::new(function));
                    inputs.extend(
                        arguments.into_iter().map(Box::new)
                    );

                    inputs
                },
                output,
            ));
        }

        let output = function_types.apply_type(
            function_type,
            &argument_types,
        )?;

        let mut inputs = Vec::with_capacity(
            1 + arguments.len()
        );

        inputs.push(Box::new(function));

        inputs.extend(
            arguments.into_iter().map(Box::new)
        );

        Ok(Self::dynamic(
            DynamicPrimitive::Apply,
            inputs,
            output,
        ))
    }

    pub fn lbind(
        function: Connection,
        fixed: Connection,
        function_types: &mut crate::registry::FunctionTypeRegistry,
    ) -> Result<Self, String> {
        let function_type =
            function.output_type_with(function_types);

        let fixed_type =
            fixed.output_type_with(function_types);

        if let Connection::CallablePrimitive { primitive } = &function {
            let family = primitive.name;

            return Ok(Self::dynamic_family(
                DynamicPrimitive::Lbind,
                vec![
                    Box::new(function),
                    Box::new(fixed.clone()),
                ],
                Type::Callable,
                family,
                fixed,
                true,
            ));
        }

        let output = function_types.lbind_type(
            function_type,
            fixed_type,
        )?;

        Ok(Self::dynamic(
            DynamicPrimitive::Lbind,
            vec![
                Box::new(function),
                Box::new(fixed),
            ],
            output,
        ))
    }

    pub fn rbind(
        function: Connection,
        fixed: Connection,
        function_types: &mut crate::registry::FunctionTypeRegistry,
    ) -> Result<Self, String> {
        let function_type =
            function.output_type_with(function_types);

        let fixed_type =
            fixed.output_type_with(function_types);

        if let Connection::CallablePrimitive { primitive } = &function {
            let family = primitive.name;

            return Ok(Self::dynamic_family(
                DynamicPrimitive::Rbind,
                vec![
                    Box::new(function),
                    Box::new(fixed.clone()),
                ],
                Type::Callable,
                family,
                fixed,
                true,
            ));
        }

        if let Connection::Dynamic {
            family: Some(family),
            output: Type::Callable,
            ..
        } = &function
        {
            return Ok(Self::dynamic_family(
                DynamicPrimitive::Rbind,
                vec![
                    Box::new(function.clone()),
                    Box::new(fixed.clone()),
                ],
                Type::Callable,
                family,
                fixed,
                false,
            ));
        }

        let output = function_types.rbind_type(
            function_type,
            fixed_type,
        )?;

        Ok(Self::dynamic(
            DynamicPrimitive::Rbind,
            vec![
                Box::new(function),
                Box::new(fixed),
            ],
            output,
        ))
    }

    pub fn output_with_inputs(
        &self,
        environment: &InputEnvironment,
    ) -> Result<Value, String> {
        match self {
            Connection::Input { name, ty } => {
                let value = environment
                    .get(name)
                    .ok_or_else(|| {
                        format!("missing input '{}'", name)
                    })?;

                if value.output_type() != *ty {
                    return Err(format!(
                        "input '{}' expects {:?}, got {:?}",
                        name,
                        ty,
                        value.output_type()
                    ));
                }

                Ok(value.clone())
            }

            Connection::Constant { name: _, value } => {
                Ok(value.clone())
            }

            Connection::CallablePrimitive { primitive } => {
                Ok(Value::Function(Box::new(
                    Function::primitive_family(primitive),
                )))
            }

            Connection::Primitive {
                primitive,
                inputs,
            } => {
                let mut values = Vec::with_capacity(inputs.len());

                for input in inputs {
                    values.push(
                        input.output_with_inputs(environment)?
                    );
                }

                (primitive.apply)(&values)
            }
            
            Connection::Dynamic {
                primitive,
                inputs,
                ..
            } => {
                let mut values = Vec::with_capacity(inputs.len());

                for input in inputs {
                    values.push(
                        input.output_with_inputs(environment)?
                    );
                }

                match primitive {
                    DynamicPrimitive::Lbind => {
                        if values.len() != 2 {
                            return Err(format!(
                                "lbind expects 2 arguments, got {}",
                                values.len()
                            ));
                        }

                        crate::registry::lbind_value(
                            &values[0],
                            &values[1],
                        )
                    }

                    DynamicPrimitive::Rbind => {
                        if values.len() != 2 {
                            return Err(format!(
                                "rbind expects 2 arguments, got {}",
                                values.len()
                            ));
                        }

                        crate::registry::rbind_value(
                            &values[0],
                            &values[1],
                        )
                    }

                    DynamicPrimitive::Apply => {
                        if values.is_empty() {
                            return Err(
                                "apply expects a function".to_string()
                            );
                        }

                        let function = &values[0];
                        let arguments = values[1..].to_vec();

                        crate::registry::apply_values(
                            function,
                            arguments,
                        )
                    }

                    DynamicPrimitive::HistoricalApply => {
                        if values.len() != 2 {
                            return Err(
                                "historical apply expects function and container"
                                    .to_string()
                            );
                        }

                        crate::registry::map_apply_value(
                            &values[0],
                            &values[1],
                        )
                    }
                }
            }
        }
    }

    pub fn is_open(&self) -> bool {
        match self {
            Connection::Input { name: _, ty: _ } => true,

            Connection::Constant { name: _, value: _} => false,

            Connection::CallablePrimitive { .. } => {
                todo!("CallablePrimitive behavior will be implemented next")
            }

            Connection::Primitive { inputs, .. } => {
                inputs.iter().any(|input| input.is_open())
            }

            Connection::Dynamic { inputs, .. } => {
                inputs.iter().any(|input| input.is_open())
            }
        }
    }

    pub fn terminal(value: Value) -> Self {
        let name = match &value {
            Value::Boolean(value) => value.to_string(),
            Value::Integer(value) => value.to_string(),
            Value::IntegerTuple((a, b)) => {
                format!("({}, {})", a, b)
            }
            Value::Grid(_) => "Grid".to_string(),
            Value::Indices(_) => "Indices".to_string(),
            Value::Object(_) => "Object".to_string(),
            &Value::Function(_) => "Function".to_string(),
            Value::IntegerVector(_) => "IntegerVector".to_string(),
            Value::GridVector(_) => "GridVector".to_string(),
            Value::TupleVector(_) => "TupleVector".to_string(),
            Value::ObjectVector(_) => "ObjectVector".to_string(),
            Value::Objects(_) => "Objects".to_string(),
        };

        Self::Constant { name, value }
    }

    pub fn named_terminal(
        name: impl Into<String>,
        value: Value,
    ) -> Self {
        Self::Constant {
            name: name.into(),
            value,
        }
    }
    
    pub fn input(
        name: impl Into<String>,
        input_type: Type,
    ) -> Self {
        Self::Input {
            name: name.into(),
            ty: input_type,
        }
    }

    pub fn output_type(&self) -> Type {
        match self {
            Connection::Primitive { primitive, .. } => primitive.output,
            Connection::Constant { name: _, value } => value.output_type(),
            Connection::Input { ty, .. } => {
                *ty
            }
            Connection::CallablePrimitive { .. } => Type::Callable,
            Connection::Dynamic { output, .. } => *output,
        }
    }

    pub fn output(&self) -> Result<Value, String> {
        if self.is_open() {
            return Err("cannot evaluate an open program without an input".to_string());
        }

        match self {
            Connection::Input { .. } => {
                unreachable!("open inputs are rejected above")
            }

            Connection::Constant { name: _, value } => {
                Ok(value.clone())
            }

            Connection::CallablePrimitive { .. } => {
                todo!("CallablePrimitive behavior will be implemented next")
            }

            Connection::Primitive {
                primitive,
                inputs,
            } => {
                let mut values = Vec::with_capacity(inputs.len());

                for input in inputs {
                    values.push(input.output()?);
                }

                (primitive.apply)(&values)
            }

            Connection::Dynamic { .. } => {
                let environment = InputEnvironment::new();
                self.output_with_inputs(&environment)
            }
        }
    }

    pub fn expression(&self) -> String {
        match self {
            Connection::Input { name, .. } => {
                name.clone()
            }

            Connection::Constant { name, .. } => {
                name.clone()
            }

            Connection::CallablePrimitive { primitive } => {
                primitive.name.to_string()
            }

            Connection::Primitive { primitive, inputs } => {
                let args = inputs
                    .iter()
                    .map(|input| input.expression())
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{}({})", primitive.name, args)
            }

            Connection::Dynamic {
                primitive,
                inputs,
                ..
            } => {
                let name = primitive.name();

                let arguments = inputs
                    .iter()
                    .map(|input| input.expression())
                    .collect::<Vec<_>>();

                format!(
                    "{}({})",
                    name,
                    arguments.join(", ")
                )
            }
        }
    }
    
    pub fn callable_primitive(
        primitive: &'static PrimitiveEntry,
    ) -> Self {
        Self::CallablePrimitive { primitive }
    }

    pub fn substitute_input(
        &self,
        name: &str,
        replacement: &Connection,
    ) -> Connection {
        match self {
            Connection::Input { name: input_name, .. } => {
                if input_name == name {
                    replacement.clone()
                } else {
                    self.clone()
                }
            }

            Connection::Constant { .. } => {
                self.clone()
            }

            Connection::CallablePrimitive { .. } => {
                todo!("CallablePrimitive behavior will be implemented next")
            }

            Connection::Primitive {
                primitive,
                inputs,
            } => {
                let inputs = inputs
                    .iter()
                    .map(|input| {
                        Box::new(
                            input.substitute_input(
                                name,
                                replacement,
                            )
                        )
                    })
                    .collect();

                Connection::new(
                    primitive,
                    inputs,
                )
                .expect("substitution must preserve primitive signature")
            }

            Connection::Dynamic {
                primitive,
                inputs,
                output,
                family,
                fixed,
                left,
            } => {
                let inputs = inputs
                    .iter()
                    .map(|input| {
                        Box::new(
                            input.substitute_input(
                                name,
                                replacement,
                            )
                        )
                    })
                    .collect();

                Connection::Dynamic {
                    primitive: *primitive,
                    inputs,
                    output: *output,
                    family: *family,
                    fixed: fixed
                        .as_ref()
                        .map(|fixed| Box::new(fixed.substitute_input(name, replacement))),
                    left: *left,
                }
            }
        }
    }

    pub fn to_json(&self) -> Result<ConnectionJson, String> {
        match self {
            Connection::Input { name, ty } => {
                Ok(ConnectionJson::Input {
                    name: name.clone(),
                    ty: (*ty).try_into()?,
                })
            }

            Connection::Constant { name, value } => {
                Ok(ConnectionJson::Constant {
                    name: name.clone(),
                    value: value.clone().try_into()?,
                })
            }

            Connection::CallablePrimitive { primitive } => {
                Ok(ConnectionJson::CallablePrimitive {
                    primitive: primitive.name.to_string(),
                })
            }

            Connection::Primitive {
                primitive,
                inputs,
            } => {
                Ok(ConnectionJson::Primitive {
                    primitive: primitive.name.to_string(),
                    inputs: inputs
                        .iter()
                        .map(|input| input.to_json())
                        .collect::<Result<Vec<_>, _>>()?,
                })
            }

            Connection::Dynamic {
                primitive,
                inputs,
                output,
                family,
                fixed,
                left,
            } => {
                Ok(ConnectionJson::Dynamic {
                    primitive: (*primitive).into(),
                    inputs: inputs
                        .iter()
                        .map(|input| input.to_json())
                        .collect::<Result<Vec<_>, _>>()?,
                    output: (*output).try_into()?,
                    family: family.map(str::to_string),
                    fixed: fixed
                        .as_ref()
                        .map(|fixed| fixed.to_json())
                        .transpose()?
                        .map(Box::new),
                    left: *left,
                })
            }
        }
    }
}

impl Value {
    pub fn output_type(&self) -> Type {
        match self {
            Value::Boolean(_) => Type::Boolean,
            Value::Integer(_) => Type::Integer,
            Value::Grid(_) => Type::Grid,
            Value::IntegerTuple(_) => Type::IntegerTuple,
            Value::Indices(_) => Type::Indices,
            Value::Object(_) => Type::Object,
            Value::Function(_) => {
                // provisoirement impossible à déterminer sans registre
                todo!()
            }
            Value::IntegerVector(_) => Type::IntegerVector,
            Value::GridVector(_) => Type::GridVector,
            Value::TupleVector(_) => Type::TupleVector,
            Value::ObjectVector(_) => Type::ObjectVector,
            Value::Objects(_) => Type::Objects,
        }
    }

    pub fn ty(&self) -> Type {
        match self {
            Value::Boolean(_) => Type::Boolean,
            Value::Integer(_) => Type::Integer,
            Value::IntegerTuple(_) => Type::IntegerTuple,
            Value::Grid(_) => Type::Grid,
            Value::Indices(_) => Type::Indices,
            Value::Object(_) => Type::Object,
            Value::Function(_) => {
                todo!()
            }
            Value::IntegerVector(_) => Type::IntegerVector,
            Value::GridVector(_) => Type::GridVector,
            Value::TupleVector(_) => Type::TupleVector,
            Value::ObjectVector(_) => Type::ObjectVector,
            Value::Objects(_) => Type::Objects,
        }
    }

    pub fn output_type_with(
        &self,
        function_types: &mut FunctionTypeRegistry,
    ) -> Type {
        match self {
            Value::Boolean(_) => Type::Boolean,
            Value::Integer(_) => Type::Integer,
            Value::IntegerTuple(_) => Type::IntegerTuple,
            Value::Grid(_) => Type::Grid,
            Value::Indices(_) => Type::Indices,
            Value::Object(_) => Type::Object,
            Value::Function(function) => {
                function_types.type_of(
                    &function.inputs
                        .iter()
                        .map(|input| input.ty)
                        .collect::<Vec<_>>(),
                    function.body_type(),
                )
            }
            Value::IntegerVector(_) => Type::IntegerVector,
            Value::GridVector(_) => Type::GridVector,
            Value::TupleVector(_) => Type::TupleVector,
            Value::ObjectVector(_) => Type::ObjectVector,
            Value::Objects(_) => Type::Objects,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Type;
    use crate::search::{generate, semantic_signature, generate_inputs};
    use crate::signature::InputSpec;
    use crate::registry::{PRIMITIVES, Value, find_primitive};
    use crate::function::{Function, FunctionKind};
    use rand::SeedableRng;

    #[test]
    fn integer_generation_has_expected_cardinality() {
        let terminals = Terminals {
            values: vec![
                NamedTerminal::new("ONE", Value::Integer(1)),
                NamedTerminal::new("TWO", Value::Integer(2)),
                NamedTerminal::new("THREE", Value::Integer(3)),
            ],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let depth_1 = generate(
            Type::Integer,
            1,
            &terminals,
            &inputs,
            &mut function_types,
        );

        let depth_2 = generate(
            Type::Integer,
            2,
            &terminals,
            &inputs,
            &mut function_types,
        );

        assert_eq!(depth_1.len(), 63);
        assert_eq!(depth_2.len(), 119671);
    }

    #[test]
    fn same_program_runs_on_different_grids() {
        let primitive = crate::registry::find_by_name_and_inputs(
                "hmirror",
                &[Type::Grid],
            )
            .unwrap();

        let program = Connection::new(
            primitive,
            vec![Box::new(
                Connection::input("I", Type::Grid),
            )],
        )
        .unwrap();

        let environments_a = vec![
            {
                let mut environment = InputEnvironment::new();

                environment.insert(
                    "I".to_string(),
                    Value::Grid(vec![
                        vec![1, 2, 3],
                        vec![4, 5, 6],
                    ]),
                );

                environment
            },
        ];

        let environments_b = vec![
            {
                let mut environment = InputEnvironment::new();

                environment.insert(
                    "I".to_string(),
                    Value::Grid(vec![
                        vec![7, 8],
                        vec![9, 0],
                    ]),
                );

                environment
            },
        ];

        let signature_a = semantic_signature(
            &program,
            &environments_a,
        )
        .unwrap();

        let signature_b = semantic_signature(
            &program,
            &environments_b,
        )
        .unwrap();

        assert_eq!(
            signature_a,
            vec![
                vec![
                    vec![3, 2, 1],
                    vec![6, 5, 4],
                ],
            ]
        );

        assert_eq!(
            signature_b,
            vec![
                vec![
                    vec![8, 7],
                    vec![0, 9],
                ],
            ]
        );
    }

    #[test]
    fn generated_grid_programs_use_named_input() {
        let terminals = Terminals {
            values: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::Grid,
            1,
            &terminals,
            &inputs,
            &mut function_types,
        );

        let expressions: Vec<String> = programs
            .iter()
            .map(|program| program.expression())
            .collect();

        assert!(expressions.contains(&"hmirror(I)".to_string()));
        assert!(expressions.contains(&"vmirror(I)".to_string()));
    }

    #[test]
    fn integer_constant_evaluates() {
        let program = Connection::terminal(
            Value::Integer(42),
        );

        let result = program.output().unwrap();

        assert!(matches!(
            result,
            Value::Integer(42)
        ));
    }

    #[test]
    fn connection_tracks_whether_it_is_open() {
        let terminals = Terminals {
            values: vec![],
        };

        let closed = Connection::terminal(
            Value::Integer(42),
        );

        let open = Connection::input("I", Type::Grid);

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = crate::search::generate(
            Type::Grid,
            1,
            &terminals,
            &inputs,
            &mut function_types,
        );

        assert!(!closed.is_open());
        assert!(open.is_open());

        for program in programs {
            assert!(program.is_open());
        }
    }

    #[test]
    fn output_requires_a_closed_program() {
        let closed = Connection::terminal(
            Value::Integer(42),
        );

        assert!(closed.output().is_ok());

        let open = Connection::input("I", Type::Grid);

        let result = open.output();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "cannot evaluate an open program without an input"
        );
    }

    #[test]
    fn same_named_input_can_be_reused() {
        let input_1 = Connection::input("I", Type::Grid);
        let input_2 = Connection::input("I", Type::Grid);

        let hmirror = crate::registry::find_by_name_and_inputs(
            "hmirror",
            &[Type::Grid],
        )
        .unwrap();

        let vmirror = crate::registry::find_by_name_and_inputs(
            "vmirror",
            &[Type::Grid],
        )
        .unwrap();

        let program_1 = Connection::new(
            hmirror,
            vec![Box::new(input_1)],
        )
        .unwrap();

        let program_2 = Connection::new(
            vmirror,
            vec![Box::new(input_2)],
        )
        .unwrap();

        let grid = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
        ];

        let mut environment = InputEnvironment::new();

        environment.insert(
            "I".to_string(),
            Value::Grid(grid),
        );

        let result_1 = program_1
            .output_with_inputs(&environment)
            .unwrap();

        let result_2 = program_2
            .output_with_inputs(&environment)
            .unwrap();

        assert!(matches!(
            result_1,
            Value::Grid(ref grid)
                if *grid == vec![
                    vec![3, 2, 1],
                    vec![6, 5, 4],
                ]
        ));

        assert!(matches!(
            result_2,
            Value::Grid(ref grid)
                if *grid == vec![
                    vec![4, 5, 6],
                    vec![1, 2, 3],
                ]
        ));
    }

    #[test]
    fn same_input_can_feed_multiple_arguments() {
        let input_1 = Connection::input("I", Type::Grid);
        let input_2 = Connection::input("I", Type::Grid);

        let hmirror = crate::registry::find_by_name_and_inputs(
            "hmirror",
            &[Type::Grid],
        )
        .unwrap();

        let vconcat = crate::registry::find_by_name_and_inputs(
            "vconcat",
            &[Type::Grid, Type::Grid],
        )
        .unwrap();

        let mirrored = Connection::new(
            hmirror,
            vec![Box::new(input_1)],
        )
        .unwrap();

        let program = Connection::new(
            vconcat,
            vec![
                Box::new(mirrored),
                Box::new(input_2),
            ],
        )
        .unwrap();

        assert_eq!(
            program.expression(),
            "vconcat(hmirror(I), I)"
        );

        let grid = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
        ];

        let mut environment = InputEnvironment::new();

        environment.insert(
            "I".to_string(),
            Value::Grid(grid),
        );

        let result = program
            .output_with_inputs(&environment)
            .unwrap();

        assert!(matches!(
            result,
            Value::Grid(ref grid)
                if *grid == vec![
                    vec![3, 2, 1],
                    vec![6, 5, 4],
                    vec![1, 2, 3],
                    vec![4, 5, 6],
                ]
        ));
    }

    #[test]
    fn named_inputs_can_bind_to_different_values() {
        let input_i = Connection::input("I", Type::Grid);
        let input_j = Connection::input("J", Type::Grid);

        let vconcat = find_primitive(
            "vconcat",
            &[Type::Grid, Type::Grid],
            Type::Grid,
        );

        let program = Connection::new(
            vconcat,
            vec![
                Box::new(input_i),
                Box::new(input_j),
            ],
        )
        .unwrap();

        let grid_i = vec![
            vec![1, 2],
            vec![3, 4],
        ];

        let grid_j = vec![
            vec![5, 6],
        ];

        let mut environment = InputEnvironment::new();

        environment.insert(
            "I".to_string(),
            Value::Grid(grid_i),
        );

        environment.insert(
            "J".to_string(),
            Value::Grid(grid_j),
        );

        let result = program
            .output_with_inputs(&environment)
            .unwrap();

        assert!(matches!(
            result,
            Value::Grid(ref grid)
                if *grid == vec![
                    vec![1, 2],
                    vec![3, 4],
                    vec![5, 6],
                ]
        ));
    }

    #[test]
    fn input_generation_uses_available_named_inputs() {
        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
            InputSpec {
                name: "J".to_string(),
                ty: Type::Grid,
            },
            InputSpec {
                name: "N".to_string(),
                ty: Type::Integer,
            },
        ];

        let programs = generate_inputs(
            Type::Grid,
            &inputs,
        );

        assert_eq!(programs.len(), 2);

        let expressions = programs
            .iter()
            .map(Connection::expression)
            .collect::<Vec<_>>();

        assert!(expressions.contains(&"I".to_string()));
        assert!(expressions.contains(&"J".to_string()));
        assert!(!expressions.contains(&"N".to_string()));
    }

    #[test]
    fn grid_generation_supports_multiple_named_inputs() {
        let terminals = Terminals {
            values: vec![],
        };

        let inputs = vec![
            InputSpec {
                name: "I".to_string(),
                ty: Type::Grid,
            },
            InputSpec {
                name: "J".to_string(),
                ty: Type::Grid,
            },
        ];

        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let programs = generate(
            Type::Grid,
            1,
            &terminals,
            &inputs,
            &mut function_types,
        );

        assert_eq!(programs.len(), 34);

        let expressions = programs
            .iter()
            .map(Connection::expression)
            .collect::<Vec<_>>();

        assert!(expressions.contains(&"hmirror(I)".to_string()));
        assert!(expressions.contains(&"hmirror(J)".to_string()));
        assert!(expressions.contains(&"vmirror(I)".to_string()));
        assert!(expressions.contains(&"vmirror(J)".to_string()));

        assert!(expressions.contains(
            &"vconcat(I, I)".to_string()
        ));

        assert!(expressions.contains(
            &"vconcat(I, J)".to_string()
        ));

        assert!(expressions.contains(
            &"vconcat(J, I)".to_string()
        ));

        assert!(expressions.contains(
            &"vconcat(J, J)".to_string()
        ));

        assert!(expressions.contains(
            &"hconcat(I, I)".to_string()
        ));

        assert!(expressions.contains(
            &"hconcat(I, J)".to_string()
        ));

        assert!(expressions.contains(
            &"hconcat(J, I)".to_string()
        ));

        assert!(expressions.contains(
            &"hconcat(J, J)".to_string()
        ));
    }

    #[test]
    fn dataset_can_hold_multiple_named_inputs() {
        let grid_i = vec![
            vec![1, 2],
        ];

        let grid_j = vec![
            vec![3, 4],
        ];

        let mut environment = InputEnvironment::new();

        environment.insert(
            "I".to_string(),
            Value::Grid(grid_i.clone()),
        );

        environment.insert(
            "J".to_string(),
            Value::Grid(grid_j.clone()),
        );

        let dataset = Dataset {
            environments: vec![
                environment,
            ],
        };

        assert_eq!(
            dataset.environments.len(),
            1,
        );

        assert!(matches!(
            dataset.environments[0].get("I"),
            Some(Value::Grid(grid)) if *grid == grid_i
        ));

        assert!(matches!(
            dataset.environments[0].get("J"),
            Some(Value::Grid(grid)) if *grid == grid_j
        ));
    }

    #[test]
    fn indices_connection_can_feed_ulcorner() {
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

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::terminal(
                    Value::Indices(indices),
                )),
            ],
        )
        .expect("connection should be valid");

        let result = connection
            .output()
            .expect("connection should evaluate");

        assert!(matches!(
            result,
            Value::IntegerTuple((2, 1))
        ));
    }

    #[test]
    fn all_corner_connections_produce_expected_values() {
        let indices = std::collections::BTreeSet::from([
            (2, 5),
            (4, 1),
            (7, 9),
        ]);

        let cases = [
            ("ulcorner", (2, 1)),
            ("urcorner", (2, 9)),
            ("llcorner", (7, 1)),
            ("lrcorner", (7, 9)),
        ];

        for (name, expected) in cases {
            let primitive = find_primitive(
                name,
                &[Type::Indices],
                Type::IntegerTuple,
            );

            let connection = Connection::new(
                primitive,
                vec![
                    Box::new(Connection::terminal(
                        Value::Indices(indices.clone()),
                    )),
                ],
            )
            .expect("connection should be valid");

            let result = connection
                .output()
                .expect("connection should evaluate");

            assert!(
                matches!(result, Value::IntegerTuple(value) if value == expected),
                "unexpected result for {name}: {result:?}"
            );
        }
    }

    #[test]
    fn crop_connection_can_be_evaluated() {
        let grid = vec![
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 0],
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 0],
        ];

        let crop = find_primitive(
            "crop",
            &[Type::Grid, Type::IntegerTuple, Type::IntegerTuple],
            Type::Grid,
        );

        let connection = Connection::new(
            crop,
            vec![
                Box::new(Connection::terminal(Value::Grid(grid))),
                Box::new(Connection::terminal(
                    Value::IntegerTuple((1, 1)),
                )),
                Box::new(Connection::terminal(
                    Value::IntegerTuple((2, 3)),
                )),
            ],
        )
        .expect("crop connection should be valid");

        assert!(matches!(
            connection.output(),
            Ok(Value::Grid(grid))
                if grid == vec![
                    vec![7, 8, 9],
                    vec![2, 3, 4],
                ]
        ));
    }

    #[test]
    fn add_overloads_can_be_connected_and_evaluated() {
        let integer_integer = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .unwrap();

        let program = Connection::new(
            integer_integer,
            vec![
                Box::new(Connection::terminal(Value::Integer(2))),
                Box::new(Connection::terminal(Value::Integer(3))),
            ],
        )
        .unwrap();

        match program.output().unwrap() {
            Value::Integer(value) => assert_eq!(value, 5),
            other => panic!("expected Integer, got {:?}", other),
        }

        let tuple_tuple = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::IntegerTuple, Type::IntegerTuple],
        )
        .unwrap();

        let program = Connection::new(
            tuple_tuple,
            vec![
                Box::new(Connection::terminal(
                    Value::IntegerTuple((2, 3)),
                )),
                Box::new(Connection::terminal(
                    Value::IntegerTuple((4, 5)),
                )),
            ],
        )
        .unwrap();

        match program.output().unwrap() {
            Value::IntegerTuple(value) => {
                assert_eq!(value, (6, 8));
            }
            other => panic!("expected IntegerTuple, got {:?}", other),
        }

        let integer_tuple = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::IntegerTuple],
        )
        .unwrap();

        let program = Connection::new(
            integer_tuple,
            vec![
                Box::new(Connection::terminal(Value::Integer(2))),
                Box::new(Connection::terminal(
                    Value::IntegerTuple((4, 5)),
                )),
            ],
        )
        .unwrap();

        match program.output().unwrap() {
            Value::IntegerTuple(value) => {
                assert_eq!(value, (6, 7));
            }
            other => panic!("expected IntegerTuple, got {:?}", other),
        }

        let tuple_integer = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::IntegerTuple, Type::Integer],
        )
        .unwrap();

        let program = Connection::new(
            tuple_integer,
            vec![
                Box::new(Connection::terminal(
                    Value::IntegerTuple((4, 5)),
                )),
                Box::new(Connection::terminal(Value::Integer(2))),
            ],
        )
        .unwrap();

        match program.output().unwrap() {
            Value::IntegerTuple(value) => {
                assert_eq!(value, (6, 7));
            }
            other => panic!("expected IntegerTuple, got {:?}", other),
        }
    }

    #[test]
    fn expression_with_typed_input_can_be_evaluated() {
        let input = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let x = input.connection();

        let two = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let add = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("add(Integer, Integer) must exist");

        let program = Connection::new(
            add,
            vec![Box::new(two), Box::new(x)],
        )
        .expect("add(TWO, X) must be valid");

        let mut environment = InputEnvironment::new();
        environment.insert(
            "X".to_string(),
            Value::Integer(3),
        );

        let result = program
            .output_with_inputs(&environment)
            .expect("program must evaluate");

        assert_eq!(result.output_type(), Type::Integer);

        match result {
            Value::Integer(value) => assert_eq!(value, 5),
            other => panic!("expected Integer(5), got {:?}", other),
        }
    }

    #[test]
    fn symbolic_function_can_be_constructed() {
        let input = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let x = input.connection();

        let two = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let add = crate::registry::find_by_name_and_inputs(
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

        assert_eq!(function.inputs[0].ty, Type::Integer);
        assert_eq!(function.body.output_type(), Type::Integer);
    }

    #[test]
    fn symbolic_function_body_can_be_evaluated() {
        let input = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let x = input.connection();

        let two = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let add = crate::registry::find_by_name_and_inputs(
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

        let mut environment = InputEnvironment::new();
        environment.insert(
            "X".to_string(),
            Value::Integer(3),
        );

        let result = function
            .body
            .output_with_inputs(&environment)
            .expect("function body must evaluate");

        match result {
            Value::Integer(value) => assert_eq!(value, 5),
            other => panic!("expected Integer(5), got {:?}", other),
        }
    }

    #[test]
    fn connection_can_substitute_an_input() {
        let input = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let x = input.connection();

        let two = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let add = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("add(Integer, Integer) must exist");

        let expression = Connection::new(
            add,
            vec![
                Box::new(x.clone()),
                Box::new(x.clone()),
            ],
        )
        .expect("add(X, X) must be valid");

        let substituted =
            expression.substitute_input("X", &two);

        assert_eq!(
            substituted.expression(),
            "add(TWO, TWO)"
        );
    }

    #[test]
    fn lbind_connection_has_bound_function_type() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function_type = function_types.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
                crate::signature::InputSpec {
                    name: "Y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let fixed = Connection::terminal(
            Value::Integer(10),
        );

        let bound = Connection::lbind(
            function,
            fixed,
            &mut function_types,
        )
        .unwrap();

        let expected = function_types
            .lbind_type(
                function_type,
                Type::Integer,
            )
            .unwrap();

        assert_eq!(
            bound.output_type(),
            expected,
        );
    }

    #[test]
    fn lbind_connection_has_symbolic_expression() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
                crate::signature::InputSpec {
                    name: "Y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let fixed = Connection::terminal(
            Value::Integer(10),
        );

        let bound = Connection::lbind(
            function,
            fixed,
            &mut function_types,
        )
        .unwrap();

        assert_eq!(
            bound.expression(),
            "lbind(Function, 10)"
        );
    }

    #[test]
    fn lbind_connection_evaluates_to_function() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
                crate::signature::InputSpec {
                    name: "Y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let fixed = Connection::terminal(
            Value::Integer(10),
        );

        let bound = Connection::lbind(
            function,
            fixed,
            &mut function_types,
        )
        .unwrap();

        let value = bound.output().unwrap();

        match value {
            Value::Function(function) => {
                assert_eq!(function.inputs.len(), 1);
                assert_eq!(
                    function.inputs[0].ty,
                    Type::Integer
                );
                assert_eq!(
                    function.body.output_type(),
                    Type::Integer
                );

                match function.body {
                    Connection::Constant { value, .. } => {
                        match value {
                            Value::Integer(value) => {
                                assert_eq!(value, 10);
                            }
                            other => {
                                panic!(
                                    "expected integer constant, got {:?}",
                                    other
                                );
                            }
                        }
                    }

                    other => {
                        panic!(
                            "expected constant body, got {:?}",
                            other
                        );
                    }
                }
            }

            other => {
                panic!(
                    "expected Function, got {:?}",
                    other
                );
            }
        }
    }

    #[test]
    fn lbind_result_can_be_lbound_again() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
                crate::signature::InputSpec {
                    name: "Y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let first = Connection::lbind(
            function,
            Connection::terminal(Value::Integer(10)),
            &mut function_types,
        )
        .unwrap();

        let second = Connection::lbind(
            first,
            Connection::terminal(Value::Integer(20)),
            &mut function_types,
        )
        .unwrap();

        match second.output().unwrap() {
            Value::Function(function) => {
                assert!(function.inputs.is_empty());
            }

            other => {
                panic!(
                    "expected Function, got {:?}",
                    other
                );
            }
        }
    }

    #[test]
    fn apply_connection_has_function_output_type() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
                crate::signature::InputSpec {
                    name: "Y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let applied = Connection::apply(
            function,
            vec![
                Connection::terminal(Value::Integer(10)),
                Connection::terminal(Value::Integer(20)),
            ],
            &mut function_types,
        )
        .unwrap();

        assert_eq!(
            applied.output_type(),
            Type::Integer,
        );
    }

    #[test]
    fn apply_connection_has_symbolic_expression() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
                crate::signature::InputSpec {
                    name: "Y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let applied = Connection::apply(
            function,
            vec![
                Connection::terminal(Value::Integer(10)),
                Connection::terminal(Value::Integer(20)),
            ],
            &mut function_types,
        )
        .unwrap();

        assert_eq!(
            applied.expression(),
            "apply(Function, 10, 20)"
        );
    }

    #[test]
    fn apply_connection_evaluates() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
                crate::signature::InputSpec {
                    name: "Y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let applied = Connection::apply(
            function,
            vec![
                Connection::terminal(Value::Integer(10)),
                Connection::terminal(Value::Integer(20)),
            ],
            &mut function_types,
        )
        .unwrap();

        match applied.output().unwrap() {
            Value::Integer(value) => {
                assert_eq!(value, 10);
            }

            other => {
                panic!(
                    "expected Integer, got {:?}",
                    other
                );
            }
        }
    }

    #[test]
    fn substitute_input_traverses_dynamic_connection() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let applied = Connection::apply(
            function,
            vec![
                Connection::input(
                    "Y".to_string(),
                    Type::Integer,
                ),
            ],
            &mut function_types,
        )
        .unwrap();

        let replaced = applied.substitute_input(
            "Y",
            &Connection::terminal(
                Value::Integer(42),
            ),
        );

        assert_eq!(
            replaced.expression(),
            "apply(Function, 42)"
        );
    }
    
    #[test]
    fn rbind_connection_has_bound_function_type() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function_type = function_types.type_of(
            &[Type::Integer, Type::Integer],
            Type::Integer,
        );

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
                crate::signature::InputSpec {
                    name: "Y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let fixed = Connection::terminal(
            Value::Integer(10),
        );

        let bound = Connection::rbind(
            function,
            fixed,
            &mut function_types,
        )
        .unwrap();

        let expected = function_types
            .rbind_type(
                function_type,
                Type::Integer,
            )
            .unwrap();

        assert_eq!(
            bound.output_type(),
            expected,
        );
    }

    #[test]
    fn rbind_connection_has_symbolic_expression() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
                crate::signature::InputSpec {
                    name: "Y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let fixed = Connection::terminal(
            Value::Integer(10),
        );

        let bound = Connection::rbind(
            function,
            fixed,
            &mut function_types,
        )
        .unwrap();

        assert_eq!(
            bound.expression(),
            "rbind(Function, 10)"
        );
    }

    #[test]
    fn rbind_connection_evaluates_to_function() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = crate::function::Function::new(
            vec![
                crate::signature::InputSpec {
                    name: "X".to_string(),
                    ty: Type::Integer,
                },
                crate::signature::InputSpec {
                    name: "Y".to_string(),
                    ty: Type::Integer,
                },
            ],
            Connection::input(
                "X".to_string(),
                Type::Integer,
            ),
        );

        let function = Connection::terminal(
            Value::Function(Box::new(function)),
        );

        let fixed = Connection::terminal(
            Value::Integer(10),
        );

        let bound = Connection::rbind(
            function,
            fixed,
            &mut function_types,
        )
        .unwrap();

        let value = bound.output().unwrap();

        match value {
            Value::Function(function) => {
                assert_eq!(function.inputs.len(), 1);
                assert_eq!(
                    function.inputs[0].ty,
                    Type::Integer
                );
                assert_eq!(
                    function.body.output_type(),
                    Type::Integer
                );

                match function.body {
                    Connection::Input { name, .. } => {
                        assert_eq!(name, "X");
                    }

                    other => {
                        panic!(
                            "expected input body, got {:?}",
                            other
                        );
                    }
                }
            }

            other => {
                panic!(
                    "expected Function, got {:?}",
                    other
                );
            }
        }
    }

    #[test]
    fn callable_primitive_has_callable_type() {
        let connection = Connection::callable_primitive(&crate::primitives::add_entry);
        assert_eq!(connection.output_type(), Type::Callable);
    }

    #[test]
    fn callable_primitive_evaluates_to_function() {
        let connection =
            Connection::callable_primitive(&crate::primitives::add_entry);

        let environment = InputEnvironment::new();

        let value = connection
            .output_with_inputs(&environment)
            .unwrap();

        match value {
            Value::Function(function) => {
                assert!(matches!(
                    function.kind,
                    FunctionKind::PrimitiveFamily("add")
                ));
            }
            other => panic!(
                "expected Function, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn callable_primitive_can_be_applied() {
        let connection =
            Connection::callable_primitive(&crate::primitives::add_entry);

        let environment = InputEnvironment::new();

        let value = connection
            .output_with_inputs(&environment)
            .unwrap();

        let result = crate::registry::apply_values(
            &value,
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
    fn apply_connection_accepts_callable_primitive() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = Connection::callable_primitive(
            &crate::primitives::add_entry,
        );

        let applied = Connection::apply(
            function,
            vec![
                Connection::terminal(
                    Value::Integer(2),
                ),
                Connection::terminal(
                    Value::Integer(3),
                ),
            ],
            &mut function_types,
        );

        let applied = applied.unwrap();

        let environment =
            crate::environment::InputEnvironment::new();

        let result = applied
            .output_with_inputs(&environment)
            .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }

    #[test]
    fn lbind_connection_accepts_callable_primitive() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = Connection::callable_primitive(
            &crate::primitives::add_entry,
        );

        let fixed = Connection::terminal(
            Value::Integer(2),
        );

        let bound = Connection::lbind(
            function,
            fixed,
            &mut function_types,
        );

        let bound = bound.unwrap();

        let environment =
            crate::environment::InputEnvironment::new();

        let function =
            bound.output_with_inputs(&environment).unwrap();

        let result =
            crate::registry::apply_values(
                &function,
                vec![Value::Integer(3)],
            )
            .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }

    #[test]
    fn rbind_connection_accepts_callable_primitive() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function = Connection::callable_primitive(
            &crate::primitives::add_entry,
        );

        let fixed = Connection::terminal(
            Value::Integer(3),
        );

        let bound = Connection::rbind(
            function,
            fixed,
            &mut function_types,
        );

        let bound = bound.unwrap();

        let environment =
            crate::environment::InputEnvironment::new();

        let function =
            bound.output_with_inputs(&environment).unwrap();

        let result =
            crate::registry::apply_values(
                &function,
                vec![Value::Integer(2)],
            )
            .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));

        assert_eq!(
            bound.output_type(),
            Type::Callable
        );
    }

    #[test]
    fn apply_connection_accepts_lbound_callable() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function =
            Connection::callable_primitive(
                &crate::primitives::add_entry,
            );

        let bound =
            Connection::lbind(
                function,
                Connection::terminal(
                    Value::Integer(2),
                ),
                &mut function_types,
            )
            .unwrap();

        let environment =
            crate::environment::InputEnvironment::new();

        let bound_value =
            bound.output_with_inputs(&environment)
                .unwrap();

        assert!(matches!(
            bound_value,
            Value::Function(_)
        ));

        let applied =
            Connection::apply(
                bound,
                vec![
                    Connection::terminal(
                        Value::Integer(3),
                    ),
                ],
                &mut function_types,
            )
            .unwrap();

        let environment =
            crate::environment::InputEnvironment::new();

        let result =
            applied.output_with_inputs(&environment)
                .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }

    #[test]
    fn lbind_connection_has_callable_output_type() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function =
            Connection::callable_primitive(
                &crate::primitives::add_entry,
            );

        let bound =
            Connection::lbind(
                function,
                Connection::terminal(
                    Value::Integer(2),
                ),
                &mut function_types,
            )
            .unwrap();

        assert_eq!(
            bound.output_type_with(
                &mut function_types,
            ),
            Type::Callable
        );
    }

    #[test]
    fn bind_callable_twice_applies_both_fixed_arguments() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        let function =
            Connection::callable_primitive(
                &crate::primitives::add_entry,
            );

        let left_bound =
            Connection::lbind(
                function,
                Connection::terminal(
                    Value::Integer(2),
                ),
                &mut function_types,
            )
            .unwrap();

        let right_bound =
            Connection::rbind(
                left_bound,
                Connection::terminal(
                    Value::Integer(3),
                ),
                &mut function_types,
            )
            .unwrap();

        let environment =
            crate::environment::InputEnvironment::new();

        let result =
            right_bound
                .output_with_inputs(&environment)
                .unwrap();

        let function = match result {
            Value::Function(function) => function,
            other => panic!(
                "expected Function, got {other:?}"
            ),
        };

        let result = function
            .apply_values(Vec::new())
            .unwrap();

        assert!(matches!(
            result,
            Value::Integer(5)
        ));
    }    

    #[test]
    fn callable_generation_produces_callable_connections() {
        let mut function_types =
            crate::registry::FunctionTypeRegistry::new();

        crate::search::register_primitive_function_types(
            &mut function_types,
        );

        let inputs = Vec::<crate::signature::InputSpec>::new();

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let terminals = Terminals::arc_agi();

        for _ in 0..100 {
            let connection = crate::search::build_connection(
                Type::Callable,
                2,
                &terminals,
                &inputs,
                &mut function_types,
                &mut rng,
            )
            .expect("failed to generate Callable connection");

            assert_eq!(
                connection.output_type(),
                Type::Callable
            );
        }
    }

    #[test]
    fn connection_json_round_trip_serializes() {
        let primitive = crate::registry::PRIMITIVES
            .iter()
            .find(|primitive| {
                primitive.name == "hmirror"
                    && primitive.inputs == &[Type::Grid]
                    && primitive.output == Type::Grid
            })
            .expect("expected hmirror");

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
        )
        .expect("expected valid connection");

        let json = connection
            .to_json()
            .expect("expected JSON representation");

        let text =
            serde_json::to_string_pretty(&json)
                .expect("expected JSON serialization");

        println!("{text}");

        assert!(text.contains("\"kind\": \"Primitive\""));
        assert!(text.contains("\"primitive\": \"hmirror\""));
    }

    #[test]
    fn connection_json_input_deserializes() {
        let connection = Connection::input(
            "I",
            Type::Grid,
        );

        let json = connection
            .to_json()
            .expect("expected JSON representation");

        let restored =
            Connection::try_from(json)
                .expect("expected Connection");

        assert_eq!(
            restored.expression(),
            connection.expression(),
        );

        assert_eq!(
            restored.output_type(),
            connection.output_type(),
        );
    }

    #[test]
    fn connection_json_constant_deserializes() {
        let connection = Connection::Constant {
            name: "grid".to_string(),
            value: Value::Grid(vec![
                vec![1, 2],
                vec![3, 4],
            ]),
        };

        let json = connection
            .to_json()
            .expect("expected JSON representation");

        let restored =
            Connection::try_from(json)
                .expect("expected Connection");

        assert_eq!(
            restored.expression(),
            connection.expression(),
        );

        assert_eq!(
            restored.output_type(),
            connection.output_type(),
        );
    }

    #[test]
    fn connection_json_primitive_deserializes() {
        let primitive =
            crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| {
                    primitive.name == "hmirror"
                        && primitive.inputs == &[Type::Grid]
                        && primitive.output == Type::Grid
                })
                .expect("expected hmirror");

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
        )
        .expect("expected valid connection");

        let json = connection
            .to_json()
            .expect("expected JSON representation");

        let restored =
            Connection::try_from(json)
                .expect("expected Connection");

        assert_eq!(
            restored.expression(),
            connection.expression(),
        );

        assert_eq!(
            restored.output_type(),
            connection.output_type(),
        );
    }

    #[test]
    fn connection_json_callable_primitive_deserializes() {
        let primitive =
            crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| {
                    primitive.name == "identity"
                        && primitive.output == Type::Callable
                })
                .expect("expected identity callable");

        let connection =
            Connection::callable_primitive(primitive);

        let json = connection
            .to_json()
            .expect("expected JSON representation");

        let restored =
            Connection::try_from(json)
                .expect("expected Connection");

        assert_eq!(
            restored.expression(),
            connection.expression(),
        );

        assert_eq!(
            restored.output_type(),
            connection.output_type(),
        );
    }

    #[test]
    fn connection_json_dynamic_deserializes() {
        let connection = Connection::Dynamic {
            primitive: DynamicPrimitive::Apply,
            inputs: vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
            output: Type::Grid,
            family: None,
            fixed: None,
            left: false,
        };

        let json = connection
            .to_json()
            .expect("expected JSON representation");

        let restored =
            Connection::try_from(json)
                .expect("expected Connection");

        assert_eq!(
            restored.expression(),
            connection.expression(),
        );

        assert_eq!(
            restored.output_type(),
            connection.output_type(),
        );
    }

    #[test]
    fn connection_json_round_trip() {
        let primitive =
            crate::registry::PRIMITIVES
                .iter()
                .find(|primitive| {
                    primitive.name == "hmirror"
                        && primitive.inputs == &[Type::Grid]
                        && primitive.output == Type::Grid
                })
                .expect("expected hmirror");

        let connection = Connection::new(
            primitive,
            vec![
                Box::new(Connection::input(
                    "I",
                    Type::Grid,
                )),
            ],
        )
        .expect("expected valid connection");

        let json = connection
            .to_json()
            .expect("expected JSON representation");

        let text =
            serde_json::to_string_pretty(&json)
                .expect("expected JSON serialization");

        let decoded: ConnectionJson =
            serde_json::from_str(&text)
                .expect("expected JSON deserialization");

        let restored =
            Connection::try_from(decoded)
                .expect("expected Connection");

        assert_eq!(
            restored.expression(),
            connection.expression(),
        );

        assert_eq!(
            restored.output_type(),
            connection.output_type(),
        );
    }
}
