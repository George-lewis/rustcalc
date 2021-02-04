pub fn slice(s: &str, a: usize, b: usize) -> String {
    s.chars().skip(a).take(b - a).collect()
}