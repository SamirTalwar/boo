import Decidable.Decidable

import Expression
import Identifier
import Native
import Primitive

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

data Intermediate : Expression -> Type where
  IIdentifier : (identifier : Identifier) -> Intermediate (EIdentifier identifier)
  INative : (native : Native) -> Intermediate (ENative native)
  IApply : (function : Expression) -> (argument : Expression) -> Intermediate (EApply function argument)
  IAssign : (name : Identifier) -> (value : Expression) -> (body : Expression) -> Intermediate (EAssign name value body)
  IMatch : (value : Expression) -> (patterns : List (Pattern, Expression)) -> Intermediate (EMatch value patterns)

mutual
  evaluate : Expression -> Either EvaluationError Expression
  evaluate expr =
    case canProgress expr of
      Yes exprCanProgress =>
        case step exprCanProgress of
          Right next => evaluate next
          Left error => Left error
      No _ => Right expr

  canProgress : (expr : Expression) -> Dec (Intermediate expr)
  canProgress expr@(EPrimitive primitive) =
    No $ \case
      IIdentifier {} impossible
      INative {} impossible
      IApply {} impossible
      IAssign {} impossible
      IMatch {} impossible
  canProgress expr@(EIdentifier identifier) = Yes (IIdentifier identifier)
  canProgress expr@(ENative native) = Yes (INative native)
  canProgress expr@(EFunction parameter body) =
    No $ \case
      IIdentifier {} impossible
      INative {} impossible
      IApply {} impossible
      IAssign {} impossible
      IMatch {} impossible
  canProgress expr@(EApply function argument) = Yes (IApply function argument)
  canProgress expr@(EAssign name value inner) = Yes (IAssign name value inner)
  canProgress expr@(EMatch value patterns) = Yes (IMatch value patterns)

  step : {expr : Expression} -> Intermediate expr -> Either EvaluationError Expression
  step (IIdentifier identifier) = Left $ EvaluationErrorUnknownIdentifier identifier
  step (INative (MkNative _ impl)) =
    case impl (MkNativeContext $ Left . NativeErrorUnknownIdentifier) of
      Right primitive => Right $ EPrimitive primitive
      Left (NativeErrorUnknownIdentifier identifier) => Left $ EvaluationErrorUnknownIdentifier identifier
      Left error => Left $ EvaluationErrorNative error
  step (IApply function argument) =
    case canProgress function of
      Yes functionCanProgress => (\f => EApply f argument) <$> step functionCanProgress
      No functionCannotProgress =>
        case function of
          EPrimitive {} => Left $ EvaluationErrorInvalidFunctionApplication function
          EFunction parameter body => Right $ substitute parameter argument body
          EIdentifier {} => void $ functionCannotProgress (IIdentifier {})
          ENative {} => void $ functionCannotProgress (INative {})
          EApply {} => void $ functionCannotProgress (IApply {})
          EAssign {} => void $ functionCannotProgress (IAssign {})
          EMatch {} => void $ functionCannotProgress (IMatch {})
  step (IAssign name value inner) = Right $ substitute name value inner
  step (IMatch value patterns@[]) = Left EvaluationErrorMatchWithoutBaseCase
  step (IMatch value patterns@((pattern, result) :: restOfPatterns)) =
    case pattern of
      PatternAnything => Right result
      PatternPrimitive expected =>
        case canProgress value of
          Yes valueCanProgress => (\v => EMatch v patterns) <$> step valueCanProgress
          No _ =>
            case value of
              EPrimitive actual =>
                if actual == expected
                  then Right result
                  else Right $ EMatch (EPrimitive actual) restOfPatterns
              expr => Right $ EMatch expr restOfPatterns

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
                         Right (EPrimitive primitive) => Right primitive
                         Right _ => Left NativeErrorInvalidPrimitive
                         Left _ => Left NativeErrorUnknown
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

steps : Expression -> List (Either EvaluationError Expression)
steps expr =
  Right expr :: case canProgress expr of
    Yes exprCanProgress =>
      case step exprCanProgress of
        Right next => steps next
        Left error => [Left error]
    No _ => []
