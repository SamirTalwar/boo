use num_bigint::BigInt;
use proptest::strategy::Strategy;

type Small = i32;
type Large = BigInt;

#[derive(Debug, Clone)]
pub enum Integer {
    Small(Small),
    Large(Large),
}

impl From<i16> for Integer {
    fn from(value: i16) -> Self {
        Integer::Small(value as i32)
    }
}

impl From<i32> for Integer {
    fn from(value: i32) -> Self {
        Integer::Small(value)
    }
}

impl From<i64> for Integer {
    fn from(value: i64) -> Self {
        match Small::try_from(value) {
            Ok(value) => Integer::Small(value),
            Err(_) => Integer::Large(value.into()),
        }
    }
}

impl From<i128> for Integer {
    fn from(value: i128) -> Self {
        match Small::try_from(value) {
            Ok(value) => Integer::Small(value),
            Err(_) => Integer::Large(value.into()),
        }
    }
}

impl std::str::FromStr for Integer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Small::from_str(s)
            .map(Integer::Small)
            .map_err(|_| ())
            .or_else(|_| Large::from_str(s).map(Integer::Large).map_err(|_| ()))
    }
}

impl std::fmt::Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Integer::Small(value) => value.fmt(f),
            Integer::Large(value) => value.fmt(f),
        }
    }
}

impl PartialEq for Integer {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Integer::Small(l), Integer::Small(r)) => l == r,
            (Integer::Small(l), Integer::Large(r)) => Large::from(*l) == *r,
            (Integer::Large(l), Integer::Small(r)) => *l == Large::from(*r),
            (Integer::Large(l), Integer::Large(r)) => l == r,
        }
    }
}

impl Eq for Integer {}

impl std::ops::Add for &Integer {
    type Output = Integer;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Small(l), Integer::Small(r)) => match (*l).checked_add(*r) {
                Some(result) => Integer::Small(result),
                None => Integer::Large(Large::from(*l) + Large::from(*r)),
            },
            (Integer::Small(l), Integer::Large(r)) => Integer::Large(Large::from(*l) + r),
            (Integer::Large(l), Integer::Small(r)) => Integer::Large(l + Large::from(*r)),
            (Integer::Large(l), Integer::Large(r)) => Integer::Large(l + r),
        }
    }
}

impl std::ops::Add for Integer {
    type Output = Integer;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl std::ops::Sub for &Integer {
    type Output = Integer;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Small(l), Integer::Small(r)) => match (*l).checked_sub(*r) {
                Some(result) => Integer::Small(result),
                None => Integer::Large(Large::from(*l) - Large::from(*r)),
            },
            (Integer::Small(l), Integer::Large(r)) => Integer::Large(Large::from(*l) - r),
            (Integer::Large(l), Integer::Small(r)) => Integer::Large(l - Large::from(*r)),
            (Integer::Large(l), Integer::Large(r)) => Integer::Large(l - r),
        }
    }
}

impl std::ops::Sub for Integer {
    type Output = Integer;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}

impl std::ops::Mul for &Integer {
    type Output = Integer;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Small(l), Integer::Small(r)) => match (*l).checked_mul(*r) {
                Some(result) => Integer::Small(result),
                None => Integer::Large(Large::from(*l) * Large::from(*r)),
            },
            (Integer::Small(l), Integer::Large(r)) => Integer::Large(Large::from(*l) * r),
            (Integer::Large(l), Integer::Small(r)) => Integer::Large(l * Large::from(*r)),
            (Integer::Large(l), Integer::Large(r)) => Integer::Large(l * r),
        }
    }
}

impl std::ops::Mul for Integer {
    type Output = Integer;

    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

impl Integer {
    pub fn arbitrary() -> impl Strategy<Value = Integer> {
        proptest::num::i128::ANY.prop_map(|n| n.into())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::proptest_helpers::*;

    use super::*;

    #[test]
    fn test_from_string() {
        check(&Integer::arbitrary(), |value| {
            let input = format!("{}", value);
            prop_assert_eq!(input.parse::<Integer>(), Ok(value));
            Ok(())
        })
    }

    #[test]
    fn test_addition() {
        check(&(any::<i128>(), any::<i128>()), |(left, right)| {
            prop_assert_eq!(
                Integer::from(left) + Integer::from(right),
                Integer::Large(Large::from(left) + Large::from(right))
            );
            Ok(())
        })
    }

    #[test]
    fn test_subtraction() {
        check(&(any::<i128>(), any::<i128>()), |(left, right)| {
            prop_assert_eq!(
                Integer::from(left) - Integer::from(right),
                Integer::Large(Large::from(left) - Large::from(right))
            );
            Ok(())
        })
    }

    #[test]
    fn test_multiplication() {
        check(&(any::<i128>(), any::<i128>()), |(left, right)| {
            prop_assert_eq!(
                Integer::from(left) * Integer::from(right),
                Integer::Large(Large::from(left) * Large::from(right))
            );
            Ok(())
        })
    }
}
