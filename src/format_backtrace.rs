use backtrace::BacktraceSymbol;
use regex::regex;
use yansi::Paint;

fn fixup_simple_closure(name: &str) -> Option<String> {
    let re = regex!(r"^(\w+)\[[a-z0-9]{16}\]((?:::(?:\w+|\{closure#[0-9]+\}))+)");
    let captures = re.captures(name);
    if let Some(captures) = captures {
        return Some(format!("{}{}", &captures[1], &captures[2]).to_string());
    }

    None
}

fn fixup_simple_func(name: &str) -> Option<String> {
    let re = regex!(r"^(\w+)\[[a-z0-9]{16}\]::((?:\w|::)+)$");
    let captures = re.captures(name);
    if let Some(captures) = captures {
        let module = &captures[1];
        let name = &captures[2];
        return Some(format!("{}::{}", module, name).to_string());
    }

    None
}

fn fixup_ugly_rust_name(symbol: &BacktraceSymbol) -> (String, String) {
    match symbol.name() {
        Some(name) => {
            let name = name.to_string();
            let file = symbol
                .filename()
                .and_then(|f| f.to_str())
                .unwrap_or("???")
                .to_string();
            let line = symbol
                .lineno()
                .map(|l| l.to_string())
                .unwrap_or(String::from("???"));

            if let Some(name) = fixup_simple_func(&name) {
                return (
                    name.to_string(),
                    format!("{}  [{}:{}]", name.bold(), file, line,).to_string(),
                );
            }

            if let Some(name) = fixup_simple_closure(&name) {
                return (
                    name.to_string(),
                    format!("{}  [{}:{}]", name.bold(), file, line,).to_string(),
                );
            }
            let simple_re = regex!(r"^\w+$");
            let n = if simple_re.is_match(&name) {
                name.clone()
            } else {
                String::from("???")
            };

            (
                name,
                format!("{} [{}:{}]", n.bold(), file, line,).to_string(),
            )
        }
        None => (String::from("Unknown"), String::from("Unknown")),
    }
}

pub fn format_backtrace(start: &str) -> String {
    let bt = backtrace::Backtrace::new();
    let mut got_method = false;
    let mut got_last = false;
    let stack_trace: Vec<String> = bt
        .frames()
        .iter()
        .filter_map(|f| {
            if got_last {
                return None;
            }
            let info = f.symbols().first().map(fixup_ugly_rust_name);
            if let Some(ref info) = info
                && info
                    .0
                    .contains("cesty::test_runner::cesty_run_test_internal")
            {
                got_last = true;
                return None;
            }
            if got_method {
                return info.map(|i| i.1);
            }

            if let Some(info) = info.clone()
                && info.0 == start
            {
                got_method = true;
            }
            None
        })
        .collect();

    stack_trace.join("\n  ")
}
