//! Utility module containing math functions.

/// Find the u128 square root of `input` (using binary search) rounding down.
///
/// ### Parameters:
///
/// * `input`: [`u128`] - The number to find the square root of.
///
/// ### Returns:
/// The largest x, such that x*x is <= input of type [`u128`]
pub fn u128_sqrt(input: u128) -> u64 {
    // Search between 0 and 2 << 64 as this is the feasible output space.
    let mut low: u128 = u128::MIN;
    let mut high: u128 = 2 << 64;

    // Binary search (round down)
    while low != high - 1 {
        // Determine middle between high and low.
        // Cannot overflow as: low < high <= MAX/2  =>  low + high < 2*high <= MAX
        let middle = (low + high) / 2;

        match middle.checked_mul(middle) {
            Some(middle_squared) if middle_squared <= input => {
                low = middle; // Keep searching in right side
            }
            _ => {
                high = middle; // Keep searching in left side
            }
        }
    }
    low.try_into().unwrap()
}

/// Divides two [`u128`] types and rounds up.
///
/// ### Parameters:
///
/// * `numerator`: The numerator for the division.
///
/// * `denominator`: The denominator for the division.
///
/// ### Returns:
///
/// The result of the division, rounded up, of type [`u128`].
pub fn u128_division_ceil(numerator: u128, denominator: u128) -> Result<u128, &'static str> {
    let div_floor = numerator
        .checked_div(denominator)
        .ok_or("Division by zero")?;
    let rem = numerator
        .checked_rem(denominator)
        .ok_or("Division by zero")?;
    Ok(div_floor + u128::from(rem != 0))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    pub fn manual_u128_test() {
        assert_eq!(u128_sqrt(25), 5);
        assert_eq!(u128_sqrt(20), 4);
        assert_eq!(u128_sqrt(0), 0);
        assert_eq!(u128_sqrt(1), 1);
        assert_eq!(u128_sqrt(316227 * 316227), 316227);
        assert_eq!(
            u128_sqrt(100_000_000_000 * 100_000_000_000),
            100_000_000_000
        );
    }

    fn generic_u128_sqrt_identity(output: u64) {
        let input = output as u128;
        assert_eq!(output, u128_sqrt(input * input));
    }

    fn generic_u128_sqrt_stable(input: u128) {
        let sqrt = u128_sqrt(input);
        let sqrt_squared: u128 = u128::from(sqrt) * u128::from(sqrt);
        assert_eq!(sqrt, u128_sqrt(sqrt_squared));
        assert!(sqrt_squared <= input);

        // u64::MAX ^ 2 is out of range.
        if sqrt < u64::MAX {
            let sqrt_plus_1_squared: u128 = u128::from(sqrt + 1) * u128::from(sqrt + 1);
            assert!(input <= sqrt_plus_1_squared);
        }
    }

    #[test]
    fn seen_proptest_failures() {
        u128_sqrt(36893488147419103231);
        generic_u128_sqrt_identity(6074001000);
        generic_u128_sqrt_stable(36893488147419103231);

        u128_sqrt(u128::MIN);
        assert_eq!(u128_sqrt(u128::MAX), u64::MAX);
        generic_u128_sqrt_identity(u64::MIN);
        generic_u128_sqrt_stable(u128::MIN);
        generic_u128_sqrt_identity(u64::MAX - 1);
        generic_u128_sqrt_stable(u128::MAX - 1);
        generic_u128_sqrt_identity(u64::MAX);
        generic_u128_sqrt_stable(u128::MAX);
    }

    proptest! {
        #[test]
        fn u128_sqrt_identity(i in any::<u64>()) {
            generic_u128_sqrt_identity(i);
        }
    }

    proptest! {
        #[test]
        fn u128_sqrt_stable(i in any::<u128>()) {
            generic_u128_sqrt_stable(i);
        }
    }

    proptest! {
        #[test]
        fn u128_sqrt_must_not_crash(i in any::<u128>()) {
            u128_sqrt(i);
        }
    }

    #[test]
    pub fn test_u128_division_ceil() {
        // Division by 0 cases is guarded against by u128 type and the source code

        let div1 = u128_division_ceil(10, 2);
        let div2 = u128_division_ceil(999, 66);
        let div3 = u128_division_ceil(15, 4);

        assert_eq!(div1, Ok(5));
        assert_eq!(div2, Ok(16));
        assert_eq!(div3, Ok(4));
        assert_eq!(u128_division_ceil(15, 0), Err("Division by zero"));
    }

    #[test]
    pub fn test_u128_division_ceil_2() {
        let a: u128 = 0xDEADBEEF;
        let b: u128 = 0xC0FFEE;
        let k: u128 = a * b;
        assert_eq!(u128_division_ceil(k, a), Ok(b));
        assert_eq!(u128_division_ceil(k, b), Ok(a));
    }
}
