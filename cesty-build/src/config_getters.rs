use std::collections::VecDeque;

use miette::{Context, Result, bail};

pub fn string(
    config: &toml::map::Map<String, toml::Value>,
    test_name: &str,
    key: &'static str,
) -> Result<String> {
    let Some(value) = config.get(key) else {
        return Ok(String::new());
    };

    Ok(value
        .as_str()
        .wrap_err(format!(
            "Field '{}' in test '{}' must be a string, but found: {}",
            key,
            test_name,
            value.type_str()
        ))?
        .to_string())
}

pub fn string_pairs(
    config: &toml::map::Map<String, toml::Value>,
    test_name: &str,
    key: &'static str,
) -> Result<Vec<(String, String)>> {
    let Some(value) = config.get(key) else {
        return Ok(Vec::new());
    };

    let values = value.as_table().wrap_err(format!(
        "Field '{}' in test '{}' must be a table (map) of key-value pairs, but found: {}",
        key,
        test_name,
        value.type_str()
    ))?;

    fn _cleanup(mut s: String) -> String {
        if s.starts_with('"') && s.ends_with('"') {
            s.remove(0);
            s.pop();
        }
        s
    }
    Ok(values
        .iter()
        .map(|value| (_cleanup(value.0.to_string()), _cleanup(value.1.to_string())))
        .collect())
}

pub fn string_array(
    config: &toml::map::Map<String, toml::Value>,
    test_name: &str,
    key: &'static str,
    required: bool,
) -> Result<VecDeque<String>> {
    let Some(value) = config.get(key) else {
        if required {
            bail!(
                "Required field '{}' is missing from test configuration '{}'. \
                 This field should contain a list of source file paths to compile for this test. \
                 Example: {} = [\"src/test.c\"]",
                key,
                test_name,
                key
            );
        }

        return Ok(VecDeque::new());
    };
    if let Some(value) = value.as_str() {
        return Ok(VecDeque::from(vec![value.to_string()]));
    }

    let values = value.as_array().wrap_err(format!(
        "Field '{}' in test '{}' must be either a string or array of strings, but found: {}",
        key,
        test_name,
        value.type_str()
    ))?;

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .wrap_err(format!(
                    "All values in '{}::{}' must be strings, but found a {}: {}",
                    test_name,
                    key,
                    value.type_str(),
                    value
                ))
                .map(|v| v.to_string())
        })
        .collect()
}
