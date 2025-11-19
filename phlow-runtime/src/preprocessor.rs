use phlow_sdk::prelude::*;
use regex::Regex;
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn preprocessor(
    phlow: &str,
    base_path: &Path,
    print_phlow: bool,
) -> Result<String, Vec<String>> {
    let (phlow, errors) = preprocessor_directives(phlow, base_path);

    if !errors.is_empty() {
        eprintln!("❌ YAML Transformation Errors:");
        for (i, error) in errors.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, error);
        }
        eprintln!();
        return Err(errors);
    }

    // Primeiro, converte blocos ``` em strings para evitar alterações indevidas dentro do bloco
    let phlow = preprocessor_markdown_string_blocks(&phlow);
    // Depois, aplica auto-transformações de objetos/arrays e PHS oculto
    let phlow = processor_transform_phs_hidden_object_and_arrays(&phlow);
    let phlow = preprocessor_transform_phs_hidden(&phlow);
    let phlow = preprocessor_eval(&phlow);
    let phlow = preprocessor_modules(&phlow)?;
    // Esta fazendo dupla checagem para gar se tudo que precisa estar com !phs está certo
    // Exemplo valores extendidos um mulplas linhas com pipe e identação
    // let phlow = preprocessor_transform_phs_hidden(&phlow);
    // let phlow = preprocessor_eval(&phlow);
    // let phlow = preprocessor_modules(&phlow)?;

    if print_phlow {
        println!("");
        println!("# PHLOW TRANSFORMED");
        println!("#####################################################################");
        println!("{}", phlow);
        println!("#####################################################################");
        println!("");
    }

    Ok(phlow)
}

fn preprocessor_directives(phlow: &str, base_path: &Path) -> (String, Vec<String>) {
    let mut errors = Vec::new();
    let include_block_regex = match Regex::new(r"(?m)^(\s*)!include\s+([^\s]+)(.*)") {
        Ok(re) => re,
        Err(_) => return (phlow.to_string(), errors),
    };
    // Captura o prefixo da linha até o !include para poder aplicar a mesma
    // tabulação nas linhas subsequentes do conteúdo incluído.
    // Grupo 1: tudo desde o início da linha até antes de !include
    // Grupo 2: caminho do arquivo
    // Grupo 3: argumentos opcionais
    let include_inline_regex = match Regex::new(r"(?m)^([^\n]*?)!include\s+([^\s]+)(.*)") {
        Ok(re) => re,
        Err(_) => return (phlow.to_string(), errors),
    };
    let import_inline_regex = match Regex::new(r"!import\s+(\S+)") {
        Ok(re) => re,
        Err(_) => return (phlow.to_string(), errors),
    };

    let with_block_includes = include_block_regex.replace_all(&phlow, |caps: &regex::Captures| {
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
            let prefix = &caps[1];
            let rel_path = &caps[2];
            let args_str = caps.get(3).map(|m| m.as_str()).unwrap_or("").trim();
            let args = parse_include_args(args_str);
            let full_path = base_path.join(rel_path);

            match process_include_file(&full_path, &args) {
                Ok(json_str) => {
                    // Calcula a indentação de continuação com o mesmo comprimento do prefixo
                    let continuation_indent: String = prefix
                        .chars()
                        .map(|ch| if ch.is_whitespace() { ch } else { ' ' })
                        .collect();

                    let mut lines = json_str.lines();
                    if let Some(first) = lines.next() {
                        let mut out = String::new();
                        out.push_str(prefix);
                        out.push_str(first);
                        for line in lines {
                            out.push('\n');
                            out.push_str(&continuation_indent);
                            out.push_str(line);
                        }
                        out
                    } else {
                        // Conteúdo vazio: mantém apenas o prefixo
                        prefix.to_string()
                    }
                }
                Err(e) => {
                    errors.push(format!("Error including file {}: {}", rel_path, e));
                    format!("{}<!-- Error including file: {} -->", prefix, rel_path)
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
                    if extension == "phs" {
                        let one_line = contents
                            .lines()
                            .map(str::trim)
                            .collect::<Vec<_>>()
                            .join(" ")
                            .replace('"', "\\\"");

                        // Somente arquivos com extensão .phs devem ser tratados como código PHS.
                        // Demais extensões devem ser retornadas como string literal.

                        format!(r#""{{{{ {} }}}}""#, one_line)
                    } else {
                        let content = contents.to_value().to_json_inline();
                        format!(r#"{}"#, content)
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

// identifica blocos de codigo sem phs que são objetos ou arrays, E envole eles com !phs ${ ... }
fn processor_transform_phs_hidden_object_and_arrays(phlow: &str) -> String {
    let mut result = String::new();
    let mut lines = phlow.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed_line = line.trim_start();

        if let Some(colon_pos) = trimmed_line.find(':') {
            let key = &trimmed_line[..colon_pos].trim();
            let value = &trimmed_line[colon_pos + 1..].trim();
            let starts_with_brace = value.starts_with('{');
            let starts_with_bracket = value.starts_with('[');

            if starts_with_brace || starts_with_bracket {
                let indent = &line[..line.len() - trimmed_line.len()];
                let mut block_lines = vec![value.to_string()];

                // Verifica se o bloco fecha na mesma linha
                if !(starts_with_brace && value.ends_with('}'))
                    && !(starts_with_bracket && value.ends_with(']'))
                {
                    // Continua lendo até encontrar o fechamento
                    while let Some(next_line) = lines.next() {
                        block_lines.push(next_line.trim().to_string());
                        if (starts_with_brace && next_line.trim().ends_with('}'))
                            || (starts_with_bracket && next_line.trim().ends_with(']'))
                        {
                            break;
                        }
                    }
                }

                let single_line = block_lines.join(" ");
                result.push_str(&format!("{}{}: !phs ${{ {} }}\n", indent, key, single_line));
                continue;
            }
        }

        result.push_str(line);
        result.push_str("\n");
    }

    // Remove a última quebra de linha extra se houver
    if result.ends_with('\n') {
        result.pop();
    }
    result
}

// Essa função identifica qualquer valore de propriedade que inicie com
//palavras reservadas do phs ou seja um algoritimo e inclui a tag !phs automaticamente
fn preprocessor_transform_phs_hidden(phlow: &str) -> String {
    let operators: Vec<&'static str> = vec![
        "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "??", "?:", "!",
    ];
    let mut reserved_keywords = vec![
        "if", "else", "for", "while", "loop", "match", "let", "const", "fn", "return", "switch",
        "case", "default", "try", "catch", "throw", "when", "payload", "input", "steps", "main",
        "setup", "envs", "tests",
    ];
    reserved_keywords.extend(&operators);

    let mut result = String::new();

    for line in phlow.lines() {
        let trimmed_line = line.trim_start();

        if let Some(colon_pos) = trimmed_line.find(':') {
            let key = &trimmed_line[..colon_pos].trim();
            let value = &trimmed_line[colon_pos + 1..].trim();

            let first_word = value
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("")
                .split('.')
                .next()
                .unwrap_or("");

            if first_word == "!phs" {
                result.push_str(line);
                result.push_str("\n");
                continue;
            }

            // Não marcar blocos iniciando com ``` como !phs; são strings
            if (first_word.starts_with("`") && !first_word.starts_with("```"))
                || first_word.starts_with("${")
            {
                let indent = &line[..line.len() - trimmed_line.len()];
                let content = &format!("{}{}: !phs {}\n", indent, key, value);
                result.push_str(content);
                continue;
            }

            if reserved_keywords.contains(&first_word)
                || (operators
                    .iter()
                    .any(|op| value.contains(&format!(" {} ", op)))
                    && !value.starts_with('"')
                    && !value.starts_with('\''))
            {
                let indent = &line[..line.len() - trimmed_line.len()];
                result.push_str(&format!("{}{}: !phs {}\n", indent, key, value));
                continue;
            }
        } else if trimmed_line.starts_with("-") {
            let after_dash = trimmed_line[1..].trim_start();
            let first_word = after_dash
                .split_whitespace()
                .next()
                .unwrap_or("")
                .split('.')
                .next()
                .unwrap_or("")
                .split('[')
                .next()
                .unwrap_or("");

            if first_word == "!phs" {
                result.push_str(line);
                result.push_str("\n");
                continue;
            }

            // Não marcar blocos iniciando com ``` como !phs; são strings
            if first_word.starts_with("`") && !first_word.starts_with("```") {
                let indent = &line[..line.len() - trimmed_line.len()];
                result.push_str(&format!("{}- !phs {}\n", indent, after_dash));
                continue;
            }

            if reserved_keywords.contains(&first_word)
                || (operators
                    .iter()
                    .any(|op| after_dash.contains(&format!(" {} ", op)))
                    && !after_dash.starts_with('"')
                    && !after_dash.starts_with('\''))
            {
                let indent = &line[..line.len() - trimmed_line.len()];
                result.push_str(&format!("{}- !phs {}\n", indent, after_dash));
                continue;
            }
        }

        result.push_str(line);
        result.push_str("\n");
    }

    result.pop();
    result.to_string()
}

fn is_inside_double_quotes(s: &str, idx: usize) -> bool {
    let mut in_quotes = false;
    let mut escaped = false;
    for (i, ch) in s.chars().enumerate() {
        if i >= idx {
            break;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
        }
    }
    in_quotes
}

fn preprocessor_eval(phlow: &str) -> String {
    let mut result = String::new();
    let mut lines = phlow.lines().peekable();

    while let Some(line) = lines.next() {
        // Encontra um !phs fora de aspas na linha
        let mut search_start = 0usize;
        let mut found_pos: Option<usize> = None;
        while let Some(rel) = line[search_start..].find("!phs") {
            let pos = search_start + rel;
            if !is_inside_double_quotes(line, pos) {
                found_pos = Some(pos);
                break;
            }
            search_start = pos + 4; // avança após !phs
        }

        if let Some(pos) = found_pos {
            let before_eval = &line[..pos];
            let after_eval = if line.len() > pos + 4 {
                line[pos + 4..].trim()
            } else {
                ""
            };
            let indent = " ".repeat(pos);
            // Verifica se o bloco é markdown-style ```
            if after_eval.starts_with("```") {
                // Bloco markdown-style interpretado como string literal
                let mut block_lines = vec![];

                if after_eval == "```" {
                    while let Some(next_line) = lines.next() {
                        if next_line.trim() == "```" {
                            break;
                        }
                        block_lines.push(next_line.trim().to_string());
                    }
                } else if let Some(end_pos) = after_eval[3..].find("```") {
                    let mut inner_code = after_eval[3..3 + end_pos].trim().to_string();
                    // Remove rótulo de linguagem se existir no inline
                    if let Some(space_idx) = inner_code.find(' ') {
                        let (first, rest) = inner_code.split_at(space_idx);
                        if !first.is_empty() {
                            inner_code = rest.trim_start().to_string();
                        }
                    }
                    block_lines.push(inner_code);
                }

                let single_line = block_lines.join(" ");
                let escaped = single_line.replace('"', "\\\"");

                // Saída como string simples, sem "{{ }}"
                if before_eval.trim().is_empty() {
                    result.push_str(&format!("{}\"{}\"\n", indent, escaped));
                } else {
                    result.push_str(&format!("{}\"{}\"\n", before_eval, escaped));
                }
            }
            // Verifica se o bloco é delimitado por ${}
            else if after_eval.starts_with("${") {
                // Bloco de código delimitado por ${}
                let mut block_content = String::new();
                let mut brace_count = 0;
                let mut dollar_brace_started = false;

                // Primeiro, verifica se há conteúdo na mesma linha
                let mut chars = after_eval.chars().peekable();
                while let Some(ch) = chars.next() {
                    block_content.push(ch);

                    if ch == '$' && chars.peek() == Some(&'{') {
                        // Consome o '{'
                        if let Some(next_ch) = chars.next() {
                            block_content.push(next_ch);
                            brace_count += 1;
                            dollar_brace_started = true;
                        }
                    } else if ch == '{' && dollar_brace_started {
                        brace_count += 1;
                    } else if ch == '}' && dollar_brace_started {
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

                // Remove as chaves externas ${} e processa o conteúdo
                let inner_content =
                    if block_content.starts_with("${") && block_content.ends_with('}') {
                        &block_content[2..block_content.len() - 1]
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

                // Escape apenas aspas duplas
                let escaped = single_line.replace('"', "\\\"");

                if before_eval.trim().is_empty() {
                    result.push_str(&format!("{}\"{{{{ {} }}}}\"\n", indent, escaped));
                } else {
                    result.push_str(&format!("{}\"{{{{ {} }}}}\"\n", before_eval, escaped));
                }
            }
            // Verifica se o bloco é uma template string com crases
            else if after_eval.starts_with('`') && after_eval.ends_with('`') {
                // Template string com crases - converte para sintaxe de template string
                let inner_content = &after_eval[1..after_eval.len() - 1];
                let escaped = inner_content.replace('"', "\\\"");
                result.push_str(&format!("{}\"{{{{ `{}` }}}}\"\n", before_eval, escaped));
            }
            // Verifica se o bloco é uma única linha
            else if !after_eval.is_empty() {
                let escaped = after_eval.replace('"', "\\\"");
                result.push_str(&format!("{}\"{{{{ {} }}}}\"\n", before_eval, escaped));
            }
            // Caso contrário, processa como um bloco indentado
            else {
                // Bloco indentado
                let mut block_lines = vec![];
                let current_line_indent = line.chars().take_while(|c| c.is_whitespace()).count();

                while let Some(&next_line) = lines.peek() {
                    let line_indent = next_line.chars().take_while(|c| c.is_whitespace()).count();

                    if next_line.trim().is_empty() {
                        // Pula linhas vazias sem adicionar conteúdo
                        lines.next();
                        continue;
                    } else if line_indent > current_line_indent {
                        // Esta linha é mais indentada que a linha do !phs, então faz parte do bloco
                        let content = next_line.trim().to_string();
                        if !content.is_empty() {
                            block_lines.push(content);
                        }
                        lines.next();
                    } else {
                        break;
                    }
                }

                let single_line = block_lines.join(" ");
                let escaped = single_line.replace('"', "\\\"");

                // Só adiciona se houver conteúdo válido
                if !escaped.trim().is_empty() {
                    if before_eval.trim().is_empty() {
                        result.push_str(&format!("{}\"{{{{ {} }}}}\"\n", indent, escaped));
                    } else {
                        result.push_str(&format!("{}\"{{{{ {} }}}}\"\n", before_eval, escaped));
                    }
                } else {
                    // Se não há conteúdo válido, apenas preserva a linha original sem processamento
                    result.push_str(&format!("{}\n", line));
                }
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result.pop();
    result.to_string()
}

// Converte blocos iniciados com ``` em valores de string, sem necessidade de !phs
fn preprocessor_markdown_string_blocks(phlow: &str) -> String {
    let mut result = String::new();
    let mut lines = phlow.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed_line = line.trim_start();

        // Caso: item de lista - ``` ... ``` (processar antes de propriedade para evitar confusão com ':')
        if trimmed_line.starts_with('-') {
            // Caso: item de lista - ``` ... ```
            let after_dash = trimmed_line[1..].trim_start();
            if after_dash.starts_with("```") {
                let indent = &line[..line.len() - trimmed_line.len()];
                let mut block_lines: Vec<String> = Vec::new();

                if after_dash == "```" {
                    while let Some(next_line) = lines.next() {
                        if next_line.trim() == "```" {
                            break;
                        }
                        block_lines.push(next_line.trim().to_string());
                    }
                } else if let Some(end_pos) = after_dash[3..].find("```") {
                    let mut inner = after_dash[3..3 + end_pos].trim().to_string();
                    if let Some(space_idx) = inner.find(' ') {
                        let (first, rest) = inner.split_at(space_idx);
                        if !first.is_empty() {
                            inner = rest.trim_start().to_string();
                        }
                    }
                    block_lines.push(inner);
                } else {
                    // Sem fechamento na mesma linha: continua lendo até ```
                    while let Some(next_line) = lines.next() {
                        if next_line.trim() == "```" {
                            break;
                        }
                        block_lines.push(next_line.trim().to_string());
                    }
                }

                let single_line = block_lines.join(" ");
                let escaped = single_line.replace('"', "\\\"");
                result.push_str(&format!("{}- \"{}\"\n", indent, escaped));
                continue;
            }
        } else if let Some(colon_pos) = trimmed_line.find(':') {
            // Caso: propriedade key: ``` ... ```
            let key = &trimmed_line[..colon_pos].trim();
            let value = trimmed_line[colon_pos + 1..].trim();
            if value.starts_with("```") {
                let indent = &line[..line.len() - trimmed_line.len()];
                let mut block_lines: Vec<String> = Vec::new();

                if value == "```" {
                    while let Some(next_line) = lines.next() {
                        if next_line.trim() == "```" {
                            break;
                        }
                        block_lines.push(next_line.trim().to_string());
                    }
                } else if let Some(end_pos) = value[3..].find("```") {
                    let mut inner = value[3..3 + end_pos].trim().to_string();
                    // Se houver rótulo de linguagem no início, remove-o
                    if let Some(space_idx) = inner.find(' ') {
                        let (first, rest) = inner.split_at(space_idx);
                        if !first.is_empty() {
                            inner = rest.trim_start().to_string();
                        }
                    }
                    block_lines.push(inner);
                } else {
                    // Sem fechamento na mesma linha: continua lendo até ```
                    while let Some(next_line) = lines.next() {
                        if next_line.trim() == "```" {
                            break;
                        }
                        block_lines.push(next_line.trim().to_string());
                    }
                }

                let single_line = block_lines.join(" ");
                let escaped = single_line.replace('"', "\\\"");
                result.push_str(&format!("{}{}: \"{}\"\n", indent, key, escaped));
                continue;
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result.pop();
    result
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
    // Para arquivos incluídos, só aplicamos preprocessor_directives para processar outros !include
    // mas não aplicamos modules, auto_phs e eval pois isso será feito no arquivo principal
    let (transformed, errors) = preprocessor_directives(&with_args, parent);

    if !errors.is_empty() {
        return Err(errors.join("; "));
    }

    Ok(transformed)
}

fn preprocessor_modules(phlow: &str) -> Result<String, Vec<String>> {
    // Pre-processa o YAML para escapar valores que começam com ! para evitar problemas de parsing
    let escaped_phlow = escape_yaml_exclamation_values(phlow);

    // Parse o YAML para extrair módulos disponíveis
    let parsed: Value = match serde_yaml::from_str(&escaped_phlow) {
        Ok(val) => val,
        Err(_) => return Ok(phlow.to_string()), // Se não conseguir parsear, retorna o original
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
        return Ok(phlow.to_string()); // Sem módulos para transformar
    }

    // Parse novamente para modificar, usando o YAML escapado
    let mut parsed_mut: Value = match serde_yaml::from_str(&escaped_phlow) {
        Ok(val) => val,
        Err(_) => return Ok(phlow.to_string()),
    };

    // Mantém uma cópia para detectar se houve alterações
    let original_parsed = parsed_mut.clone();

    transform_value(
        &mut parsed_mut,
        &available_modules,
        false, // Começa como false, só será true dentro de steps, then ou else
    );

    // Se nada mudou, preserva o YAML original (mantendo comentários e formatação)
    if parsed_mut == original_parsed {
        return Ok(phlow.to_string());
    }

    // Converte de volta para YAML e desfaz o escape
    match serde_yaml::to_string(&parsed_mut) {
        Ok(result) => Ok(unescape_yaml_exclamation_values(&result)),
        Err(_) => Ok(phlow.to_string()),
    }
}

const EXCLUSIVE_PROPERTIES: &[&str] = &[
    "use",
    "to",
    "id",
    "label",
    "assert",
    "assert_eq",
    "condition",
    "return",
    "payload",
    "input",
    "then",
    "else",
    "steps",
];

// Função recursiva para transformar o YAML
fn transform_value(
    value: &mut Value,
    available_modules: &std::collections::HashSet<String>,
    is_in_transformable_context: bool,
) {
    match value {
        Value::Mapping(map) => {
            // Collect pending transformations: (original key, original value, optional (action, args))
            let mut transformations: Vec<(Value, Value, Option<(String, Vec<String>)>)> =
                Vec::new();

            for (key, val) in map.iter() {
                if let Some(key_str) = key.as_str() {
                    // Só transforma se estiver em um contexto transformável (raiz de steps, then ou else)
                    if is_in_transformable_context {
                        // Verifica se a chave contém um ponto (module.action)
                        if key_str.contains('.') {
                            let parts: Vec<&str> = key_str.split('.').collect();
                            if parts.len() >= 2 {
                                let module_name = parts[0];
                                let action_name = parts[1].to_string();
                                let extra_args: Vec<String> =
                                    parts.iter().skip(2).map(|s| s.to_string()).collect();

                                // Verifica se não é uma propriedade exclusiva e se o módulo está disponível
                                if !EXCLUSIVE_PROPERTIES.contains(&module_name)
                                    && (available_modules.contains(module_name)
                                        || !available_modules.is_empty())
                                {
                                    transformations.push((
                                        key.clone(),
                                        val.clone(),
                                        Some((action_name, extra_args)),
                                    ));
                                }
                            }
                        } else {
                            // Se não é uma propriedade exclusiva e é um módulo disponível
                            if !EXCLUSIVE_PROPERTIES.contains(&key_str)
                                && available_modules.contains(key_str)
                            {
                                transformations.push((key.clone(), val.clone(), None));
                            }
                        }
                    }
                }
            }

            // Aplica as transformações
            for (key, old_val, action_and_args) in transformations {
                map.remove(&key);

                let mut new_entry = Mapping::new();

                // Extrai o nome do módulo (remove a ação se houver)
                let module_name = if let Some(key_str) = key.as_str() {
                    if key_str.contains('.') {
                        key_str.split('.').next().unwrap_or(key_str)
                    } else {
                        key_str
                    }
                } else {
                    ""
                };

                new_entry.insert(
                    Value::String("use".to_string()),
                    Value::String(module_name.to_string()),
                );

                // Cria o input com a ação como primeiro parâmetro, se houver
                let input_value = if let Some((action_name, args_vec)) = action_and_args {
                    // Se há uma ação, cria um novo mapeamento com action como primeiro item
                    if let Value::Mapping(old_map) = old_val {
                        let mut new_input = Mapping::new();
                        new_input.insert(
                            Value::String("action".to_string()),
                            Value::String(action_name),
                        );

                        // Adiciona os argumentos extras, se houver
                        if !args_vec.is_empty() {
                            let args_seq = Value::Sequence(
                                args_vec
                                    .into_iter()
                                    .map(Value::String)
                                    .collect::<Vec<Value>>(),
                            );
                            new_input.insert(Value::String("args".to_string()), args_seq);
                        }

                        // Adiciona os outros parâmetros depois da ação
                        for (old_key, old_value) in old_map.iter() {
                            new_input.insert(old_key.clone(), old_value.clone());
                        }

                        Value::Mapping(new_input)
                    } else {
                        // Se old_val não é um mapeamento, cria um novo com apenas a ação
                        let mut new_input = Mapping::new();
                        new_input.insert(
                            Value::String("action".to_string()),
                            Value::String(action_name),
                        );

                        if !args_vec.is_empty() {
                            let args_seq = Value::Sequence(
                                args_vec
                                    .into_iter()
                                    .map(Value::String)
                                    .collect::<Vec<Value>>(),
                            );
                            new_input.insert(Value::String("args".to_string()), args_seq);
                        }
                        Value::Mapping(new_input)
                    }
                } else {
                    // Se não há ação, usa o valor original
                    old_val
                };

                new_entry.insert(Value::String("input".to_string()), input_value);

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

                transform_value(val, available_modules, next_is_transformable);
            }
        }
        Value::Sequence(seq) => {
            for item in seq.iter_mut() {
                transform_value(item, available_modules, is_in_transformable_context);
            }
        }
        _ => {}
    }
}

// Função para escapar valores que começam com ! para evitar interpretação como tags YAML
fn escape_yaml_exclamation_values(yaml: &str) -> String {
    let regex = match Regex::new(r"((?::\s*|-\s+\w+:\s*))(!\w.*?)\s*$") {
        Ok(re) => re,
        Err(_) => return yaml.to_string(),
    };

    let result = regex
        .replace_all(yaml, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let exclamation_value = &caps[2];
            format!(r#"{} "__PHLOW_ESCAPE__{}""#, prefix, exclamation_value)
        })
        .to_string();

    result
}

// Função para desfazer o escape dos valores com !
fn unescape_yaml_exclamation_values(yaml: &str) -> String {
    let regex = match Regex::new(r"__PHLOW_ESCAPE__(!\w[^\s]*)") {
        Ok(re) => re,
        Err(_) => return yaml.to_string(),
    };

    let result = regex
        .replace_all(yaml, |caps: &regex::Captures| {
            let exclamation_value = &caps[1];
            exclamation_value.to_string()
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessor_transform_phs_hidden_object_and_arrays() {
        let input = r#"
        key1: {
            "name": "value",
            "list": [1, 2, 3]
        }
        key2: normal_value
        "#;

        let transformed = processor_transform_phs_hidden_object_and_arrays(input);

        assert!(
            transformed.contains("key1: !phs ${ { \"name\": \"value\", \"list\": [1, 2, 3] } }")
        );
        assert!(!transformed.contains("key2: !phs normal_value"));
    }

    #[test]
    fn test_preprocessor_transform_phs_hidden() {
        let input = r#"
        key1: if condition { do_something() }
        key2: "normal string"
        - for item in list { process(item) }
        "#;

        let transformed = preprocessor_transform_phs_hidden(input);
        assert!(transformed.contains("key1: !phs if condition { do_something() }"));
        assert!(transformed.contains("- !phs for item in list { process(item) }"));
    }

    #[test]
    fn test_preprocessor_eval() {
        let input = r#"
        key1: !phs if condition { do_something() }
        key2: !phs ```
        multi_line_code();
        another_line();
        ```
        key3: !phs ${ for item in list { process(item) } }
        "#;
        let transformed = preprocessor_eval(input);
        assert!(transformed.contains("key1: \"{{ if condition { do_something() } }}\""));
        // Blocos com ``` devem ser tratados como strings simples
        assert!(transformed.contains("key2: \"multi_line_code(); another_line();\""));
        assert!(transformed.contains("key3: \"{{ for item in list { process(item) } }}\""));
    }

    #[test]
    fn test_preprocessor_modules() {
        let input = r#"
        modules:
          - module: test_module

        steps:
          - test_module:
              param1: value1
              param2: value2
          - another_step:
              action: do_something
          - new_module.my_action:
              paramA: valueA
        "#;

        let expected = r#"modules:
- module: test_module
steps:
- use: test_module
  input:
    param1: value1
    param2: value2
- another_step:
    action: do_something
- use: new_module
  input:
    action: my_action
    paramA: valueA
"#;

        let transformed = preprocessor_modules(input).unwrap();
        println!("Transformed:\n{}", transformed);
        assert_eq!(transformed, expected);
    }

    #[test]
    fn test_preprocessor_modules_with_action_args() {
        let input = r#"
        modules:
          - module: test_module

        steps:
          - test_module.my_action.info.data:
              param1: value1
        "#;

        let expected = r#"modules:
- module: test_module
steps:
- use: test_module
  input:
    action: my_action
    args:
    - info
    - data
    param1: value1
"#;

        let transformed = preprocessor_modules(input).unwrap();
        println!("Transformed with args:\n{}", transformed);
        assert_eq!(transformed, expected);
    }

    #[test]
    fn test_preprocessor_eval_triple_backtick_blocks_as_string() {
        // Multilinha com abertura/fechamento em linhas separadas
        let input_multiline = r#"
        key_md_block: !phs ```
        first_line();
        second_line();
        ```
        "#;
        let transformed_multiline = preprocessor_eval(input_multiline);
        assert!(transformed_multiline.contains("key_md_block: \"first_line(); second_line();\""));

        // Inline na mesma linha
        let input_inline = r#"
        key_inline_block: !phs ```single_line();```
        "#;
        let transformed_inline = preprocessor_eval(input_inline);
        assert!(transformed_inline.contains("key_inline_block: \"single_line();\""));

        // Inline com linguagem
        let input_inline_lang = r#"
        key_inline_lang: !phs ```json {"a":1}```
        "#;
        let transformed_inline_lang = preprocessor_eval(input_inline_lang);
        assert!(transformed_inline_lang.contains("key_inline_lang: \"{\\\"a\\\":1}\""));

        // Item de lista com bloco markdown
        let input_list = r#"
        - !phs ```
        a();
        b();
        ```
        "#;
        let transformed_list = preprocessor_eval(input_list);
        assert!(transformed_list.contains("- \"a(); b();\""));
    }

    #[test]
    fn test_preprocessor_markdown_string_blocks_with_language_labels() {
        // Propriedade com linguagem e conteúdo inline
        let input_prop_inline = r#"
        prop: ```js doThing();```
        "#;
        let out_prop_inline = preprocessor_markdown_string_blocks(input_prop_inline);
        assert!(out_prop_inline.contains("prop: \"doThing();\""));

        // Propriedade multilinha com linguagem
        let input_prop_multiline = r#"
        prompt: ```md
        Hello
        World
        ```
        "#;
        let out_prop_multi = preprocessor_markdown_string_blocks(input_prop_multiline);
        assert!(out_prop_multi.contains("prompt: \"Hello World\""));

        // Item de lista inline com linguagem
        let input_list_inline = r#"
        - ```json {"x":2}```
        "#;
        let out_list_inline = preprocessor_markdown_string_blocks(input_list_inline);
        println!("out_list_inline=<<<{}>>>", out_list_inline);
        assert!(out_list_inline.contains("- \"{\\\"x\\\":2}\""));

        // Item de lista multilinha com linguagem
        let input_list_multi = r#"
        - ```sql
        select 1;
        select 2;
        ```
        "#;
        let out_list_multi = preprocessor_markdown_string_blocks(input_list_multi);
        assert!(out_list_multi.contains("- \"select 1; select 2;\""));
    }

    fn temporary_included_file() -> std::io::Result<()> {
        let content = r#"
        {
            "included_key1": "!arg arg1",
            "included_key2": "!arg arg2"
        }
        "#;

        fs::write("included_file.phlow", content)
    }

    fn remove_temporary_included_file() -> std::io::Result<()> {
        fs::remove_file("included_file.phlow")
    }

    fn temporary_included_inline_file() -> std::io::Result<()> {
        let content = r#"{
    "a": 1,
    "b": 2
}"#;
        fs::write("included_inline.phlow", content)
    }

    fn remove_temporary_included_inline_file() -> std::io::Result<()> {
        fs::remove_file("included_inline.phlow")
    }

    #[test]
    fn test_preprocessor() {
        // create temporay included_file.phlow
        temporary_included_file().unwrap();

        let input = r#"
        !include included_file.phlow arg1='value1' arg2="value2"

        key1: if condition { do_something() }
        key2: {
            "name": "value",
            "list": [1, 2, 3]
        }
        key3: ```
        multi_line_code();
        another_line();
        ```
        modules:
          - module: test_module

        steps:
          - test_module:
              param1: value1
              param2: value2
        "#;

        let expected: &str = r#"
        

                {

                    "included_key1": "value1",

                    "included_key2": "value2"

                }

                

        key1: "{{ if condition { do_something() } }}"
        key2: "{{ { \"name\": \"value\", \"list\": [1, 2, 3] } }}"
        key3: "multi_line_code(); another_line();"
        modules:
          - module: test_module

        steps:
          - test_module:
              param1: value1
              param2: value2
        "#;

        let processed = preprocessor(input, &Path::new(".").to_path_buf(), false).unwrap();
        println!("Processed:\n{}", processed);

        assert_eq!(processed, expected);

        remove_temporary_included_file().unwrap();
    }

    #[test]
    fn test_preprocessor_directives_inline_include_indentation_on_mapping_value() {
        temporary_included_inline_file().unwrap();

        let input = "  key: !include included_inline.phlow";
        let (result, errors) = preprocessor_directives(input, Path::new("."));

        assert!(errors.is_empty(), "Errors found: {:?}", errors);

        // Espera-se que as quebras de linha do conteúdo incluído recebam a mesma
        // tabulação do prefixo até o !include (no caso, 2 espaços + 'key: ' = 7 espaços)
        // Observação: o conteúdo incluído possui indentação própria (4 espaços). Como
        // preservamos a tabulação do include E a indentação interna do conteúdo,
        // as linhas internas ficam com (tabulação_do_prefixo + indentação_do_conteúdo) = 7 + 4 = 11 espaços.
        let expected = "  key: {\n           \"a\": 1,\n           \"b\": 2\n       }";

        assert_eq!(
            result, expected,
            "Inline include indentation mismatch.\nGot:\n<<<{}>>>\nExpected:\n<<<{}>>>",
            result, expected
        );

        remove_temporary_included_inline_file().unwrap();
    }

    #[test]
    fn test_no_phs() {
        let input = r#"modules:
  - module: log
  - module: fs
  - module: openai
    with:
      api_key: envs.OPENAI_API_KEY
  - module: amqp
    with:
      vhost: "nixyz"
      queue_name: "queue.etl.carrefour.raw"
      max_concurrency: 1
      definition:
        vhosts:
          - name: "nixyz"
        exchanges:
          - name: "x.etl"
            type: direct
            durable: true
            vhost: "nixyz"
            auto_delete: true
        queues:
          - name: "queue.etl.carrefour.raw"
            vhost: "nixyz"
            durable: true
        bindings:
          - source: "x.extract"
            vhost: "nixyz"
            destination: "queue.etl.carrefour.raw"
            destination_type: queue
            routing_key: "mercado.carrefour.com.br.#"
            arguments: {}
  - module: aws
    with:
      region: envs.AWS_REGION
      endpoint_url: envs.AWS_S3_ENDPOINT_URL
      s3_force_path_style: true
      secret_access_key: envs.AWS_SECRET_ACCESS_KEY
      access_key_id: envs.AWS_ACCESS_KEY_ID

main: amqp

steps:
  - log.info:
      message: "Starting processing message"

  - payload: main.parse()

  - log.info:
      message: payload

  - id: fetch_s3_object
    aws.s3.get_object:
      bucket: payload.bucket
      key: payload.key

  - assert: payload.success != true
    then:
      - log.error:
          message: payload.error
      - return: false

  - log.info:
      message: "Fetched object from S3"

  - fs.write:
      path: ./input.json
      content: steps.fetch_s3_object
      force: true
  
  - id: gpt
    openai.chat:
      model: "gpt-5-nano"
      messages:
        - role: user
          content: "\# Product Extraction Prompt with JSON Schema\n\nAnalyze an input HTML page, extract all relevant product information, and return this data as a single object strictly following the provided JSON schema for a \"Product\" as below.\n\n## JSON Schema for \"Product\"\n\n```json\n{\n  \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n  \"title\": \"Product\",\n  \"type\": \"object\",\n  \"additionalProperties\": false,\n  \"properties\": {\n    \"id\": { \"type\": \"string\" },\n    \"sku\": { \"type\": \"string\" },\n    \"name\": { \"type\": \"string\" },\n    \"description\": { \"type\": \"string\" },\n    \"price\": { \"type\": \"number\" },\n    \"originalPrice\": { \"type\": \"number\" },\n    \"currency\": { \"type\": \"string\" },\n    \"stock\": { \"type\": \"integer\" },\n    \"availability\": { \"type\": \"string\" },\n    \"brand\": { \"type\": \"string\" },\n    \"category\": { \"type\": \"string\" },\n    \"categories\": { \"type\": \"array\", \"items\": { \"type\": \"string\" } },\n    \"tags\": { \"type\": \"array\", \"items\": { \"type\": \"string\" } },\n    \"images\": {\n      \"type\": \"array\",\n      \"items\": {\n        \"type\": \"object\",\n        \"additionalProperties\": false,\n        \"properties\": {\n          \"url\": { \"type\": \"string\", \"format\": \"uri\" },\n          \"alt\": { \"type\": \"string\" }\n        }\n      }\n    },\n    \"videos\": {\n      \"type\": \"array\",\n      \"items\": {\n        \"type\": \"object\",\n        \"additionalProperties\": false,\n        \"properties\": {\n          \"url\": { \"type\": \"string\", \"format\": \"uri\" },\n          \"title\": { \"type\": \"string\" }\n        }\n      }\n    },\n    \"attributes\": {\n      \"type\": \"array\",\n      \"items\": {\n        \"type\": \"object\",\n        \"additionalProperties\": false,\n        \"properties\": {\n          \"name\": { \"type\": \"string\" },\n          \"value\": { \"type\": [\"string\", \"number\", \"boolean\"] }\n        }\n      }\n    },\n    \"variants\": {\n      \"type\": \"array\",\n      \"items\": {\n        \"type\": \"object\",\n        \"additionalProperties\": false,\n        \"properties\": {\n          \"id\": { \"type\": \"string\" },\n          \"sku\": { \"type\": \"string\" },\n          \"name\": { \"type\": \"string\" },\n          \"price\": { \"type\": \"number\" },\n          \"originalPrice\": { \"type\": \"number\" },\n          \"currency\": { \"type\": \"string\" },\n          \"stock\": { \"type\": \"integer\" },\n          \"attributes\": {\n            \"type\": \"array\",\n            \"items\": {\n              \"type\": \"object\",\n              \"additionalProperties\": false,\n              \"properties\": {\n                \"name\": { \"type\": \"string\" },\n                \"value\": { \"type\": [\"string\", \"number\", \"boolean\"] }\n              }\n            }\n          }\n        }\n      }\n    },\n    \"rating\": { \"type\": \"number\" },\n    \"reviewCount\": { \"type\": \"integer\" },\n    \"gtin\": { \"type\": \"string\" },\n    \"mpn\": { \"type\": \"string\" },\n    \"url\": { \"type\": \"string\", \"format\": \"uri\" },\n    \"language\": { \"type\": \"string\" },\n    \"currencySymbol\": { \"type\": \"string\" },\n    \"breadcrumbs\": { \"type\": \"array\", \"items\": { \"type\": \"string\" } },\n    \"createdAt\": { \"type\": \"string\", \"format\": \"date-time\" },\n    \"updatedAt\": { \"type\": \"string\", \"format\": \"date-time\" },\n    \"metadata\": {\n      \"type\": \"object\",\n      \"additionalProperties\": {\n        \"type\": [\"string\", \"number\", \"boolean\", \"null\"]\n      }\n    }\n  }\n}\n```\n\n## Extraction Instructions\n\nCarefully identify and map as much product information as possible from the HTML content to the respective JSON fields.  \nNormalize price, numbers, dates; do not hallucinate; omit missing values; output only one product.\n\n## Reasoning Requirements\n\nBefore outputting JSON, reason step-by-step about:  \n- How each field can be detected  \n- What normalization is needed  \n- What cannot be extracted  \n\n## Output Format\n\nReturn only **one JSON object**, no commentary or code block.\n"
        - role: user
          content: steps.fetch_s3_object.data

  - fs.write:
      path: ./payload.json
      content: payload
      force: true

  - log.info:
      message: steps.gpt.data.choices[0].message.content"#;

        let transformed = preprocessor(input, &Path::new(".").to_path_buf(), false).unwrap();

        assert!(!transformed.contains(r#"#{ \"type\": \"string\" }"#));
    }
}
