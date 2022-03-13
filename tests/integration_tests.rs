use simple_test_case::dir_cases;
use std::error::Error;
use std::result::Result;
use std::{env, process::Command};

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

fn extract_expected_data(_line_num: usize, line: &str) -> Option<String> {
    if let Some(cap) = regex!(r"// expect: ?(.*)").captures_iter(line).next() {
        let capture = &cap[1];
        return Some(capture.to_string());
    }

    if let Some(cap) = regex!(r"// (Error.*)").captures_iter(line).next() {
        let capture = &cap[1];
        return Some(format!("{capture}"));
    }

    if let Some(cap) = regex!(r"// \[((java|c) )?line (\d+)\] (Error.*)")
        .captures_iter(line)
        .next()
    {
        if let Some("c") = cap.get(2).map(|m| m.as_str()) {
            return None;
        }
        let capture = &cap[4];
        return Some(format!("{capture}"));
    }

    if let Some(cap) = regex!(r"// expect runtime error: (.+)")
        .captures_iter(line)
        .next()
    {
        let capture = &cap[1];
        return Some(format!("{capture}"));
    }

    if let Some(cap) = regex!(r"\[.*line (\d+)\] (Error.+)")
        .captures_iter(line)
        .next()
    {
        let capture = &cap[2];
        return Some(format!("{capture}"));
    }

    if let Some(cap) = regex!(r"(\[line \d+\])").captures_iter(line).next() {
        let capture = &cap[1];
        return Some(capture.to_string());
    }

    None
}

fn run_test(bin_path: &str, source_file: &str, source: &str) -> Result<(), Box<dyn Error>> {
    println!("{bin_path:?}");
    let mut expected = String::new();
    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        if let Some(line) = extract_expected_data(line_num, line) {
            dbg!(bin_path, source_file, &line);
            expected.push_str(&format!("{line}\n"));
        }
    }

    let output = Command::new(bin_path)
        .arg(&format!("{source_file}"))
        .output()?;

    let output = String::from_utf8(output.stdout)?;

    dbg!(&output, &expected);
    assert_eq!(output, expected);

    Ok(())
}

#[dir_cases(
    "data/assignment",
    "data/block",
    "data/bool",
    "data/call",
    "data/class",
    "data/closure",
    "data/comments",
    "data/constructor",
    "data/expressions",
    "data/field",
    "data/for",
    "data/nil",
    "data/operator",
    "data/print"
)]
#[test]
fn crafting_interpreters_test_suite(path: &str, contents: &str) -> Result<(), Box<dyn Error>> {
    if path.ends_with("decimal_point_at_eof.lox")
        || path.ends_with("equals_class.lox")
        || path.ends_with("equals_method.lox")
    {
        return Ok(());
    }

    let name = env::var("CARGO_PKG_NAME")?;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let bin_path = format!("{manifest_dir}/target/debug/{name}");
    run_test(&bin_path, path, contents)
}
