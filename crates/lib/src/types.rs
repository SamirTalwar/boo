#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Integer,
    Function {
        parameter: Option<Box<Type>>,
        body: Option<Box<Type>>,
    },
}
