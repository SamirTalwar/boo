data Primitive = PrimitiveInteger Integer

Eq Primitive where
  PrimitiveInteger left == PrimitiveInteger right = left == right

Show Primitive where
  show (PrimitiveInteger x) = show x

data Identifier = Id String

Eq Identifier where
  Id left == Id right = left == right

Show Identifier where
  show (Id x) = x

data NativeError =
    NativeErrorUnknownIdentifier Identifier
  | NativeErrorInvalidPrimitive
  | NativeErrorUnknown

data NativeContext = MkNativeContext (Identifier -> Either NativeError Primitive)

lookupContext : Identifier -> NativeContext -> Either NativeError Primitive
lookupContext identifier (MkNativeContext context) = context identifier

data Native = MkNative String (NativeContext -> Either NativeError Primitive)

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

FromString Identifier where
  fromString = Id

FromString Expression where
  fromString = EIdentifier . Id
