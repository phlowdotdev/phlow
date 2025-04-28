use phlow_sdk::prelude::Value;
use regex::Regex;
use serde_yaml;
use std::fs;
use std::path::Path;

pub fn yaml_helpers_transform(yaml: &str, base_path: &Path, print_yaml: bool) -> String {
    let yaml = yaml_helpers_eval(&yaml_helpers_include(yaml, base_path));

    if print_yaml {
        println!("");
        println!("#####################################################################");
        println!("# YAML TRANSFORMED");
        println!("#####################################################################");
        println!("{}", yaml);
        println!("#####################################################################");
        println!("");
    }

    yaml
}

fn yaml_helpers_include(yaml: &str, base_path: &Path) -> String {
    let include_block_regex = match Regex::new(r"(?m)^(\s*)!include\s+(\S+)") {
        Ok(re) => re,
        Err(_) => return yaml.to_string(),
    };
    let include_inline_regex = match Regex::new(r"!include\s+(\S+)") {
        Ok(re) => re,
        Err(_) => return yaml.to_string(),
    };
    let import_inline_regex = match Regex::new(r"!import\s+(\S+)") {
        Ok(re) => re,
        Err(_) => return yaml.to_string(),
    };

    let with_block_includes = include_block_regex.replace_all(yaml, |caps: &regex::Captures| {
        let indent = &caps[1];
        let rel_path = &caps[2];
        let full_path = base_path.join(rel_path);
        match process_include_file(&full_path) {
            Ok(json_str) => json_str
                .lines()
                .map(|line| format!("{}{}", indent, line))
                .collect::<Vec<_>>()
                .join("\n"),
            Err(e) => format!(
                "{}<!-- Error including file: {}: {} -->",
                indent, rel_path, e
            ),
        }
    });

    let with_inline_includes =
        include_inline_regex.replace_all(&with_block_includes, |caps: &regex::Captures| {
            let rel_path = &caps[1];
            let full_path = base_path.join(rel_path);
            match process_include_file(&full_path) {
                Ok(json_str) => json_str,
                Err(e) => format!("<!-- Error including file: {}: {} -->", rel_path, e),
            }
        });

    import_inline_regex
        .replace_all(&with_inline_includes, |caps: &regex::Captures| {
            let rel_path = &caps[1];
            let full_path = base_path.join(rel_path);
            let extension = full_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            match fs::read_to_string(&full_path) {
                Ok(contents) => {
                    let one_line = contents
                        .lines()
                        .map(str::trim)
                        .collect::<Vec<_>>()
                        .join(" ")
                        .replace('"', "\\\"");

                    if extension == "phs" || extension == "rhai" {
                        format!(r#""{{{{ {} }}}}""#, one_line)
                    } else {
                        format!(r#""{}""#, one_line)
                    }
                }
                Err(_) => format!("<!-- Error importing file: {} -->", rel_path),
            }
        })
        .to_string()
}

fn yaml_helpers_eval(yaml: &str) -> String {
    let mut result = String::new();
    let mut lines = yaml.lines().peekable();

    while let Some(line) = lines.next() {
        if let Some(pos) = line.find("!phs") {
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

fn process_include_file(path: &Path) -> Result<String, String> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let value: Value = match extension.as_str() {
        "yaml" | "yml" => {
            let parent = path.parent().unwrap_or_else(|| Path::new("."));
            let transformed = yaml_helpers_transform(&raw, parent, false);
            serde_yaml::from_str(&transformed).map_err(|e| e.to_string())?
        }
        _ => return Err("Unsupported file extension".into()),
    };

    Ok(value.to_string())
}
