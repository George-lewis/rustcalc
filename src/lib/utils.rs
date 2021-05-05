#![warn(clippy::pedantic)]

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

pub enum Pos {
    Idx(usize),
    End,
}

/// Hacky unicode slicing
///
/// * `string` - The input string
/// * `start` - The start of the slice
/// * `end` - The end of the slice
///
/// **returns**: The slice as a new `String`
///
/// ## Panics
/// Panics when `end < start` or either argument is out of bounds
/// ```should_panic
/// // These all panic
/// # use rustmatheval::utils::{slice, Pos};
/// slice("abc", 1, &Pos::Idx(0));
/// slice("", 1, &Pos::End);
/// slice("", 1, &Pos::Idx(1));
/// ```
///
/// ## Examples
/// ```
/// # use rustmatheval::utils::{slice, Pos};
/// let slice_ = slice("abcdef", 1, &Pos::Idx(4));
/// assert_eq!(slice_, "bcd");
/// let slice_ = slice("abcdef", 1, &Pos::End);
/// assert_eq!(slice_, "bcdef");
/// ```
#[must_use]
pub fn slice(string: &str, start: usize, end: &Pos) -> String {
    let len = string.chars().count();

    let end = match end {
        // This will panic if `start > end`
        Pos::Idx(idx) => *idx,
        Pos::End => len,
    } - start;

    assert!(start + end <= len, "end ({}) > len ({})", start + end, len);

    string.chars().skip(start).take(end).collect()
}

#[macro_export]
macro_rules! same {
    ($a:expr, $b:expr) => {
        ($a - $b).abs() <= f64::EPSILON * $a.max($b).abs()
    };
}

#[macro_export]
macro_rules! assert_same {
    ($a:expr, $b:expr) => {
        assert!(same!($a, $b), "{} != {}", $a, $b)
    };
    ($a:expr, $b:expr, $msg:expr, $($args:expr),*) => {
        assert!(same!($a, $b), $msg, $($args),*)
    }
}

#[cfg(test)]
mod tests {

    #![allow(unused_must_use)]

    use super::{slice, Pos};

    #[test]
    fn test_ok() {
        let input = "abcdef123456";
        assert_eq!(slice(input, 0, &Pos::End), input);
        assert_eq!(slice(input, 0, &Pos::Idx(3)), "abc");
        assert_eq!(slice(input, 6, &Pos::End), "123456");

        assert_eq!(slice(input, 0, &Pos::Idx(0)), "");
        assert_eq!(slice(input, input.chars().count(), &Pos::End), "");
    }

    #[test]
    #[should_panic]
    fn test_end_before_start() {
        slice("123", 1, &Pos::Idx(0));
    }

    #[test]
    #[should_panic]
    fn test_start_out_of_bounds() {
        slice("", 5, &Pos::End);
    }

    #[test]
    #[should_panic]
    fn test_end_out_of_bounds() {
        slice("123", 0, &Pos::Idx(4));
    }
}
