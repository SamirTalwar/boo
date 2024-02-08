data Primitive = PrimitiveInteger Integer

Eq Primitive where
  PrimitiveInteger left == PrimitiveInteger right = left == right

Show Primitive where
  show (PrimitiveInteger x) = show x
