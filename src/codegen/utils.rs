/// indent content with level
pub fn indent(content: impl AsRef<str>, level: usize) -> String {
    let prefix = "    ".repeat(level);
    content
        .as_ref()
        .split("\n")
        .into_iter()
        .map(|l| format!("{prefix}{}", l))
        .collect::<Vec<_>>()
        .join("\n")
}

/// prepend prefix for each line of content
/// e.g: prepend /// for each line of comment
pub fn multiline_prefix_with(content: &str, prefix: &str) -> String {
    content
        .trim()
        .split("\n")
        .into_iter()
        .map(|l| format!("{prefix}{}", l))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ensure at least count empty lines
#[allow(unused)]
pub fn ensure_emtpy_lines(val: &mut String, count: usize) {
    use std::fmt::Write;

    if count == 0 {
        return;
    }

    let mut empty_line_num = 0;

    let mut last_char: Option<char> = None;
    for c in val.chars().rev() {
        match c {
            '\n' => {
                if let Some(last_char) = last_char {
                    if last_char == '\n' {
                        empty_line_num += 1;
                    }
                }
                last_char = Some('\n');
            }
            _ => last_char = Some(c),
        }
    }

    // special case for val's first char is \n
    if let Some('\n') = last_char {
        empty_line_num += 1;
    }

    if count > empty_line_num {
        for _ in 0..count - empty_line_num {
            writeln!(val).expect("write to string only fail oom");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_empty_line_break() {
        fn test_it(input: &str, count: usize, expect: &str) {
            let mut val = input.to_string();
            ensure_emtpy_lines(&mut val, count);
            assert_eq!(val, expect.to_string());
        }

        for (input, count, expect) in &[
            ("", 0, ""),
            ("", 1, "\n"),
            ("\n", 1, "\n"),
            ("\n", 2, "\n\n"),
            // 2 empty lines => 3 \n
            ("foo\n", 2, "foo\n\n\n"),
        ] {
            test_it(input, *count, expect);
        }
    }
}
