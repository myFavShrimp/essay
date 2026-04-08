async fn fetch_data(key: &str) -> String {
    format!("data_{}", key)
}

#[essay::essay(cases(
    "fetch_users" => (fetch_data("users").await),
    "fetch_orders" => (fetch_data("orders").await),
), test_attr = tokio::test)]
async fn process(data: String) -> Result<String, String> {
    Ok(data.to_uppercase())
}

fn check_length(result: Result<Vec<String>, String>) {
    assert_eq!(result.unwrap().len(), 3);
}

#[essay::essay(cases(
    "split_csv" => ("a,b,c", ",") -> check_length,
    "split_pipes" => ("x|y|z", "|") -> check_length,
), test_attr = tokio::test)]
async fn split_string(input: &str, delimiter: &str) -> Result<Vec<String>, String> {
    Ok(input.split(delimiter).map(String::from).collect())
}

#[essay::essay(cases(
    "sync_fn_under_tokio" => ("hello"),
), test_attr = tokio::test)]
fn sync_under_tokio(input: &str) -> Result<usize, String> {
    Ok(input.len())
}
