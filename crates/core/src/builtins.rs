use std::sync::Arc;

use crate::ast::*;
use crate::identifier::Identifier;
use crate::native::Native;

pub fn prepare<E: ExpressionWrapper>(expr: E) -> E {
    let mut result = expr;
    for (name, builtin) in all().into_iter().rev() {
        result = E::new_unannotated(Expression::Assign(Assign {
            name,
            value: builtin,
            inner: result,
        }));
    }
    result
}

pub fn all<E: ExpressionWrapper>() -> Vec<(Identifier, E)> {
    vec![(Identifier::name_from_str("trace").unwrap(), builtin_trace())]
}

fn builtin_trace<E: ExpressionWrapper>() -> E {
    let parameter = Identifier::name_from_str("param").unwrap();
    E::new_unannotated(Expression::Function(Function {
        parameter: parameter.clone(),
        body: E::new_unannotated(Expression::Native(Native {
            unique_name: Identifier::name_from_str("trace").unwrap(),
            implementation: Arc::new(move |context| {
                let value = context.lookup_value(&parameter)?;
                eprintln!("trace: {}", value);
                Ok(value)
            }),
        })),
    }))
}
