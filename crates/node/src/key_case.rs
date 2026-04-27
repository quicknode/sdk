// Case conversion helpers shared by napi object rewrites. Used when a
// binding surface accepts/returns `serde_json::Value` (because napi cannot
// represent a flattened enum), so per-field case conversion that napi would
// normally do for `#[napi(object)]` structs has to be done manually.

pub fn camel_to_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

pub fn snake_to_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper_next = false;
    for c in s.chars() {
        if c == '_' {
            upper_next = true;
        } else if upper_next {
            out.push(c.to_ascii_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

pub fn convert_keys<F: Fn(&str) -> String + Copy>(v: serde_json::Value, f: F) -> serde_json::Value {
    match v {
        serde_json::Value::Object(o) => serde_json::Value::Object(
            o.into_iter()
                .map(|(k, v)| (f(&k), convert_keys(v, f)))
                .collect(),
        ),
        serde_json::Value::Array(a) => {
            serde_json::Value::Array(a.into_iter().map(|v| convert_keys(v, f)).collect())
        }
        other => other,
    }
}
