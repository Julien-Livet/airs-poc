use crate::connection::Connection;
use crate::signature::InputSpec;
use crate::registry::{Type, Value};
use crate::environment::InputEnvironment;

#[derive(Debug, Clone)]
pub struct Function {
    pub inputs: Vec<InputSpec>,
    pub body: Connection,
}

impl Function {
    pub fn input_type(&self) -> Type {
        self.inputs[0].ty
    }

    pub fn body_type(&self) -> Type {
        self.body.output_type()
    }
}

impl Function {
    pub fn new(
        inputs: Vec<InputSpec>,
        body: Connection,
    ) -> Self {
        Self {
            inputs,
            body,
        }
    }
}

impl Function {
    pub fn lbind(
        &self,
        fixed: Connection,
    ) -> Result<Function, String> {
        let first = self
            .inputs
            .first()
            .ok_or_else(|| "cannot lbind a function with no inputs".to_string())?;

        if first.ty != fixed.output_type() {
            return Err(format!(
                "lbind type mismatch: expected {:?}, got {:?}",
                first.ty,
                fixed.output_type()
            ));
        }

        let body = self.body.substitute_input(
            &first.name,
            &fixed,
        );

        let inputs = self.inputs
            .iter()
            .skip(1)
            .cloned()
            .collect();

        Ok(Function::new(inputs, body))
    }
}

impl Function {
    pub fn apply(
        &self,
        arguments: Vec<Connection>,
    ) -> Result<Connection, String> {
        if arguments.len() != self.inputs.len() {
            return Err(format!(
                "wrong number of arguments: expected {}, got {}",
                self.inputs.len(),
                arguments.len()
            ));
        }

        let mut body = self.body.clone();

        for (input, argument) in self.inputs.iter().zip(arguments.iter()) {
            if input.ty != argument.output_type() {
                return Err(format!(
                    "apply type mismatch: expected {:?}, got {:?}",
                    input.ty,
                    argument.output_type(),
                ));
            }

            body = body.substitute_input(
                &input.name,
                argument,
            );
        }

        Ok(body)
    }
}

impl Function {
    pub fn apply_values(
        &self,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        let arguments = arguments
            .into_iter()
            .map(Connection::terminal)
            .collect();

        let body = self.apply(arguments)?;

        let environment = InputEnvironment::new();

        body.output_with_inputs(&environment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Value;

    #[test]
    fn lbind_fixes_first_function_input() {
        let x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let x_connection = x.connection();
        let y_connection = y.connection();

        let add = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("add(Integer, Integer) must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(x_connection),
                Box::new(y_connection),
            ],
        )
        .expect("add(X, Y) must be valid");

        let function = Function::new(
            vec![x, y],
            body,
        );

        let two = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let bound = function
            .lbind(two)
            .expect("lbind must succeed");

        assert_eq!(bound.inputs.len(), 1);
        assert_eq!(bound.inputs[0].name, "Y");
        assert_eq!(bound.inputs[0].ty, Type::Integer);

        assert_eq!(
            bound.body.expression(),
            "add(TWO, Y)"
        );
    }

    #[test]
    fn function_can_be_applied_symbolically() {
        let x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("add(Integer, Integer) must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(x.connection()),
                Box::new(y.connection()),
            ],
        )
        .expect("add(X, Y) must be valid");

        let function = Function::new(
            vec![x, y],
            body,
        );

        let two = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let three = Connection::named_terminal(
            "THREE",
            Value::Integer(3),
        );

        let applied = function
            .apply(vec![two, three])
            .expect("application must succeed");

        assert_eq!(
            applied.expression(),
            "add(TWO, THREE)"
        );
    }

    #[test]
    fn function_can_be_applied_to_values() {
        let x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("add(Integer, Integer) must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(x.connection()),
                Box::new(y.connection()),
            ],
        )
        .expect("add(X, Y) must be valid");

        let function = Function::new(
            vec![x, y],
            body,
        );

        let result = function
            .apply_values(vec![
                Value::Integer(2),
                Value::Integer(3),
            ])
            .expect("application must succeed");

        match result {
            Value::Integer(value) => assert_eq!(value, 5),
            other => panic!("expected Integer(5), got {:?}", other),
        }
    }

    #[test]
    fn lbind_function_can_be_applied() {
        let x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = crate::registry::find_by_name_and_inputs(
            "add",
            &[Type::Integer, Type::Integer],
        )
        .expect("add(Integer, Integer) must exist");

        let body = Connection::new(
            add,
            vec![
                Box::new(x.connection()),
                Box::new(y.connection()),
            ],
        )
        .expect("add(X, Y) must be valid");

        let function = Function::new(
            vec![x, y],
            body,
        );

        let two = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let bound = function
            .lbind(two)
            .expect("lbind must succeed");

        let result = bound
            .apply_values(vec![Value::Integer(3)])
            .expect("bound function must apply");

        match result {
            Value::Integer(value) => assert_eq!(value, 5),
            other => panic!("expected Integer(5), got {:?}", other),
        }
    }

    #[test]
    fn lbind_function_type_matches_registry() {
        let input_x = InputSpec {
            name: "X".to_string(),
            ty: Type::Integer,
        };

        let input_y = InputSpec {
            name: "Y".to_string(),
            ty: Type::Integer,
        };

        let add = crate::registry::find_by_name_and_inputs(
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

        let function = Function::new(
            vec![input_x, input_y],
            body,
        );

        let fixed = Connection::named_terminal(
            "TWO",
            Value::Integer(2),
        );

        let bound = function
            .lbind(fixed)
            .expect("lbind must succeed");

        assert_eq!(
            bound.inputs.len(),
            1
        );

        assert_eq!(
            bound.inputs[0].ty,
            Type::Integer
        );

        assert_eq!(
            bound.body_type(),
            Type::Integer
        );
    }
}
