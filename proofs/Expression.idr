import Identifier
import Native
import Primitive

data Pattern = PatternAnything | PatternPrimitive Primitive

Show Pattern where
  show PatternAnything = "_"
  show (PatternPrimitive x) = show x

data Expression : Type where
  EPrimitive : Primitive -> Expression
  EIdentifier : Identifier -> Expression
  ENative : Native -> Expression
  EFunction : Identifier -> Expression -> Expression
  EApply : Expression -> Expression -> Expression
  EAssign : Identifier -> Expression -> Expression -> Expression
  EMatch : Expression -> List (Pattern, Expression) -> Expression

Show Expression where
  show (EPrimitive x) = show x
  show (EIdentifier x) = show x
  show (ENative (MkNative name _)) = name
  show (EFunction param body) = "fn " ++ show param ++ " -> (" ++ show body ++ ")"
  show (EApply func arg) = "(" ++ show func ++ ") (" ++ show arg ++ ")"
  show (EAssign name value inner) = "let " ++ show name ++ " = " ++ show value ++ " in " ++ show inner
  show (EMatch value patterns) = "match " ++ show value ++ " { " ++ showPatterns patterns ++ "}"
    where
      showPattern : (Pattern, Expression) -> String
      showPattern (pattern, body) = show pattern ++ " -> " ++ show body
      showPatterns : List (Pattern, Expression) -> String
      showPatterns [] = ""
      showPatterns [x] = showPattern x
      showPatterns (x :: rest) = showPattern x ++ "; " ++ showPatterns rest

bint : Integer -> Expression
bint = EPrimitive . PrimitiveInteger

bid : String -> Expression
bid = EIdentifier . Id

bfn : String -> Expression -> Expression
bfn param body = EFunction (Id param) body

infixl 7 $$
($$) : Expression -> Expression -> Expression
($$) = EApply

blet : String -> Expression -> Expression -> Expression
blet name value inner = EAssign (Id name) value inner

FromString Expression where
  fromString = EIdentifier . Id
