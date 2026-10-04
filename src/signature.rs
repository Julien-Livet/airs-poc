use crate::registry::Type;
use crate::connection::{Connection};

#[derive(Debug, Clone)]
pub struct InputSpec {
    pub name: String,
    pub ty: Type,
}

impl InputSpec {
    pub fn accepts(&self, ty: Type) -> bool {
        self.ty == ty
    }

    pub fn connection(&self) -> Connection {
        Connection::input(
            self.name.clone(),
            self.ty,
        )
    }
}
