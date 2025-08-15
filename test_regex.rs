fn main() {
    let text = r#"  - assert: main == 1
  - payload: main + 1  
  - return: payload * 2"#;

    let regex = regex::Regex::new(r"(?m)^(\s*)(assert):\s*([^!\s][^\n]*)$").unwrap();
    
    println!("Testing regex...");
    for caps in regex.captures_iter(text) {
        println!("Match: indent='{}', property='{}', value='{}'", 
                 &caps[1], &caps[2], &caps[3]);
    }
    
    let result = regex.replace_all(text, |caps: &regex::Captures| {
        format!("{}{}:  !phs {}", &caps[1], &caps[2], &caps[3])
    });
    
    println!("\nResult:\n{}", result);
}
