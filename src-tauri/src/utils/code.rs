/// Lightweight, dependency-free code detection.
///
/// The clipboard watcher used to rely on the external `hyperpolyglot` crate
/// (disabled). This module replaces it with a small set of heuristics so
/// that code snippets are classified as `ContentType::Code` with a
/// best-effort language, which powers syntax highlighting in the preview.
///
/// The heuristics are deliberately conservative: normal prose and short
/// fragments must not be misclassified as code.
pub fn detect_code(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.len() < 8 {
        return None;
    }

    let lines: Vec<&str> = trimmed.lines().collect();

    // JSON wins outright, before any keyword scoring: config files
    // (package.json, tauri.conf.json, ...) mention "npm"/"git" and contain
    // "fi" inside "identifier", which used to let bash steal them.
    if looks_like_json(trimmed) {
        return Some("json".to_string());
    }

    // Unambiguous headers / preprocessor lines.
    if let Some(lang) = header_language(trimmed) {
        return Some(lang);
    }

    // Language-unique signatures override keyword scoring (e.g. a C# class
    // also looks a lot like Java, but Console.Write is unambiguous).
    for (signature, lang) in UNIQUE_SIGNATURES {
        if trimmed.contains(signature) {
            return Some(lang.to_string());
        }
    }

    // Single line snippets: only accept very strong signals.
    if lines.len() == 1 {
        if let Some(lang) = bracket_language(trimmed) {
            return Some(lang);
        }
        return if looks_like_statement(trimmed) {
            Some(
                best_keyword_match(trimmed)
                    .map(|(name, _)| name)
                    .unwrap_or_else(|| "javascript".to_string()),
            )
        } else {
            None
        };
    }

    // Multi-line: combine structural signals with keyword scoring.
    let structural = structural_score(trimmed, &lines);
    let keyword_best = best_keyword_match(trimmed);

    if let Some((lang, distinct)) = &keyword_best {
        // Flat formats like SQL have little "structure"; trust a clear
        // keyword winner when there is at least one structural signal, or
        // several distinct keywords on a longer snippet.
        if structural >= 1 || lines.len() >= 3 || *distinct >= 3 {
            return Some(lang.clone());
        }
    }

    // key: value blocks with indentation are almost always YAML/TOML.
    if is_yaml_like(&lines) {
        return Some("yaml".to_string());
    }

    // No clear keyword winner: allow pure-structure formats (JSON, CSS,
    // HTML, ...) when the structural evidence is strong.
    if structural >= 4 {
        return structural_language(trimmed, &lines);
    }

    None
}

/// Single-line JSON object/array detection.
fn bracket_language(line: &str) -> Option<String> {
    let t = line.trim();
    if t.starts_with('{') && t.ends_with('}') && t.contains("\":") {
        return Some("json".to_string());
    }
    if t.starts_with('[')
        && t.ends_with(']')
        && (t.contains('"') || t.contains('}') || t.contains('{'))
    {
        return Some("json".to_string());
    }
    None
}

/// Multi-line JSON detection: starts with `{` (object, needs at least one
/// quoted key) or `[` (array, needs quoted values).
fn looks_like_json(t: &str) -> bool {
    if t.starts_with('{') {
        return t.contains("\":");
    }
    if t.starts_with('[') {
        return t.matches('"').count() >= 2;
    }
    false
}

fn is_yaml_like(lines: &[&str]) -> bool {
    let colon_lines = lines
        .iter()
        .filter(|l| {
            let t = l.trim_end();
            t.ends_with(':') && !t.contains("http")
        })
        .count();
    let indented = lines
        .iter()
        .filter(|l| l.starts_with([' ', '\t']) && !l.trim().is_empty())
        .count();
    colon_lines >= 2 && indented >= 2
}

fn header_language(text: &str) -> Option<String> {
    if text.starts_with("#!") || text.starts_with('`') {
        Some("bash".to_string())
    } else if text.starts_with("<?php") {
        Some("php".to_string())
    } else if text.starts_with("<!DOCTYPE") || text.starts_with("<html") {
        Some("html".to_string())
    } else if text.starts_with("#include") || text.starts_with("#define") {
        Some(if text.contains("std::") {
            "c++".to_string()
        } else {
            "c".to_string()
        })
    } else {
        None
    }
}

fn looks_like_statement(line: &str) -> bool {
    let l = line.to_ascii_lowercase();
    let strong = [
        "def ", "fn ", "func ", "function ", "public ", "private ", "class ",
        "struct ", "interface ", "import ", "const ", "let ", "var ", "return ",
        "print(", "echo ", "select ", "insert ", "update ", "match ",
    ];
    let has_delim = line.contains(';')
        || line.contains('{')
        || line.contains('}')
        || line.contains("=>")
        || line.contains("::")
        || line.ends_with(':');
    strong.iter().any(|k| l.starts_with(k) || l.contains(k)) && has_delim
}

fn structural_score(text: &str, lines: &[&str]) -> usize {
    let mut score = 0;

    let lines_with_braces = lines
        .iter()
        .filter(|l| l.contains('{') || l.contains('}'))
        .count();
    if lines_with_braces >= 2 {
        score += 2;
    } else if lines_with_braces == 1 {
        score += 1;
    }

    if lines.iter().any(|l| l.trim_end().ends_with(';')) {
        score += 1;
    }
    if lines.iter().any(|l| l.trim_end().ends_with(':')) {
        score += 1;
    }
    if text.contains("=>") || text.contains("::") || text.contains("->") {
        score += 1;
    }
    if lines
        .iter()
        .filter(|l| l.starts_with([' ', '\t']) && !l.trim().is_empty())
        .count()
        >= 2
    {
        score += 1;
    }
    // key: value style lines (YAML-ish)
    let colon_lines = lines
        .iter()
        .filter(|l| {
            let t = l.trim_end();
            t.ends_with(':') && !t.contains("http")
        })
        .count();
    if colon_lines >= 2 {
        score += 1;
    }
    if text.starts_with('{') || text.starts_with('[') {
        score += 1;
    }
    if lines
        .iter()
        .any(|l| l.trim_start().starts_with('<') && l.contains('>'))
    {
        score += 1;
    }

    score
}

fn structural_language(text: &str, lines: &[&str]) -> Option<String> {
    let t = text.trim();
    if t.starts_with('{') || t.starts_with('[') {
        return Some("json".to_string());
    }
    if lines
        .iter()
        .any(|l| l.trim_start().starts_with('<') && l.contains('>'))
    {
        return Some("html".to_string());
    }
    if text.contains("color:")
        || text.contains("background")
        || text.contains("font-")
        || text.contains("margin")
        || text.contains("padding")
    {
        return Some("css".to_string());
    }
    let colon_lines = lines
        .iter()
        .filter(|l| {
            let s = l.trim_end();
            s.ends_with(':') && !s.contains("http")
        })
        .count();
    if colon_lines >= 2 {
        return Some("yaml".to_string());
    }
    if lines
        .iter()
        .any(|l| l.trim_start().starts_with("["))
        && lines.iter().any(|l| l.contains('='))
    {
        return Some("yaml".to_string());
    }
    None
}

/// Whole-word-ish keyword check. A plain `text.contains("fi")` used to match
/// inside "identifier" and let bash steal JSON/YAML snippets; keywords now
/// must stand alone (alphanumeric/underscore boundaries). Keywords containing
/// non-identifier characters ("std::", "=>", "console.", "<?php", ...) match
/// literally.
fn keyword_hit(text: &str, kw: &str) -> bool {
    let kw = kw.trim();
    if kw.is_empty() {
        return false;
    }
    if kw.chars().any(|c| !c.is_alphanumeric() && c != '_') {
        return text.contains(kw);
    }
    let pattern = format!(r"(?<![A-Za-z0-9_]){}(?![A-Za-z0-9_])", kw);
    match regex::Regex::new(&pattern) {
        Ok(re) => re.is_match(text),
        Err(_) => text.contains(kw),
    }
}

/// Returns the language with the highest number of distinct keywords found
/// (requires at least two distinct hits to count as a match), together with
/// the number of distinct hits.
fn best_keyword_match(text: &str) -> Option<(String, usize)> {
    let mut best: Option<(String, usize)> = None;
    for (name, keywords) in LANGUAGE_KEYWORDS {
        let distinct = keywords.iter().filter(|kw| keyword_hit(text, *kw)).count();
        if distinct >= 2
            && best.as_ref().map_or(true, |(_, score)| distinct > *score)
        {
            best = Some((name.to_string(), distinct));
        }
    }
    best
}

/// Substrings that only appear in one language, used to break ties that the
/// keyword scorer cannot resolve (C# vs Java, Rust vs others, ...).
const UNIQUE_SIGNATURES: &[(&str, &str)] = &[
    ("Console.Write", "csharp"),
    ("System.out", "java"),
    ("println!", "rust"),
    ("<?php", "php"),
    ("std::", "c++"),
    ("import java", "java"),
    ("fmt.", "go"),
    ("module.exports", "javascript"),
    // Python is easy to lose to the YAML heuristic (colon + indentation);
    // these tokens are not ambiguous (def/print are NOT - Ruby and Swift
    // use them too, so they stay in keyword scoring instead).
    ("elif ", "python"),
    ("lambda", "python"),
];

#[cfg(test)]
mod tests {
    use super::detect_code;

    /// The exact class of bug users reported: a JSON config file mentioning
    /// "npm" (and "fi" inside "identifier") was labeled bash.
    #[test]
    fn json_config_file_is_json_not_bash() {
        let json = r#"{
            "productName": "Qopy",
            "version": "0.4.0",
            "identifier": "net.pandadev.qopy",
            "build": {
                "frontendDist": "../dist",
                "beforeDevCommand": "npm run dev",
                "devUrl": "http://localhost:3000",
                "beforeBuildCommand": "npm run build"
            }
        }"#;
        assert_eq!(detect_code(json).as_deref(), Some("json"));
    }

    #[test]
    fn real_bash_stays_bash() {
        assert_eq!(
            detect_code("#!/bin/bash\nnpm install && npm run dev").as_deref(),
            Some("bash")
        );
    }

    #[test]
    fn fi_inside_identifier_is_not_a_bash_keyword() {
        let text = "identifier: net.pandadev.qopy\nname: Qopy\n";
        assert_ne!(detect_code(text).as_deref(), Some("bash"));
    }

    #[test]
    fn python_snippet_still_python() {
        let py = "def fib(n):\n    if n <= 1:\n        return n\n    return fib(n - 1) + fib(n - 2)\n";
        assert_eq!(detect_code(py).as_deref(), Some("python"));
    }
}

const LANGUAGE_KEYWORDS: &[(&str, &[&str])] = &[
    (
        "python",
        &[
            "def ", "import ", "from ", "elif ", "self", "lambda", "print(",
            "None", "True", "False", "class ", "async ", "with ", "return ",
        ],
    ),
    (
        "rust",
        &[
            "fn ", "let ", "mut ", "pub ", "impl ", "struct ", "enum ",
            "match ", "->", "::", "use ", "vec!",
        ],
    ),
    (
        "typescript",
        &[
            "interface ", "type ", "readonly ", "enum ", ": string",
            ": number", "export ", "import ",
        ],
    ),
    (
        "javascript",
        &[
            "function ", "const ", "let ", "var ", "=>", "console.",
            "document.", "window.", "require(", "module.exports", "async ",
        ],
    ),
    (
        "go",
        &["func ", "package ", "import (", "fmt.", ":=", "defer ", "chan "],
    ),
    (
        "java",
        &[
            "public ", "private ", "void ", "System.out", "class ",
            "extends ", "import java", "@Override", "static ",
        ],
    ),
    (
        "c++",
        &[
            "std::", "#include < ", "cout <<", "cin >>", "namespace ",
            "template < ",
        ],
    ),
    (
        "c",
        &[
            "#include", "int main", "printf(", "malloc(", "NULL", "#define",
            "struct ", "unsigned ",
        ],
    ),
    (
        "csharp",
        &[
            "using System", "namespace ", "Console.Write", "public class",
            "var ", "static ",
        ],
    ),
    (
        "bash",
        &[
            "#!/bin/", "echo ", "cd ", "sudo ", "export ", "apt ", "npm ",
            "git ", "curl ", "fi", "esac", "$(",
        ],
    ),
    (
        "sql",
        &[
            "SELECT ", "FROM ", "WHERE ", "INSERT INTO", "UPDATE ",
            "DELETE FROM", "JOIN ", "CREATE TABLE", "GROUP BY", "ORDER BY",
            "LIMIT ",
        ],
    ),
    (
        "html",
        &["<html", "<div", "<span", "<a ", "href=", "class=", "</", "<body", "<head"],
    ),
    (
        "css",
        &[
            "color:", "background", "font-", "margin", "padding", "border",
            "display:", "width:",
        ],
    ),
    ("php", &["<?php", "echo ", "function ", "$", "array("]),
    ("ruby", &["def ", "end", "puts ", "require ", "class ", "do |"]),
    ("swift", &["func ", "var ", "let ", "import ", "print(", "struct ", "guard "]),
    (
        "kotlin",
        &["fun ", "val ", "var ", "companion object", "import ", "main("],
    ),
];
