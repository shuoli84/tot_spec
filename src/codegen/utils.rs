/// indent content with level
pub fn indent(content: &str, level: usize) -> String {
    let prefix = "    ".repeat(level);
    content
        .split("\n")
        .into_iter()
        .map(|l| format!("{prefix}{}", l))
        .collect::<Vec<_>>()
        .join("\n")
}
