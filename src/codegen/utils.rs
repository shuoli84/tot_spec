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
