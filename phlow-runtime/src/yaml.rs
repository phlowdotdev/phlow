use regex::Regex;
use std::fs;

pub fn yaml_helpers_transform(yaml: &str) -> String {
    yaml_helpers_eval(&yaml_helpers_include(yaml))
}

fn yaml_helpers_include(yaml: &str) -> String {
    let include_regex = Regex::new(r"(?m)^(\s*)!include\s+(\S+)").unwrap();

    include_regex
        .replace_all(yaml, |caps: &regex::Captures| {
            let indent = &caps[1];
            let path = &caps[2];
            match fs::read_to_string(path) {
                Ok(contents) => {
                    // Aplica recursivamente para permitir includes dentro de includes
                    let included = yaml_helpers_include(&contents);
                    included
                        .lines()
                        .map(|line| format!("{}{}", indent, line))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                Err(_) => format!("{}<!-- Error including file: {} -->", indent, path),
            }
        })
        .to_string()
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
                if before_eval.trim().is_empty() {
                    result.push_str(&format!("{}{{{{ {} }}}}\n", indent, single_line));
                } else {
                    result.push_str(&format!("{}{{{{ {} }}}}\n", before_eval, single_line));
                }
            } else if !after_eval.is_empty() {
                result.push_str(&format!("{}{{{{ {} }}}}\n", before_eval, after_eval));
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
                result.push_str(&format!("{}{{{{ {} }}}}\n", indent, single_line));
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
            item: {{ 1 + 1 }}
            {{ 2 + 2 }}
            item2: {{ 3 + 3 }}
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
            {{ 1 + 1 }}
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
            item: {{ 1 + 1 }}
            item2: {{ let a = 2; let b = 2; a + b }}
            {{ 3 + 3 }}
        "#;

        assert_eq!(yaml_helpers_eval(yaml), expected);
    }
}
