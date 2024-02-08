data Identifier = Id String

Eq Identifier where
  Id left == Id right = left == right

Show Identifier where
  show (Id x) = x

FromString Identifier where
  fromString = Id
