#![warn(clippy::pedantic)]

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
/// Hacky unicode slicing
pub fn slice(s: &str, a: usize, b: i64) -> String {
    let last = if b > 0 {
        (b as usize) - a
    } else {
        s.chars().count() - (b.abs() as usize)
    };
    s.chars().skip(a).take(last).collect()
}