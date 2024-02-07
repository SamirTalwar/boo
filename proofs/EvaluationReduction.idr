import Expression

data EvaluationError =
    EvaluationErrorUnknownIdentifier Identifier
  | EvaluationErrorInvalidFunctionApplication Expression
  | EvaluationErrorMatchWithoutBaseCase
  | EvaluationErrorNative NativeError

Show EvaluationError where
  show (EvaluationErrorUnknownIdentifier identifier) = "Evaluation error: unknown identifier: " ++ show identifier
  show (EvaluationErrorInvalidFunctionApplication expression) = "Evaluation error: invalid function application: " ++ show expression
  show EvaluationErrorMatchWithoutBaseCase = "Evaluation error: match without base case"
  show (EvaluationErrorNative error) = "Evaluation error: " ++ show error

data Evaluated : Type where
  EvPrimitive : Primitive -> Evaluated
  EvFunction : Identifier -> Expression -> Evaluated
  EvFailure : EvaluationError -> Evaluated

Show Evaluated where
  show (EvPrimitive x) = show x
  show (EvFunction parameter body) = show parameter ++ " -> (" ++ show body ++ ")"
  show (EvFailure error) = show error

data Progress : Type where
  Next : Expression -> Progress
  Complete : Evaluated -> Progress

Show Progress where
  show (Next x) = show x
  show (Complete x) = show x

mutual
  evaluate : Expression -> Evaluated
  evaluate expr =
    case step expr of
      Next next => evaluate next
      Complete evaluated => evaluated

  step : Expression -> Progress
  step (EPrimitive primitive) = Complete $ EvPrimitive primitive
  step (EIdentifier identifier) = Complete $ EvFailure $ EvaluationErrorUnknownIdentifier identifier
  step (ENative (MkNative _ impl)) =
    case impl (MkNativeContext $ Left . NativeErrorUnknownIdentifier) of
      Right primitive => Complete $ EvPrimitive primitive
      Left (NativeErrorUnknownIdentifier identifier) => Complete $ EvFailure $ EvaluationErrorUnknownIdentifier identifier
      Left error => Complete $ EvFailure $ EvaluationErrorNative error
  step (EFunction parameter body) = Complete $ EvFunction parameter body
  step (EApply function argument) =
   case step function of
      Next functionNext => Next (EApply functionNext argument)
      Complete (EvPrimitive primitive) => Complete $ EvFailure $ EvaluationErrorInvalidFunctionApplication (EPrimitive primitive)
      Complete (EvFunction parameter body) => Next $ substitute parameter argument body
      Complete (EvFailure error) => Complete $ EvFailure error
  step (EAssign name value inner) = Next $ substitute name value inner
  step (EMatch value patterns@[]) = Complete $ EvFailure EvaluationErrorMatchWithoutBaseCase
  step (EMatch value patterns@((pattern, result) :: restOfPatterns)) =
    case pattern of
      PatternAnything => Next result
      PatternPrimitive expected =>
        case step value of
          Next valueNext => Next $ EMatch valueNext patterns
          Complete (EvPrimitive actual) =>
            if actual == expected 
              then Next result
              else Next $ EMatch (EPrimitive actual) restOfPatterns
          Complete (EvFunction parameter body) => Next $ EMatch (EFunction parameter body) restOfPatterns
          Complete (EvFailure error) => Complete $ EvFailure error

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

steps : Expression -> List Progress
steps expr =
  Next expr :: case step expr of
    Next next => steps next
    result => [result]
