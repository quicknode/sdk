use reqwest::get;
use serde::Deserialize;

#[derive(Deserialize)]
struct UuidResponse {
    uuid: String,
}
// Simple function: add two numbers
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// Function that can fail: divide two numbers
pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("Can't divide by zero".to_string());
    }
    Ok(a / b)
}

/// Gets a uuid from httpbin
pub async fn get_external_uuid() -> Result<String, Box<dyn std::error::Error>> {
    let uuid: UuidResponse = get("https://httpbin.org/uuid").await?.json().await?;

    Ok(uuid.uuid)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(10.0, 2.0).unwrap(), 5.0);
        assert!(divide(1.0, 0.0).is_err());
    }
}
