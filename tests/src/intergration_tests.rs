use std::error::Error;
use std::result::Result;
use std::{env, process::Command};

use simple_test_case::dir_cases;

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

fn run_test(mut command: Command, source_file: &str, source: &str) -> Result<(), Box<dyn Error>> {
    let mut expected = String::new();
    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        if let Some(line) = extract_expected_data(line_num, line) {
            dbg!(source_file, &line);
            expected.push_str(&format!("{line}\n"));
        }
    }

    let output = command.arg(&format!("{source_file}")).output()?;

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
    "data/function",
    "data/if",
    "data/inheritance",
    "data/logical_operator",
    "data/method",
    "data/nil",
    "data/number",
    "data/operator",
    "data/print",
    "data/regression",
    "data/return",
    "data/string",
    "data/super",
    "data/this",
    "data/trivial",
    "data/variable",
    "data/while"
)]
#[test]
pub fn crafting_interpreters_test_suite(path: &str, contents: &str) -> Result<(), Box<dyn Error>> {
    dbg!(&path);
    let mut binary_path =
        env::current_exe().expect("need current binary path to find binary to test");
    loop {
        {
            let parent = binary_path.parent();
            if parent.is_none() {
                panic!(
                    "Failed to locate binary path from original path: {:?}",
                    env::current_exe()
                );
            }
            let parent = parent.unwrap();
            if parent.is_dir() && parent.file_name().unwrap() == "target" {
                break;
            }
        }
        binary_path.pop();
    }

    binary_path.push(if cfg!(target_os = "windows") {
        format!("interpreter_main.exe",)
    } else {
        "interpreter_main".into()
    });

    dbg!(&binary_path);
    let command = Command::new(binary_path);

    run_test(command, &format!("../{}", path), contents)
}
