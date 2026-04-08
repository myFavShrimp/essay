fn check_ok_contains_hello(result: Result<String, String>) {
    assert!(result.unwrap().contains("hello"));
}

#[essay::essay(cases(
    "single_arg" => ("hello"),
    "empty_string" => (""),
))]
fn reverse_string(s: &str) -> Result<String, String> {
    Ok(s.chars().rev().collect())
}

#[essay::essay(cases(
    "two_words" => ("hello", "world"),
    "with_custom_assert" => ("hello", "there") -> check_ok_contains_hello,
))]
fn concat(a: &str, b: &str) -> Result<String, String> {
    Ok(format!("{} {}", a, b))
}

fn check_ok_even_length(result: Result<Vec<i32>, String>) {
    for n in result.unwrap() {
        assert_eq!(n % 2, 0);
    }
}

#[essay::essay(cases(
    "small_range" => (1, 10),
    "single_element" => (4, 4),
    "verify_even" => (1, 20) -> check_ok_even_length,
))]
fn even_numbers(from: i32, to: i32) -> Result<Vec<i32>, String> {
    Ok((from..=to).filter(|n| n % 2 == 0).collect())
}

fn check_error(result: Result<f64, String>) {
    assert!(result.is_err());
}

#[essay::essay(cases(
    "basic" => (10.0, 3.0),
    "divide_by_zero" => (1.0, 0.0) -> check_error,
))]
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("division by zero".into());
    }
    Ok(a / b)
}
