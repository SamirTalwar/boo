#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum Integer {
    Small(i64),
}

type Small = i64;

impl From<i32> for Integer {
    fn from(value: i32) -> Self {
        Integer::Small(value as Small)
    }
}

impl From<i64> for Integer {
    fn from(value: i64) -> Self {
        Integer::Small(value)
    }
}

impl std::str::FromStr for Integer {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Small::from_str(s).map(Integer::Small)
    }
}

impl std::fmt::Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Integer::Small(value) => write!(f, "{}", value),
        }
    }
}

impl std::ops::Add for &Integer {
    type Output = Integer;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Small(l), Integer::Small(r)) => Integer::Small(l + r),
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
            (Integer::Small(l), Integer::Small(r)) => Integer::Small(l - r),
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
            (Integer::Small(l), Integer::Small(r)) => Integer::Small(l * r),
        }
    }
}

impl std::ops::Mul for Integer {
    type Output = Integer;

    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        arbtest::builder().run(|u| {
            let left = u.arbitrary::<i16>()? as Small;
            let right = u.arbitrary::<i16>()? as Small;
            assert_eq!(
                Integer::Small(left) + Integer::Small(right),
                Integer::Small(left + right)
            );
            Ok(())
        });
    }

    #[test]
    fn test_subtraction() {
        arbtest::builder().run(|u| {
            let left = u.arbitrary::<i16>()? as Small;
            let right = u.arbitrary::<i16>()? as Small;
            assert_eq!(
                Integer::Small(left) - Integer::Small(right),
                Integer::Small(left - right)
            );
            Ok(())
        });
    }

    #[test]
    fn test_multiplication() {
        arbtest::builder().run(|u| {
            let left = u.arbitrary::<i16>()? as Small;
            let right = u.arbitrary::<i16>()? as Small;
            assert_eq!(
                Integer::Small(left) * Integer::Small(right),
                Integer::Small(left * right)
            );
            Ok(())
        });
    }
}
