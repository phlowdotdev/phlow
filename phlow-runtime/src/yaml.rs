use regex::Regex;
use serde_yaml::{Mapping, Value};
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
    let yaml = yaml_transform_modules(&yaml)?;

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
            } else if after_eval.starts_with("{") {
                // Bloco de código delimitado por {}
                let mut block_content = String::new();
                let mut brace_count = 0;

                // Primeiro, verifica se há conteúdo na mesma linha
                for ch in after_eval.chars() {
                    block_content.push(ch);
                    if ch == '{' {
                        brace_count += 1;
                    } else if ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                }

                // Se não fechou na mesma linha, continue lendo
                while brace_count > 0 {
                    if let Some(next_line) = lines.next() {
                        for ch in next_line.chars() {
                            block_content.push(ch);
                            if ch == '{' {
                                brace_count += 1;
                            } else if ch == '}' {
                                brace_count -= 1;
                                if brace_count == 0 {
                                    break;
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }

                // Remove as chaves externas e processa o conteúdo
                let inner_content =
                    if block_content.starts_with('{') && block_content.ends_with('}') {
                        &block_content[1..block_content.len() - 1]
                    } else {
                        &block_content
                    };

                // Unifica em uma linha, removendo quebras de linha desnecessárias
                let single_line = inner_content
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");

                let escaped = single_line.replace('"', "\\\"");

                if before_eval.trim().is_empty() {
                    result.push_str(&format!("{}\"{{{{ {{  {} }} }}}}\"\n", indent, escaped));
                } else {
                    result.push_str(&format!(
                        "{}\"{{{{ {{  {} }} }}}}\"\n",
                        before_eval, escaped
                    ));
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

fn yaml_transform_modules(yaml: &str) -> Result<String, Vec<String>> {
    // Lista de propriedades exclusivas do projeto
    let exclusive_properties = vec![
        "use",
        "to",
        "id",
        "label",
        "assert",
        "condition",
        "return",
        "payload",
        "input",
        "then",
        "else",
        "steps",
    ];

    // Parse o YAML para extrair módulos disponíveis
    let parsed: Value = match serde_yaml::from_str(yaml) {
        Ok(val) => val,
        Err(_) => return Ok(yaml.to_string()), // Se não conseguir parsear, retorna o original
    };

    let mut available_modules = std::collections::HashSet::new();

    // Extrai módulos da seção "modules"
    if let Some(modules) = parsed.get("modules") {
        if let Some(modules_array) = modules.as_sequence() {
            for module in modules_array {
                if let Some(module_map) = module.as_mapping() {
                    // Verifica se existe "module" ou "name"
                    if let Some(module_name) = module_map
                        .get("module")
                        .or_else(|| module_map.get("name"))
                        .and_then(|v| v.as_str())
                    {
                        // Extrai o nome do módulo de paths locais (./modules/cognito -> cognito)
                        let clean_name = if module_name.starts_with("./modules/") {
                            &module_name[10..] // Remove "./modules/"
                        } else if module_name.contains('/') {
                            // Para outros paths, pega apenas o último segmento
                            module_name.split('/').last().unwrap_or(module_name)
                        } else {
                            module_name
                        };
                        available_modules.insert(clean_name.to_string());
                    }
                }
            }
        }
    }

    if available_modules.is_empty() {
        return Ok(yaml.to_string()); // Sem módulos para transformar
    }

    // Função recursiva para transformar o YAML
    fn transform_value(
        value: &mut Value,
        available_modules: &std::collections::HashSet<String>,
        exclusive_properties: &[&str],
        is_in_transformable_context: bool,
    ) {
        match value {
            Value::Mapping(map) => {
                let mut transformations = Vec::new();

                for (key, val) in map.iter() {
                    if let Some(key_str) = key.as_str() {
                        // Só transforma se estiver em um contexto transformável (raiz de steps, then ou else)
                        if is_in_transformable_context {
                            // Se não é uma propriedade exclusiva e é um módulo disponível
                            if !exclusive_properties.contains(&key_str)
                                && available_modules.contains(key_str)
                            {
                                transformations.push((key.clone(), val.clone()));
                            }
                        }
                    }
                }

                // Aplica as transformações
                for (key, old_val) in transformations {
                    map.remove(&key);

                    let mut new_entry = Mapping::new();
                    new_entry.insert(Value::String("use".to_string()), key);
                    new_entry.insert(Value::String("input".to_string()), old_val);

                    // Adiciona a nova entrada transformada
                    for (new_key, new_val) in new_entry.iter() {
                        map.insert(new_key.clone(), new_val.clone());
                    }
                }

                // Continua a transformação recursivamente
                for (key, val) in map.iter_mut() {
                    let key_str = key.as_str().unwrap_or("");

                    // Determina se o próximo nível será transformável
                    let next_is_transformable =
                        key_str == "steps" || key_str == "then" || key_str == "else";

                    transform_value(
                        val,
                        available_modules,
                        exclusive_properties,
                        next_is_transformable,
                    );
                }
            }
            Value::Sequence(seq) => {
                for item in seq.iter_mut() {
                    transform_value(
                        item,
                        available_modules,
                        exclusive_properties,
                        is_in_transformable_context,
                    );
                }
            }
            _ => {}
        }
    }

    // Parse novamente para modificar
    let mut parsed_mut: Value = match serde_yaml::from_str(yaml) {
        Ok(val) => val,
        Err(_) => return Ok(yaml.to_string()),
    };

    transform_value(
        &mut parsed_mut,
        &available_modules,
        &exclusive_properties,
        false, // Começa como false, só será true dentro de steps, then ou else
    );

    // Converte de volta para YAML
    match serde_yaml::to_string(&parsed_mut) {
        Ok(result) => Ok(result),
        Err(_) => Ok(yaml.to_string()),
    }
}
