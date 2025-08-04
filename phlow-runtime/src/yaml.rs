use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn yaml_helpers_transform(
    yaml: &str,
    base_path: &Path,
    print_yaml: bool,
) -> Result<String, Vec<String>> {
    let (yaml, errors) = yaml_helpers_include(yaml, base_path);

    if !errors.is_empty() {
        eprintln!("❌ YAML Transformation Errors:");
        for (i, error) in errors.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, error);
        }
        eprintln!();
        return Err(errors);
    }

    let yaml = yaml_helpers_eval(&yaml);

    if print_yaml {
        println!("");
        println!("#####################################################################");
        println!("# YAML TRANSFORMED");
        println!("#####################################################################");
        println!("{}", yaml);
        println!("#####################################################################");
        println!("");
    }

    Ok(yaml)
}

fn yaml_helpers_include(yaml: &str, base_path: &Path) -> (String, Vec<String>) {
    let mut errors = Vec::new();
    let include_block_regex = match Regex::new(r"(?m)^(\s*)!include\s+([^\s]+)(.*)") {
        Ok(re) => re,
        Err(_) => return (yaml.to_string(), errors),
    };
    let include_inline_regex = match Regex::new(r"!include\s+([^\s]+)(.*)") {
        Ok(re) => re,
        Err(_) => return (yaml.to_string(), errors),
    };
    let import_inline_regex = match Regex::new(r"!import\s+(\S+)") {
        Ok(re) => re,
        Err(_) => return (yaml.to_string(), errors),
    };

    let with_block_includes = include_block_regex.replace_all(yaml, |caps: &regex::Captures| {
        let indent = &caps[1];
        let rel_path = &caps[2];
        let args_str = caps.get(3).map(|m| m.as_str()).unwrap_or("").trim();
        let args = parse_include_args(args_str);
        let full_path = base_path.join(rel_path);
        match process_include_file(&full_path, &args) {
            Ok(json_str) => json_str
                .lines()
                .map(|line| format!("{}{}", indent, line))
                .collect::<Vec<_>>()
                .join("\n"),
            Err(e) => {
                errors.push(format!("Error including file {}: {}", rel_path, e));
                format!("{}<!-- Error including file: {} -->", indent, rel_path)
            }
        }
    });

    let with_inline_includes =
        include_inline_regex.replace_all(&with_block_includes, |caps: &regex::Captures| {
            let rel_path = &caps[1];
            let args_str = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();
            let args = parse_include_args(args_str);
            let full_path = base_path.join(rel_path);
            match process_include_file(&full_path, &args) {
                Ok(json_str) => json_str,
                Err(e) => {
                    errors.push(format!("Error including file {}: {}", rel_path, e));
                    format!("<!-- Error including file: {} -->", rel_path)
                }
            }
        });

    let result = import_inline_regex
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
                Err(_) => {
                    errors.push(format!("Error importing file {}: file not found", rel_path));
                    format!("<!-- Error importing file: {} -->", rel_path)
                }
            }
        })
        .to_string();

    (result, errors)
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

fn parse_include_args(args_str: &str) -> HashMap<String, String> {
    let mut args = HashMap::new();

    if args_str.trim().is_empty() {
        return args;
    }

    // Parse arguments in format: key=value key2='value with spaces' key3="quoted value"
    let arg_regex = match Regex::new(r#"(\w+)=(?:'([^']*)'|"([^"]*)"|([^\s]+))"#) {
        Ok(re) => re,
        Err(_) => return args,
    };

    for caps in arg_regex.captures_iter(args_str) {
        let key = caps[1].to_string();
        let value = caps
            .get(2)
            .or(caps.get(3))
            .or(caps.get(4))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        args.insert(key, value);
    }

    args
}

fn process_args_in_content(content: &str, args: &HashMap<String, String>) -> (String, Vec<String>) {
    let mut errors = Vec::new();
    let arg_regex = match Regex::new(r"!arg\s+(\w+)") {
        Ok(re) => re,
        Err(_) => return (content.to_string(), errors),
    };

    let result = arg_regex
        .replace_all(content, |caps: &regex::Captures| {
            let arg_name = &caps[1];
            match args.get(arg_name) {
                Some(value) => value.clone(),
                None => {
                    errors.push(format!("Missing required argument: '{}'", arg_name));
                    format!("<!-- Error: argument '{}' not found -->", arg_name)
                }
            }
        })
        .to_string();

    (result, errors)
}

fn process_include_file(path: &Path, args: &HashMap<String, String>) -> Result<String, String> {
    let path = if path.extension().is_none() {
        let mut new_path = path.to_path_buf();
        new_path.set_extension("phlow");
        new_path
    } else {
        path.to_path_buf()
    };

    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;

    // First, process !arg directives with the provided arguments
    let (with_args, arg_errors) = process_args_in_content(&raw, args);

    // If there are argument errors, return them immediately
    if !arg_errors.is_empty() {
        return Err(arg_errors.join("; "));
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let transformed =
        yaml_helpers_transform(&with_args, parent, false).map_err(|errors| errors.join("; "))?;

    Ok(transformed)
}
