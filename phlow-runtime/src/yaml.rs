use regex::Regex;
use std::fs;

pub fn yaml_helpers_transform(yaml: &str) -> String {
    let yaml = yaml_helpers_include(yaml);
    let yaml = yaml_helpers_eval(&yaml);
    yaml
}

fn yaml_helpers_include(yaml: &str) -> String {
    let include_regex = Regex::new(r"!include\s+(\S+)").unwrap();
    include_regex
        .replace_all(yaml, |caps: &regex::Captures| {
            let path = &caps[1];
            match fs::read_to_string(path) {
                Ok(contents) => {
                    // Aplica recursivamente para permitir includes dentro de includes
                    yaml_helpers_include(&contents)
                }
                Err(_) => format!("<!-- Error including file: {} -->", path),
            }
        })
        .to_string()
}

fn yaml_helpers_eval(yaml: &str) -> String {
    let mut result = String::new();
    let mut lines = yaml.lines().peekable();

    while let Some(line) = lines.next() {
        if let Some(pos) = line.find("!eval") {
            let after_eval = line[pos + 5..].trim();
            let indent = " ".repeat(pos);

            if !after_eval.is_empty() {
                // Linha única
                result.push_str(&format!("{}{{{{ {} }}}}\n", indent, after_eval));
            } else {
                // Bloco indentado
                let mut block_lines = vec![];
                while let Some(&next_line) = lines.peek() {
                    let line_indent = next_line.chars().take_while(|c| c.is_whitespace()).count();
                    if next_line.trim().is_empty() || line_indent > pos {
                        block_lines.push(next_line[pos + 1..].to_string());
                        lines.next();
                    } else {
                        break;
                    }
                }

                let block = block_lines.join("\n");
                result.push_str(&format!("{}{{{{\n{}\n{}}}}}\n", indent, block, indent));
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result.pop();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_helpers_include() {
        let _ = fs::remove_file("test1.yaml"); // evita erro se já não existir

        let yaml = r#"
                !include test1.yaml
                !include test2.yaml
                !include test3.yaml
            "#;
        let expected = r#"
                <!-- Error including file: test1.yaml -->
                <!-- Error including file: test2.yaml -->
                <!-- Error including file: test3.yaml -->
            "#;
        assert_eq!(yaml_helpers_include(yaml), expected);
    }

    #[test]
    fn test_yaml_helpers_eval() {
        let yaml = r#"
            !eval 1 + 1
            !eval  2 + 2
            !eval 3 + 3
        "#;
        let expected = r#"
            {{ 1 + 1 }}
            {{ 2 + 2 }}
            {{ 3 + 3 }}
        "#;
        assert_eq!(yaml_helpers_eval(yaml), expected);
    }

    #[test]
    fn test_yaml_helpers_transform() {
        let test1 = r#"ok"#;
        fs::write("test_ok.yaml", test1).unwrap();

        let yaml = r#"
            !include test_ok.yaml
            !eval
            1 + 1
        "#;
        let expected = r#"
            ok
            {{ 1 + 1 }}
        "#;

        let result = yaml_helpers_transform(yaml);

        fs::remove_file("test1.yaml").unwrap();

        assert_eq!(result, expected);
    }
}
