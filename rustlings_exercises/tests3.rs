// tests3.rs
//
// This test isn't testing our function -- make it do that in such a way that
// the test passes. Then write a second test that tests whether we get the
// result we expect to get when we call `is_even(5)`.

pub fn is_even(num: i32) -> bool {
    num % 2 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_true_when_even() {
        assert!(is_even(4));
    }

    #[test]
    fn is_false_when_odd() {
        assert!(!is_even(3));
    }
}









// Further information:
// Writing tests : https://doc.rust-lang.org/book/ch11-01-writing-tests.html





// Further information:
// Writing tests : https://doc.rust-lang.org/book/ch11-01-writing-tests.html