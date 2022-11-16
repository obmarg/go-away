pub fn numbered(data: String) -> String {
    data.lines()
        .enumerate()
        .map(|(i, line)| format!("{i:>5} | {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
