#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Integer,
    Function {
        parameter: Box<Type>,
        body: Box<Type>,
    },
}
