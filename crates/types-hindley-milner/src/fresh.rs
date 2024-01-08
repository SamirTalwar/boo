use std::iter;

use boo_core::types::TypeVariable;

pub struct FreshVariables {
    values: Box<dyn Iterator<Item = TypeVariable>>,
}

impl FreshVariables {
    pub fn new() -> Self {
        Self {
            values: Box::new(
                iter::successors(Some(0), |x| Some(x + 1))
                    .map(|x| TypeVariable::new(format!("_{x}"))),
            ),
        }
    }

    pub fn next(&mut self) -> TypeVariable {
        self.values.next().unwrap()
    }
}
