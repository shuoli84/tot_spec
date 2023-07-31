use indexmap::IndexMap;

/// The instruction supported
#[derive(Debug)]
pub enum Instruction {
    /// Declare and Store to local variable
    Store {
        name: String,
        expression: Expression,
    },
    /// Call user provided functions
    CallExternal {
        name: String,
        request: Expression,
        /// local name to bind
        response: String,
    },
    Convert,
    /// Return name
    Return(String),
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Load {
        /// The name to load
        name: String,
    },
    /// Object
    Object {
        fields: IndexMap<String, Expression>,
    },
}

#[derive(Debug)]
pub enum Literal {
    Int(String),
    String(String),
    Bool(String),
}
