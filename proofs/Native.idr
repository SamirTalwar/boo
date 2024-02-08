import Identifier
import Primitive

data NativeError =
    NativeErrorUnknownIdentifier Identifier
  | NativeErrorInvalidPrimitive
  | NativeErrorUnknown

Show NativeError where
  show (NativeErrorUnknownIdentifier identifier) = "Native error: unknown identifier: " ++ show identifier
  show NativeErrorInvalidPrimitive = "Native error: invalid primitive"
  show NativeErrorUnknown = "Native error: unknown"

data NativeContext = MkNativeContext (Identifier -> Either NativeError Primitive)

data Native = MkNative String (NativeContext -> Either NativeError Primitive)

lookupContext : Identifier -> NativeContext -> Either NativeError Primitive
lookupContext identifier (MkNativeContext context) = context identifier
