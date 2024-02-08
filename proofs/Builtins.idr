import Expression
import Identifier
import Native
import Primitive

operator : String -> (Integer -> Integer -> Integer) -> Expression
operator name op =
  bfn "left" $ bfn "right" $ ENative $ MkNative name $ \context => do
    PrimitiveInteger left <- lookupContext "left" context
    PrimitiveInteger right <- lookupContext "right" context
    pure . PrimitiveInteger $ op left right

add : Expression
add = operator "(left + right)" (+)

subtract : Expression
subtract = operator "(left - right)" (-)

multiply : Expression
multiply = operator "(left * right)" (*)
