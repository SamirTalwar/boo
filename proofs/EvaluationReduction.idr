import Expression

data EvaluationError =
    EvaluationErrorUnknownIdentifier Identifier
  | EvaluationErrorInvalidFunctionApplication Expression
  | EvaluationErrorNative NativeError
  | EvaluationErrorMatchWithoutBaseCase -- replace with proofs
  | EvaluationErrorNonTerminalExpression Expression -- replace with proofs

data Evaluated : Type where
  EvPrimitive : Primitive -> Evaluated
  EvFunction : Identifier -> Expression -> Evaluated
  EvFailure : EvaluationError -> Evaluated

data Progress : (a : Type) -> Type where
  Next : a -> Progress a
  Complete : a -> Progress a
  Failure : EvaluationError -> Progress a

mutual
  evaluate : Expression -> Evaluated
  evaluate expr =
    case step expr of
      Next next => evaluate next
      Complete (EPrimitive primitive) => EvPrimitive primitive
      Complete (EFunction parameter body) => EvFunction parameter body
      Complete expr => EvFailure $ EvaluationErrorNonTerminalExpression expr
      Failure message => EvFailure message

  step : Expression -> Progress Expression
  step expr@(EPrimitive {}) = Complete expr
  step (EIdentifier identifier) = Failure $ EvaluationErrorUnknownIdentifier identifier
  step (ENative (MkNative _ impl)) =
    case impl (MkNativeContext $ Left . NativeErrorUnknownIdentifier) of
      Right primitive => Complete $ EPrimitive primitive
      Left (NativeErrorUnknownIdentifier identifier) => Failure $ EvaluationErrorUnknownIdentifier identifier
      Left error => Failure $ EvaluationErrorNative error
  step expr@(EFunction {}) = Complete expr
  step (EApply function argument) =
   case step function of
      Next functionNext => Next (EApply functionNext argument)
      Complete (EFunction parameter body) => Next $ substitute parameter argument body
      Complete expr => Failure $ EvaluationErrorInvalidFunctionApplication expr
      Failure message => Failure message
  step (EAssign name value inner) = Next $ substitute name value inner
  step (EMatch value patterns@[]) = Failure EvaluationErrorMatchWithoutBaseCase
  step (EMatch value patterns@((pattern, result) :: restOfPatterns)) =
    case pattern of
      PatternAnything => Next result
      PatternPrimitive expected =>
        case step value of
          Next valueNext => Next $ EMatch valueNext patterns
          Complete valueComplete@(EPrimitive actual) =>
            if actual == expected 
              then Next result
              else Next $ EMatch valueComplete restOfPatterns
          Complete valueComplete => Next $ EMatch valueComplete restOfPatterns
          Failure message => Failure message

  substitute : Identifier -> Expression -> Expression -> Expression
  substitute identifier replacement expr = substitute' identifier replacement expr []
    where
      substitute' : Identifier -> Expression -> Expression -> List Identifier -> Expression
      substitute' identifier replacement expr bound =
        case expr of
          EPrimitive _ => expr
          EIdentifier i =>
            if identifier == i
              then avoidAlphaCapture replacement bound
              else expr
          ENative (MkNative nativeName impl) =>
            ENative $ MkNative nativeName $ \outerContext =>
              impl $ MkNativeContext $ \i =>
                if identifier == i
                  then case evaluate replacement of
                         EvPrimitive primitive => Right primitive
                         EvFunction {} => Left NativeErrorInvalidPrimitive
                         EvFailure message => Left NativeErrorUnknown
                  else lookupContext i outerContext
          EFunction parameter body =>
            if parameter == identifier
              then expr
              else EFunction parameter (substitute' identifier replacement body (parameter :: bound))
          EApply function argument =>
            EApply
              (substitute' identifier replacement function bound)
              (substitute' identifier replacement argument bound)
          EAssign name value inner =>
            EAssign
              name
              (substitute' identifier replacement value bound)
              (substitute' identifier replacement inner (name :: bound))
          EMatch value patterns =>
            EMatch
              (substitute' identifier replacement value bound)
              (map (\(pattern, result) => (pattern, substitute' identifier replacement result bound)) patterns)

  -- implement this
  avoidAlphaCapture : Expression -> List Identifier -> Expression
  avoidAlphaCapture expr _ = expr

steps : Expression -> List Expression
steps expr =
  expr :: case step expr of
    Next next => steps next
    Complete complete => [complete]
    Failure _ => []
