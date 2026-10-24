//! Small RFC-4180 CSV reader (quotes, escaped quotes, embedded newlines, BOM,
//! comma / semicolon / tab auto-detected) plus header auto-detection.

pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn col(&self, wanted: Option<&str>, candidates: &[&str]) -> Option<usize> {
        let norm = |s: &str| s.trim().trim_start_matches(':').to_lowercase().replace([' ', '-'], "_");
        if let Some(w) = wanted {
            let w = norm(w);
            return self.header.iter().position(|h| norm(h) == w);
        }
        for c in candidates {
            let c = norm(c);
            if let Some(i) = self.header.iter().position(|h| norm(h) == c) {
                return Some(i);
            }
        }
        None
    }
}

fn detect_delim(first_line: &str) -> char {
    let count = |d: char| {
        let mut in_q = false;
        first_line.chars().filter(|&c| {
            if c == '"' {
                in_q = !in_q;
            }
            !in_q && c == d
        }).count()
    };
    [',', ';', '\t'].into_iter().max_by_key(|&d| count(d)).unwrap_or(',')
}

pub fn parse(text: &str) -> Result<Table, String> {
    let text = text.trim_start_matches('\u{feff}');
    let first = text.lines().next().ok_or("empty file")?;
    let d = detect_delim(first);
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_q = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if in_q {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_q = false;
                }
            } else {
                field.push(c);
            }
        } else if c == '"' && field.is_empty() {
            in_q = true;
        } else if c == d {
            row.push(std::mem::take(&mut field));
        } else if c == '\n' || c == '\r' {
            if c == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            row.push(std::mem::take(&mut field));
            if !(row.len() == 1 && row[0].is_empty()) {
                rows.push(std::mem::take(&mut row));
            } else {
                row.clear();
            }
        } else {
            field.push(c);
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    if rows.is_empty() {
        return Err("no rows".into());
    }
    let header = rows.remove(0).into_iter().map(|h| h.trim().to_string()).collect();
    Ok(Table { header, rows })
}

/// Split a multi-valued provenance cell: "a;b", "a|b", "['a', 'b']", "[\"a\",\"b\"]".
pub fn split_multi(cell: &str) -> Vec<String> {
    let t = cell.trim().trim_start_matches('[').trim_end_matches(']');
    let seps: &[char] = if t.contains(';') || t.contains('|') { &[';', '|'] } else { &[','] };
    t.split(seps)
        .map(|s| s.trim().trim_matches(|c| c == '\'' || c == '"').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quotes_and_newlines() {
        let t = parse("\u{feff}a,b,c\n1,\"x, \"\"y\"\"\nz\",3\r\n\n4,5,6\n").unwrap();
        assert_eq!(t.header, vec!["a", "b", "c"]);
        assert_eq!(t.rows[0][1], "x, \"y\"\nz");
        assert_eq!(t.rows.len(), 2);
    }
    #[test]
    fn semicolon_and_detect() {
        let t = parse("source;target;:TYPE\nA;B;rel\n").unwrap();
        assert_eq!(t.col(None, &["type"]), Some(2));
        assert_eq!(t.col(None, &["target"]), Some(1));
    }
    #[test]
    fn multi() {
        assert_eq!(split_multi("['d1', 'd2']"), vec!["d1", "d2"]);
        assert_eq!(split_multi("d1;d2|d3"), vec!["d1", "d2", "d3"]);
        assert_eq!(split_multi("doc-7"), vec!["doc-7"]);
    }
}
