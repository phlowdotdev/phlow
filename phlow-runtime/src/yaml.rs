use regex::Regex;
use serde_json::json;
use serde_yaml;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn yaml_helpers_transform(yaml: &str) -> String {
    yaml_helpers_eval(&yaml_helpers_include(yaml))
}

fn yaml_helpers_include(yaml: &str) -> String {
    let include_block_regex = Regex::new(r"(?m)^(\s*)!include\s+(\S+)").unwrap();
    let include_inline_regex = Regex::new(r"!include\s+(\S+)").unwrap();
    let import_inline_regex = Regex::new(r"!import\s+(\S+)").unwrap();

    // Trata includes em bloco (preservando indentação)
    let with_block_includes = include_block_regex.replace_all(yaml, |caps: &regex::Captures| {
        let indent = &caps[1];
        let path = &caps[2];
        match process_include_file(path) {
            Ok(json_str) => json_str
                .lines()
                .map(|line| format!("{}{}", indent, line))
                .collect::<Vec<_>>()
                .join("\n"),
            Err(e) => format!("{}<!-- Error including file: {}: {} -->", indent, path, e),
        }
    });

    // Trata includes inline
    let with_inline_includes =
        include_inline_regex.replace_all(&with_block_includes, |caps: &regex::Captures| {
            let path = &caps[1];
            match process_include_file(path) {
                Ok(json_str) => json_str,
                Err(e) => format!("<!-- Error including file: {}: {} -->", path, e),
            }
        });

    // Trata import inline
    import_inline_regex
        .replace_all(&with_inline_includes, |caps: &regex::Captures| {
            let path = &caps[1];
            match fs::read_to_string(path) {
                Ok(contents) => {
                    let one_line = contents
                        .lines()
                        .map(str::trim)
                        .collect::<Vec<_>>()
                        .join(" ")
                        .replace('"', "\\\"");
                    format!(r#""{{{{ {} }}}}""#, one_line)
                }
                Err(_) => format!("<!-- Error importing file: {} -->", path),
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

fn process_include_file(path: &str) -> Result<String, String> {
    let path_obj = Path::new(path);
    let extension = path_obj
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let value = match extension.as_str() {
        "yaml" | "yml" => {
            let transformed = yaml_helpers_transform(&raw);
            let yaml_value: serde_yaml::Value =
                serde_yaml::from_str(&transformed).map_err(|e| e.to_string())?;
            serde_json::to_value(yaml_value).map_err(|e| e.to_string())?
        }
        "json" => serde_json::from_str::<serde_json::Value>(&raw).map_err(|e| e.to_string())?,
        "toml" => {
            let toml_value: toml::Value = toml::from_str(&raw).map_err(|e| e.to_string())?;
            serde_json::to_value(toml_value).map_err(|e| e.to_string())?
        }
        _ => return Err("Unsupported file extension".into()),
    };

    // Converte o conteúdo para uma string JSON bonita
    serde_json::to_string_pretty(&value).map_err(|e| e.to_string())
}
