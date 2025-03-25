use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;

pub fn yaml_helpers_transform(yaml: &str) -> String {
    yaml_helpers_eval(&yaml_helpers_include(yaml))
}

fn yaml_helpers_include(yaml: &str) -> String {
    let include_block_regex = Regex::new(r"(?m)^(\s*)!include\s+(\S+)").unwrap();
    let include_inline_regex = Regex::new(r"!include\s+(\S+)").unwrap();

    // Primeiro: processa includes em bloco (início da linha, com indentação)
    let with_block_includes = include_block_regex.replace_all(yaml, |caps: &regex::Captures| {
        let indent = &caps[1];
        let path = &caps[2];
        match process_included_file(path, Some(indent)) {
            Ok(json) => json,
            Err(err) => format!("{}<!-- {} -->", indent, err),
        }
    });

    // Depois: processa includes inline
    include_inline_regex
        .replace_all(&with_block_includes, |caps: &regex::Captures| {
            let path = &caps[1];
            match process_included_file(path, None) {
                Ok(json) => json,
                Err(err) => format!("<!-- {} -->", err),
            }
        })
        .to_string()
}

fn process_included_file(path: &str, indent: Option<&str>) -> Result<String, String> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let content =
        fs::read_to_string(path).map_err(|_| format!("Error including file: {}", path))?;

    let json = match ext.as_str() {
        "yaml" | "yml" => {
            let transformed = yaml_helpers_transform(&content);
            let value: serde_yaml::Value =
                serde_yaml::from_str(&transformed).map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
        }
        "json" => {
            let value: serde_json::Value =
                serde_json::from_str(&content).map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
        }
        "toml" => {
            let value: toml::Value = content.parse::<toml::Value>().map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
        }
        _ => return Err(format!("Unsupported file type: {}", path)),
    };

    if let Some(indent) = indent {
        Ok(json
            .lines()
            .map(|line| format!("{}{}", indent, line))
            .collect::<Vec<_>>()
            .join("\n"))
    } else {
        Ok(json)
    }
}

fn yaml_helpers_eval(yaml: &str) -> String {
    let mut result = String::new();
    let mut lines = yaml.lines().peekable();

    while let Some(line) = lines.next() {
        if let Some(pos) = line.find("!eval") {
            let before_eval = &line[..pos];
            let after_eval = line[pos + 5..].trim();
            let indent = " ".repeat(pos);

            if after_eval.starts_with("```") {
                // Bloco markdown-style
                let mut block_lines = vec![];

                if after_eval == "```" {
                    while let Some(next_line) = lines.next() {
                        if next_line.trim() == "```" {
                            break;
                        }
                        block_lines.push(next_line.trim().to_string());
                    }
                } else if let Some(end_pos) = after_eval[3..].find("```") {
                    let inner_code = &after_eval[3..3 + end_pos];
                    block_lines.push(inner_code.trim().to_string());
                }

                let single_line = block_lines.join(" ");
                let escaped = single_line.replace('"', "\\\"");

                if before_eval.trim().is_empty() {
                    result.push_str(&format!("{}\"{{{{ {} }}}}\"\n", indent, escaped));
                } else {
                    result.push_str(&format!("{}\"{{{{ {} }}}}\"\n", before_eval, escaped));
                }
            } else if !after_eval.is_empty() {
                let escaped = after_eval.replace('"', "\\\"");
                result.push_str(&format!("{}\"{{{{ {} }}}}\"\n", before_eval, escaped));
            } else {
                // Bloco indentado
                let mut block_lines = vec![];
                while let Some(&next_line) = lines.peek() {
                    let line_indent = next_line.chars().take_while(|c| c.is_whitespace()).count();
                    if next_line.trim().is_empty() || line_indent > pos {
                        block_lines.push(next_line[pos + 1..].trim().to_string());
                        lines.next();
                    } else {
                        break;
                    }
                }

                let single_line = block_lines.join(" ");
                let escaped = single_line.replace('"', "\\\"");

                result.push_str(&format!("{}\"{{{{ {} }}}}\"\n", indent, escaped));
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result.pop();
    result.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_helpers_include() {
        let _ = fs::remove_file("test1.yaml"); // evita erro se já não existir

        let yaml = r#"
                item: 
                  !include test1.yaml
                !include test2.yaml
                !include test3.yaml
            "#;
        let expected = r#"
                item: 
                  <!-- Error including file: test1.yaml -->
                <!-- Error including file: test2.yaml -->
                <!-- Error including file: test3.yaml -->
            "#;
        assert_eq!(yaml_helpers_include(yaml), expected);
    }

    #[test]
    fn test_yaml_helpers_eval() {
        let yaml = r#"
            item: !eval 1 + 1
            !eval  2 + 2
            item2: !eval 3 + 3
        "#;
        let expected = r#"
            item: "{{ 1 + 1 }}"
            "{{ 2 + 2 }}"
            item2: "{{ 3 + 3 }}"
        "#;
        assert_eq!(yaml_helpers_eval(yaml), expected);
    }

    #[test]
    fn test_yaml_helpers_transform() {
        let test1 = r#"ok"#;
        fs::write("test_ok.yaml", test1).unwrap();

        let yaml = r#"
            !include test_ok.yaml
            !eval 1 + 1
        "#;
        let expected = r#"
            ok
            "{{ 1 + 1 }}"
        "#;

        let result = yaml_helpers_transform(yaml);

        fs::remove_file("test_ok.yaml").unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_yaml_helpers_eval_block_with_backticks() {
        let yaml = r#"
            item: !eval 1 + 1
            item2: !eval ```
                let a = 2;
                let b = 2;
                a + b
            ```
            !eval 3 + 3
        "#;

        let expected = r#"
            item: "{{ 1 + 1 }}"
            item2: "{{ let a = 2; let b = 2; a + b }}"
            "{{ 3 + 3 }}"
        "#;

        assert_eq!(yaml_helpers_eval(yaml), expected);
    }
}
