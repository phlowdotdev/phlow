use crossterm::{
    cursor::{Hide, Show},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseEventKind,
    },
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use serde_json::Value as JsonValue;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

const DEFAULT_DEBUG_PORT: u16 = 31400;
const TIMEOUT_MESSAGE: &str = "Timed out waiting for response";

struct AppState {
    output: String,
    input: String,
    scroll_from_bottom: usize,
    search_term: Option<String>,
    search_index: usize,
    history: Vec<String>,
    dropdown_open: bool,
    dropdown_index: usize,
    dropdown_suppressed: bool,
    timeline_steps: Vec<String>,
    current_step_uuid: Option<String>,
    status_notice: Option<String>,
    show_help: bool,
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState {
        output: "phlow-tui-inspect: waiting for command".to_string(),
        input: String::new(),
        scroll_from_bottom: 0,
        search_term: None,
        search_index: 0,
        history: Vec::new(),
        dropdown_open: false,
        dropdown_index: 0,
        dropdown_suppressed: false,
        timeline_steps: Vec::new(),
        current_step_uuid: None,
        status_notice: None,
        show_help: true,
    };

    let initial_output = run_show();
    apply_output_show(&mut state, initial_output);

    let result = run_app(&mut terminal, &mut state);

    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        Show,
        LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
) -> io::Result<()> {
    render(terminal, state)?;

    loop {
        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key(key, state) {
                        render(terminal, state)?;
                        continue;
                    }

                    if should_exit(key) {
                        break;
                    }

                    render(terminal, state)?;
                }
                Event::Mouse(event) => {
                    if handle_mouse(event.kind, state) {
                        render(terminal, state)?;
                    }
                }
                Event::Resize(_, _) => {
                    render(terminal, state)?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn should_exit(key: KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } | KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}

fn handle_key(key: KeyEvent, state: &mut AppState) -> bool {
    if state.show_help {
        if matches!(
            key,
            KeyEvent {
                code: KeyCode::Esc,
                ..
            }
        ) {
            state.show_help = false;
            return true;
        }
        return false;
    }

    match key {
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            if state.dropdown_open {
                if let Some(suggestion) = selected_suggestion(state) {
                    let selection = suggestion.trim_end();
                    if !selection.is_empty() {
                        execute_selected_command(state, selection);
                    }
                    state.dropdown_open = false;
                    state.dropdown_suppressed = false;
                    return true;
                }
            }
            let trimmed = state.input.trim().to_string();
            if !trimmed.is_empty() {
                if trimmed.starts_with('/') {
                    handle_slash_command(&trimmed, state);
                    state.input.clear();
                    on_input_changed(state);
                    return true;
                }

                record_history(state, &trimmed);
                if trimmed.eq_ignore_ascii_case("show") {
                    apply_output_show(state, run_command(&trimmed));
                } else {
                    apply_output(state, run_command(&trimmed));
                }
                state.input.clear();
                on_input_changed(state);
            }
            true
        }
        KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            record_history(state, "/n");
            apply_output(state, run_next_and_step());
            true
        }
        KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            record_history(state, "/a");
            apply_output(state, run_next_and_all());
            true
        }
        KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            record_history(state, "/r");
            apply_output(state, run_release_and_all());
            true
        }
        KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            record_history(state, "/w");
            apply_output_show(state, run_show());
            true
        }
        KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            record_history(state, "/g");
            apply_output(state, run_command("STEP"));
            true
        }
        KeyEvent {
            code: KeyCode::Char('m'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            record_history(state, "/m");
            state.show_help = true;
            state.dropdown_open = false;
            true
        }
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            record_history(state, "/d");
            move_search(state, -1);
            true
        }
        KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            record_history(state, "/e");
            move_search(state, 1);
            true
        }
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            record_history(state, "/x");
            clear_search(state);
            true
        }
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            state.input.pop();
            on_input_changed(state);
            true
        }
        KeyEvent {
            code: KeyCode::Up,
            ..
        } => {
            navigate_dropdown(state, -1);
            true
        }
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            navigate_dropdown(state, 1);
            true
        }
        KeyEvent {
            code: KeyCode::PageUp,
            ..
        } => {
            scroll_by(state, page_scroll(state));
            true
        }
        KeyEvent {
            code: KeyCode::PageDown,
            ..
        } => {
            scroll_by(state, -page_scroll(state));
            true
        }
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            if state.dropdown_open {
                state.dropdown_open = false;
                state.dropdown_suppressed = true;
                return true;
            }
            false
        }
        KeyEvent {
            code: KeyCode::Home,
            ..
        } => {
            state.scroll_from_bottom = usize::MAX;
            true
        }
        KeyEvent {
            code: KeyCode::End,
            ..
        } => {
            state.scroll_from_bottom = 0;
            true
        }
        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            ..
        } => {
            state.input.push(ch);
            on_input_changed(state);
            true
        }
        _ => false,
    }
}

fn handle_slash_command(command: &str, state: &mut AppState) {
    handle_slash_command_internal(command, state, true);
}

fn handle_slash_command_internal(command: &str, state: &mut AppState, record: bool) {
    let mut parts = command.splitn(2, char::is_whitespace);
    let cmd = parts.next().unwrap_or("").trim();
    let arg = parts.next().unwrap_or("").trim();

    match cmd {
        "/n" => {
            if record {
                record_history(state, command);
            }
            apply_output(state, run_next_and_step());
        }
        "/a" => {
            if record {
                record_history(state, command);
            }
            apply_output(state, run_next_and_all());
        }
        "/r" => {
            if record {
                record_history(state, command);
            }
            apply_output(state, run_release_and_all());
        }
        "/w" => {
            if record {
                record_history(state, command);
            }
            apply_output_show(state, run_show());
        }
        "/g" => {
            if record {
                record_history(state, command);
            }
            apply_output(state, run_command("STEP"));
        }
        "/m" => {
            if record {
                record_history(state, command);
            }
            state.show_help = true;
            state.dropdown_open = false;
        }
        "/d" => {
            if record {
                record_history(state, command);
            }
            move_search(state, -1);
        }
        "/e" => {
            if record {
                record_history(state, command);
            }
            move_search(state, 1);
        }
        "/x" => {
            if record {
                record_history(state, command);
            }
            clear_search(state);
        }
        "/s" => {
            if arg.is_empty() {
                clear_search(state);
            } else {
                if record {
                    record_history(state, command);
                }
                state.search_term = Some(arg.to_string());
                state.search_index = 0;
                scroll_to_current_search(state);
            }
        }
        _ => set_output(state, "Unknown command".to_string()),
    }
}

fn set_output(state: &mut AppState, output: String) {
    state.output = output;
    state.scroll_from_bottom = 0;
    state.dropdown_open = false;
    state.status_notice = None;
    update_current_step_from_output(state);
}

fn set_output_show(state: &mut AppState, output: String) {
    set_output(state, output);
    update_timeline_from_show(state);
}

fn apply_output(state: &mut AppState, output: String) {
    if let Some(notice) = notice_from_output(&output) {
        state.status_notice = Some(notice);
        state.dropdown_open = false;
        return;
    }
    set_output(state, output);
}

fn apply_output_show(state: &mut AppState, output: String) {
    if let Some(notice) = notice_from_output(&output) {
        state.status_notice = Some(notice);
        state.dropdown_open = false;
        return;
    }
    set_output_show(state, output);
}

fn update_current_step_from_output(state: &mut AppState) {
    let Ok(json) = serde_json::from_str::<JsonValue>(&state.output) else {
        return;
    };
    if let Some(uuid) = extract_step_uuid(&json) {
        state.current_step_uuid = Some(uuid);
    }
}

fn notice_from_output(output: &str) -> Option<String> {
    if output == TIMEOUT_MESSAGE {
        return Some(output.to_string());
    }
    let Ok(json) = serde_json::from_str::<JsonValue>(output) else {
        return None;
    };
    let ok = json.get("ok").and_then(JsonValue::as_bool);
    if ok == Some(false) {
        let message = json
            .get("error")
            .and_then(JsonValue::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("error");
        return Some(message.to_string());
    }
    None
}

fn execute_selected_command(state: &mut AppState, selection: &str) {
    record_history(state, selection);
    if selection.starts_with('/') {
        handle_slash_command_internal(selection, state, false);
    } else if selection.eq_ignore_ascii_case("show") {
        apply_output_show(state, run_command(selection));
    } else {
        apply_output(state, run_command(selection));
    }
    state.input.clear();
    on_input_changed(state);
}

fn extract_step_uuid(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::Object(map) => map
            .get("step")
            .and_then(|step| step.get("#uuid"))
            .and_then(JsonValue::as_str)
            .map(|uuid| uuid.to_string()),
        JsonValue::Array(items) => items
            .iter()
            .rev()
            .find_map(|item| extract_step_uuid(item)),
        _ => None,
    }
}

fn update_timeline_from_show(state: &mut AppState) {
    let Ok(json) = serde_json::from_str::<JsonValue>(&state.output) else {
        return;
    };
    let mut steps = Vec::new();
    collect_steps_from_script(&json, &mut steps);
    state.timeline_steps = steps;
}

fn collect_steps_from_script(value: &JsonValue, out: &mut Vec<String>) {
    match value {
        JsonValue::Object(map) => {
            if let Some(steps) = map.get("steps") {
                collect_steps_from_array(steps, out);
            } else if map.get("#uuid").is_some() {
                collect_step_object(value, out);
            } else {
                if let Some(then_value) = map.get("then") {
                    collect_steps_from_script(then_value, out);
                }
                if let Some(else_value) = map.get("else") {
                    collect_steps_from_script(else_value, out);
                }
            }
        }
        JsonValue::Array(_) => {
            collect_steps_from_array(value, out);
        }
        _ => {}
    }
}

fn collect_steps_from_array(value: &JsonValue, out: &mut Vec<String>) {
    if let JsonValue::Array(items) = value {
        for item in items {
            collect_step_object(item, out);
        }
    }
}

fn collect_step_object(value: &JsonValue, out: &mut Vec<String>) {
    let Some(map) = value.as_object() else {
        return;
    };
    if let Some(uuid) = map
        .get("#uuid")
        .and_then(JsonValue::as_str)
        .map(|uuid| uuid.to_string())
    {
        out.push(uuid);
    }
    if let Some(then_value) = map.get("then") {
        collect_steps_from_script(then_value, out);
    }
    if let Some(else_value) = map.get("else") {
        collect_steps_from_script(else_value, out);
    }
    if let Some(steps) = map.get("steps") {
        collect_steps_from_array(steps, out);
    }
}

fn clear_search(state: &mut AppState) {
    state.search_term = None;
    state.search_index = 0;
}

fn scroll_to_current_search(state: &mut AppState) {
    let Some(term) = state.search_term.as_deref() else {
        return;
    };
    if term.is_empty() {
        return;
    }
    let Ok((width, height)) = terminal::size() else {
        return;
    };
    let output_height = output_height_for_size(state, width, height);
    let lines = wrap_text_lines(&state.output, width.saturating_sub(2) as usize);
    let matches = find_match_lines(&lines, term);
    if matches.is_empty() {
        return;
    }
    let index = state.search_index.min(matches.len().saturating_sub(1));
    state.search_index = index;
    scroll_to_line(
        state,
        matches[index],
        lines.len(),
        output_height,
    );
}

fn move_search(state: &mut AppState, delta: isize) {
    let Some(term) = state.search_term.as_deref() else {
        return;
    };
    if term.is_empty() {
        return;
    }
    let Ok((width, height)) = terminal::size() else {
        return;
    };
    let output_height = output_height_for_size(state, width, height);
    let lines = wrap_text_lines(&state.output, width.saturating_sub(2) as usize);
    let matches = find_match_lines(&lines, term);
    if matches.is_empty() {
        return;
    }
    let len = matches.len() as isize;
    let mut index = state.search_index as isize + delta;
    if index < 0 {
        index = len - 1;
    } else if index >= len {
        index = 0;
    }
    state.search_index = index as usize;
    scroll_to_line(
        state,
        matches[state.search_index],
        lines.len(),
        output_height,
    );
}

fn scroll_to_line(
    state: &mut AppState,
    line_index: usize,
    total_lines: usize,
    output_height: usize,
) {
    if total_lines <= output_height || output_height == 0 {
        state.scroll_from_bottom = 0;
        return;
    }
    let half = output_height / 2;
    let mut start = line_index.saturating_sub(half);
    let max_start = total_lines.saturating_sub(output_height);
    if start > max_start {
        start = max_start;
    }
    state.scroll_from_bottom = total_lines.saturating_sub(output_height + start);
}

fn handle_mouse(kind: MouseEventKind, state: &mut AppState) -> bool {
    match kind {
        MouseEventKind::ScrollUp => {
            scroll_by(state, 2);
            true
        }
        MouseEventKind::ScrollDown => {
            scroll_by(state, -2);
            true
        }
        _ => false,
    }
}

fn scroll_by(state: &mut AppState, delta: isize) {
    if delta > 0 {
        state.scroll_from_bottom = state.scroll_from_bottom.saturating_add(delta as usize);
    } else if delta < 0 {
        state.scroll_from_bottom = state
            .scroll_from_bottom
            .saturating_sub(delta.unsigned_abs() as usize);
    }
}

fn page_scroll(state: &AppState) -> isize {
    match terminal::size() {
        Ok((width, height)) => output_height_for_size(state, width, height) as isize,
        Err(_) => 10,
    }
}

fn run_command(command: &str) -> String {
    match send_command(command) {
        Ok(response) => format_response(&response),
        Err(err) => format!("Error: {}", err),
    }
}

fn run_next_and_step() -> String {
    match send_command("NEXT") {
        Ok(_) => {}
        Err(err) => return format!("Error: {}", err),
    }

    poll_for_response("STEP", |resp| !is_no_step_waiting(resp))
}

fn run_release_and_all() -> String {
    match send_command("RELEASE") {
        Ok(_) => {}
        Err(err) => return format!("Error: {}", err),
    }

    poll_for_response("ALL", |_| true)
}

fn run_show() -> String {
    run_command("SHOW")
}

fn run_next_and_all() -> String {
    match send_command("NEXT") {
        Ok(_) => {}
        Err(err) => return format!("Error: {}", err),
    }

    poll_for_response("ALL", |_| true)
}

fn poll_for_response(command: &str, accept: impl Fn(&str) -> bool) -> String {
    let attempts = 20;
    let delay = Duration::from_millis(50);

    for _ in 0..attempts {
        match send_command(command) {
            Ok(response) => {
                if accept(&response) {
                    return format_response(&response);
                }
            }
            Err(err) => return format!("Error: {}", err),
        }
        thread::sleep(delay);
    }

    "Timed out waiting for response".to_string()
}

fn is_no_step_waiting(response: &str) -> bool {
    if let Ok(json) = serde_json::from_str::<JsonValue>(response) {
        let ok = json.get("ok").and_then(JsonValue::as_bool);
        let err = json.get("error").and_then(JsonValue::as_str);
        return ok == Some(false) && err == Some("no step waiting");
    }
    false
}

fn send_command(command: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect(debug_addr())
        .map_err(|err| format!("connect failed: {}", err))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| format!("timeout failed: {}", err))?;

    stream
        .write_all(command.as_bytes())
        .map_err(|err| format!("write failed: {}", err))?;
    stream
        .write_all(b"\n")
        .map_err(|err| format!("write failed: {}", err))?;
    stream.flush().map_err(|err| format!("flush failed: {}", err))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|err| format!("read failed: {}", err))?;
    Ok(response.trim_end().to_string())
}

fn debug_addr() -> String {
    let port = std::env::var("PHLOW_DEBUG_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_DEBUG_PORT);
    format!("127.0.0.1:{}", port)
}

fn format_response(response: &str) -> String {
    let parsed = serde_json::from_str::<JsonValue>(response);
    match parsed {
        Ok(value) => serde_json::to_string_pretty(&value).unwrap_or_else(|_| response.to_string()),
        Err(_) => response.to_string(),
    }
}

fn render(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &AppState,
) -> io::Result<()> {
    terminal.draw(|frame| {
        let size = frame.area();
        let summary_width = size.width as usize;
        let summary_lines = wrap_text_lines(summary_text(), summary_width);
        let summary_height = summary_height_for_width(size.width);
        let timeline_width = size.width as usize;
        let timeline_lines = build_timeline_lines(state, timeline_width);
        let timeline_height = timeline_height_for_width(&timeline_lines);
        let input_height = 3u16;
        let dropdown_height =
            dropdown_height_for_state(state, size.height, summary_height, timeline_height);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(timeline_height),
                Constraint::Min(1),
                Constraint::Length(summary_height),
                Constraint::Length(dropdown_height),
                Constraint::Length(input_height),
            ])
            .split(size);

        if timeline_height > 0 {
            let timeline_text = Text::from(timeline_lines);
            let timeline_widget =
                Paragraph::new(timeline_text).wrap(Wrap { trim: false });
            frame.render_widget(timeline_widget, chunks[0]);
        }

        let output_width = chunks[1].width.saturating_sub(2) as usize;
        let output_lines =
            build_output_lines(&state.output, state.search_term.as_deref(), output_width);
        let output_height = chunks[1].height.saturating_sub(2).max(1) as usize;
        let max_scroll = output_lines.len().saturating_sub(output_height);
        let scroll = state.scroll_from_bottom.min(max_scroll);
        let start = output_lines.len().saturating_sub(output_height + scroll);
        let scroll_offset = start.min(u16::MAX as usize) as u16;

        let output_text = Text::from(output_lines);
        let output_widget = Paragraph::new(output_text)
            .block(Block::default().borders(Borders::ALL))
            .scroll((scroll_offset, 0));
        frame.render_widget(output_widget, chunks[1]);

        let summary_text = Text::from(summary_lines.join("\n"));
        let summary_widget =
            Paragraph::new(summary_text).wrap(Wrap { trim: false });
        frame.render_widget(summary_widget, chunks[2]);

        let input_line = format!("> {}", state.input);
        let input_widget =
            Paragraph::new(input_line).block(Block::default().borders(Borders::ALL));
        if dropdown_height > 0 {
            let dropdown_candidates = dropdown_candidates(state);
            if !dropdown_candidates.is_empty() {
                let visible = dropdown_height.saturating_sub(2) as usize;
                let (start, end) =
                    dropdown_window(dropdown_candidates.len(), state.dropdown_index, visible);
                let items: Vec<ListItem> = dropdown_candidates[start..end]
                    .iter()
                    .map(|item| ListItem::new(item.clone()))
                    .collect();
                let mut list_state = ListState::default();
                if state.dropdown_index < dropdown_candidates.len() {
                    list_state.select(Some(state.dropdown_index.saturating_sub(start)));
                }
                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::LightBlue)
                            .add_modifier(Modifier::BOLD),
                    );
                frame.render_stateful_widget(list, chunks[3], &mut list_state);
            }
        }

        frame.render_widget(input_widget, chunks[4]);

        let inner_x = chunks[4].x.saturating_add(1);
        let inner_y = chunks[4].y.saturating_add(1);
        let inner_width = chunks[4].width.saturating_sub(2);
        let cursor_offset = 2 + state.input.chars().count() as u16;
        let cursor_x = inner_x.saturating_add(cursor_offset);
        let max_x = inner_x.saturating_add(inner_width.saturating_sub(1));
        frame.set_cursor_position((cursor_x.min(max_x), inner_y));

        if state.show_help {
            let help_text = help_text();
            let max_width = size.width.saturating_sub(6).max(10) as usize;
            let help_lines = wrap_text_lines(help_text, max_width);
            let content_width = help_lines
                .iter()
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(1);
            let modal_width = (content_width as u16 + 2)
                .min(size.width.saturating_sub(4))
                .max(10);
            let modal_height = (help_lines.len() as u16 + 2)
                .min(size.height.saturating_sub(2))
                .max(3);
            let area = centered_rect(modal_width, modal_height, size);
            frame.render_widget(Clear, area);
            let modal_text = Text::from(help_lines.join("\n"));
            let modal_widget = Paragraph::new(modal_text)
                .block(Block::default().borders(Borders::ALL).title("Summary"))
                .wrap(Wrap { trim: false });
            frame.render_widget(modal_widget, area);
        }
    })?;

    Ok(())
}

fn build_output_lines(output: &str, term: Option<&str>, width: usize) -> Vec<Line<'static>> {
    let term = term.filter(|value| !value.is_empty());
    let wrapped = wrap_text_lines(output, width);
    let is_json = serde_json::from_str::<JsonValue>(output).is_ok();

    wrapped
        .into_iter()
        .map(|line| {
            let mut spans = if is_json {
                json_line_spans(&line)
            } else {
                vec![Span::raw(line.to_string())]
            };
            if let Some(term) = term {
                let ranges = find_match_ranges(&line, term);
                spans = apply_search_highlight(spans, &ranges);
            }
            Line::from(spans)
        })
        .collect()
}

struct StyledToken {
    start: usize,
    end: usize,
    style: Style,
}

fn key_style() -> Style {
    Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD)
}

fn string_style() -> Style {
    Style::default().fg(Color::Yellow)
}

fn number_style() -> Style {
    Style::default().fg(Color::Magenta)
}

fn bool_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

fn null_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::DIM)
}

fn punct_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn json_line_spans(line: &str) -> Vec<Span<'static>> {
    let mut tokens = Vec::new();
    let mut i = 0usize;
    while i < line.len() {
        let ch = line[i..].chars().next().unwrap();
        match ch {
            '"' => {
                let end = find_string_end(line, i);
                let is_key = is_json_key(line, end);
                tokens.push(StyledToken {
                    start: i,
                    end,
                    style: if is_key { key_style() } else { string_style() },
                });
                i = end;
            }
            '{' | '}' | '[' | ']' | ':' | ',' => {
                let end = i + ch.len_utf8();
                tokens.push(StyledToken {
                    start: i,
                    end,
                    style: punct_style(),
                });
                i = end;
            }
            '-' | '0'..='9' => {
                let end = find_number_end(line, i);
                tokens.push(StyledToken {
                    start: i,
                    end,
                    style: number_style(),
                });
                i = end;
            }
            't' => {
                if line[i..].starts_with("true") && is_literal_boundary(line, i + 4) {
                    tokens.push(StyledToken {
                        start: i,
                        end: i + 4,
                        style: bool_style(),
                    });
                    i += 4;
                } else {
                    i += ch.len_utf8();
                }
            }
            'f' => {
                if line[i..].starts_with("false") && is_literal_boundary(line, i + 5) {
                    tokens.push(StyledToken {
                        start: i,
                        end: i + 5,
                        style: bool_style(),
                    });
                    i += 5;
                } else {
                    i += ch.len_utf8();
                }
            }
            'n' => {
                if line[i..].starts_with("null") && is_literal_boundary(line, i + 4) {
                    tokens.push(StyledToken {
                        start: i,
                        end: i + 4,
                        style: null_style(),
                    });
                    i += 4;
                } else {
                    i += ch.len_utf8();
                }
            }
            _ => {
                i += ch.len_utf8();
            }
        }
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut cursor = 0usize;
    for token in tokens {
        if token.start > cursor {
            spans.push(Span::raw(line[cursor..token.start].to_string()));
        }
        if token.end > token.start {
            spans.push(Span::styled(
                line[token.start..token.end].to_string(),
                token.style,
            ));
        }
        cursor = token.end;
    }
    if cursor < line.len() {
        spans.push(Span::raw(line[cursor..].to_string()));
    }
    spans
}

fn find_string_end(line: &str, start: usize) -> usize {
    let mut escaped = false;
    let mut end = line.len();
    let offset = start + 1;
    for (idx, ch) in line[offset..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            end = offset + idx + ch.len_utf8();
            break;
        }
    }
    end
}

fn is_json_key(line: &str, end: usize) -> bool {
    for ch in line[end..].chars() {
        if ch.is_whitespace() {
            continue;
        }
        return ch == ':';
    }
    false
}

fn find_number_end(line: &str, start: usize) -> usize {
    let mut end = start;
    while end < line.len() {
        let ch = line[end..].chars().next().unwrap();
        if ch.is_ascii_digit() || matches!(ch, '.' | 'e' | 'E' | '+' | '-') {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    end
}

fn is_literal_boundary(line: &str, end: usize) -> bool {
    if end >= line.len() {
        return true;
    }
    let ch = line[end..].chars().next().unwrap();
    ch.is_whitespace() || matches!(ch, ',' | ']' | '}' | ':')
}

fn apply_search_highlight(
    spans: Vec<Span<'static>>,
    ranges: &[(usize, usize)],
) -> Vec<Span<'static>> {
    if ranges.is_empty() {
        return spans;
    }

    let highlight = Style::default()
        .fg(Color::Black)
        .bg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let mut result = Vec::new();
    let mut offset = 0usize;
    let mut range_index = 0usize;

    for span in spans {
        let text = span.content.as_ref();
        let span_len = text.len();
        let mut local_pos = 0usize;

        while range_index < ranges.len() {
            let (range_start, range_len) = ranges[range_index];
            let range_end = range_start + range_len;

            if range_end <= offset {
                range_index += 1;
                continue;
            }
            if range_start >= offset + span_len {
                break;
            }

            let seg_start = range_start.saturating_sub(offset).max(local_pos);
            let seg_end = range_end.saturating_sub(offset).min(span_len);

            if seg_start > local_pos {
                result.push(Span::styled(
                    text[local_pos..seg_start].to_string(),
                    span.style,
                ));
            }
            if seg_end > seg_start {
                result.push(Span::styled(
                    text[seg_start..seg_end].to_string(),
                    highlight,
                ));
            }
            local_pos = seg_end;

            if range_end <= offset + span_len {
                range_index += 1;
            } else {
                break;
            }
        }

        if local_pos < span_len {
            result.push(Span::styled(
                text[local_pos..].to_string(),
                span.style,
            ));
        }

        offset += span_len;
    }

    result
}

fn wrap_text_lines(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let indent_len = line.chars().take_while(|ch| ch.is_whitespace()).count();
        let indent: String = line.chars().take(indent_len).collect();
        let mut current = indent.clone();
        let mut current_len = indent_len;
        let mut last_break: Option<usize> = None;
        let mut in_string = false;
        let mut escaped = false;

        for ch in line.chars().skip(indent_len) {
            current.push(ch);
            current_len += 1;

            if in_string {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }
            } else {
                if ch == '"' {
                    in_string = true;
                }
                if ch.is_whitespace() {
                    last_break = Some(current_len - 1);
                }
            }

            if current_len >= width {
                if let Some(break_at) = last_break {
                    let (left, right) = split_at_char_idx(&current, break_at + 1);
                    lines.push(left.trim_end().to_string());
                    current = indent.clone();
                    current.push_str(right.trim_start());
                } else {
                    lines.push(current.clone());
                    current = indent.clone();
                }
                current_len = current.chars().count();
                last_break = None;
                in_string = false;
                escaped = false;
            }
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn split_at_char_idx(value: &str, idx: usize) -> (String, String) {
    let mut split_byte = value.len();
    let mut count = 0usize;
    for (byte_idx, _) in value.char_indices() {
        if count == idx {
            split_byte = byte_idx;
            break;
        }
        count += 1;
    }
    let left = value[..split_byte].to_string();
    let right = value[split_byte..].to_string();
    (left, right)
}

fn summary_text() -> &'static str {
    "(/n|^n) next+step  (/a|^a) next+all  (/r|^r) release+all  (/w|^w) show  (/g|^g) step  (/s term) search  (/d|^d) prev  (/e|^e) next  (/x|^x) clear  (/m|^m) summary"
}

fn current_step_index(state: &AppState) -> Option<usize> {
    let current = state.current_step_uuid.as_deref()?;
    state
        .timeline_steps
        .iter()
        .position(|uuid| uuid == current)
}

fn build_timeline_lines(state: &AppState, width: usize) -> Vec<Line<'static>> {
    if state.timeline_steps.is_empty() || width == 0 {
        if let Some(notice) = state.status_notice.as_deref() {
            return vec![line_with_notice("", notice, width)];
        }
        return Vec::new();
    }

    let filled_until = current_step_index(state).map(|idx| idx + 1).unwrap_or(0);
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut line_len = 0usize;

    for (idx, _) in state.timeline_steps.iter().enumerate() {
        let symbol = if idx < filled_until { "●" } else { "○" };
        let token = if line.is_empty() {
            symbol.to_string()
        } else {
            format!(" {}", symbol)
        };
        let token_len = token.chars().count();
        if line_len + token_len > width && !line.is_empty() {
            lines.push(line);
            line = symbol.to_string();
            line_len = symbol.chars().count();
            continue;
        }
        line.push_str(&token);
        line_len += token_len;
    }

    if !line.is_empty() {
        lines.push(line);
    }
    if let Some(notice) = state.status_notice.as_deref() {
        let mut result = Vec::new();
        if let Some(first) = lines.first() {
            result.push(line_with_notice(first, notice, width));
            for line in lines.iter().skip(1) {
                result.push(plain_line(line));
            }
            return result;
        }
        return vec![line_with_notice("", notice, width)];
    }

    lines.into_iter().map(|line| plain_line(&line)).collect()
}

fn timeline_height_for_width(lines: &[Line<'_>]) -> u16 {
    if lines.is_empty() {
        return 0;
    }
    lines.len().max(1) as u16
}

fn plain_line(text: &str) -> Line<'static> {
    Line::from(Span::raw(text.to_string()))
}

fn line_with_notice(left: &str, notice: &str, width: usize) -> Line<'static> {
    if width == 0 {
        return Line::from(Span::raw(String::new()));
    }
    let notice_len = notice.chars().count();
    let notice_style = Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK);
    if notice_len >= width {
        let text = truncate_to_width(notice, width);
        return Line::from(Span::styled(text, notice_style));
    }

    let mut left_trimmed = truncate_to_width(left, width - notice_len);
    let left_len = left_trimmed.chars().count();
    if left_len < width - notice_len {
        left_trimmed.push_str(&" ".repeat(width - notice_len - left_len));
    }
    Line::from(vec![
        Span::raw(left_trimmed),
        Span::styled(notice.to_string(), notice_style),
    ])
}

fn truncate_to_width(value: &str, width: usize) -> String {
    value.chars().take(width).collect()
}

fn summary_height_for_width(width: u16) -> u16 {
    let summary_width = width as usize;
    let summary_lines = wrap_text_lines(summary_text(), summary_width);
    summary_lines.len().max(1) as u16
}

fn output_height_for_size(state: &AppState, width: u16, height: u16) -> usize {
    let summary_height = summary_height_for_width(width);
    let input_height = 3u16;
    let timeline_lines = build_timeline_lines(state, width as usize);
    let timeline_height = timeline_height_for_width(&timeline_lines);
    let dropdown_height =
        dropdown_height_for_state(state, height, summary_height, timeline_height);
    let output_height =
        height.saturating_sub(summary_height + input_height + dropdown_height + timeline_height);
    output_height.saturating_sub(2).max(1) as usize
}

fn dropdown_height_for_state(
    state: &AppState,
    height: u16,
    summary_height: u16,
    timeline_height: u16,
) -> u16 {
    if !state.dropdown_open {
        return 0;
    }
    let candidates = dropdown_candidates(state);
    if candidates.is_empty() {
        return 0;
    }
    let input_height = 3u16;
    let available = height.saturating_sub(summary_height + input_height + timeline_height);
    if available <= 2 {
        return 0;
    }
    let max_visible = available.saturating_sub(2);
    let visible = max_visible.min(candidates.len() as u16).min(8);
    if visible == 0 {
        0
    } else {
        visible + 2
    }
}

fn dropdown_window(total: usize, selected: usize, visible: usize) -> (usize, usize) {
    if total == 0 || visible == 0 {
        return (0, 0);
    }
    if total <= visible {
        return (0, total);
    }
    let half = visible / 2;
    let mut start = selected.saturating_sub(half);
    if start + visible > total {
        start = total.saturating_sub(visible);
    }
    (start, start + visible)
}

fn on_input_changed(state: &mut AppState) {
    state.dropdown_suppressed = false;
    if state.input.is_empty() {
        state.dropdown_open = false;
        state.dropdown_index = 0;
        return;
    }
    let candidates = dropdown_candidates(state);
    state.dropdown_open = !candidates.is_empty();
    state.dropdown_index = 0;
}

fn navigate_dropdown(state: &mut AppState, delta: isize) {
    if state.dropdown_suppressed {
        return;
    }
    let candidates = dropdown_candidates(state);
    if candidates.is_empty() {
        state.dropdown_open = false;
        state.dropdown_index = 0;
        return;
    }
    if !state.dropdown_open {
        state.dropdown_open = true;
        state.dropdown_index = if delta < 0 {
            candidates.len().saturating_sub(1)
        } else {
            0
        };
        return;
    }

    let len = candidates.len() as isize;
    let mut index = state.dropdown_index as isize + delta;
    if index < 0 {
        index = len - 1;
    } else if index >= len {
        index = 0;
    }
    state.dropdown_index = index as usize;
}

fn selected_suggestion(state: &AppState) -> Option<String> {
    let candidates = dropdown_candidates(state);
    if candidates.is_empty() {
        return None;
    }
    let index = state.dropdown_index.min(candidates.len().saturating_sub(1));
    Some(candidates[index].clone())
}

fn record_history(state: &mut AppState, command: &str) {
    let entry = command.trim();
    if entry.is_empty() {
        return;
    }
    state.history.retain(|item| item != entry);
    state.history.push(entry.to_string());
    if state.history.len() > 50 {
        state.history.remove(0);
    }
}

fn dropdown_candidates(state: &AppState) -> Vec<String> {
    let filter = state.input.trim_start().to_ascii_lowercase();
    let mut items = Vec::new();
    for cmd in command_list() {
        if matches_filter(cmd, &filter) {
            items.push(cmd.to_string());
        }
    }
    for entry in state.history.iter().rev() {
        if matches_filter(entry, &filter) && !items.iter().any(|item| item == entry) {
            items.push(entry.clone());
        }
    }
    items
}

fn matches_filter(value: &str, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }
    value.to_ascii_lowercase().starts_with(filter)
}

fn command_list() -> &'static [&'static str] {
    &[
        "/n", "/a", "/r", "/w", "/g", "/m", "/s ", "/d", "/e", "/x", "STEP", "SHOW",
        "RELEASE", "ALL", "NEXT", "PAUSE",
    ]
}

fn help_text() -> &'static str {
    "Command summary:\n\
/n (^n) - Send NEXT then STEP to run the next step and show it.\n\
/a (^a) - Send NEXT then ALL to run the next step and list the history.\n\
/r (^r) - Send RELEASE then ALL to run the remaining pipeline.\n\
/w (^w) - Send SHOW to display the compiled script.\n\
/g (^g) - Send STEP to fetch the current waiting step.\n\
/s <term> - Highlight matches in the output and jump to the first.\n\
/d (^d) - Jump to the previous search match.\n\
/e (^e) - Jump to the next search match.\n\
/x (^x) - Clear the search highlight.\n\
/m (^m) - Open this summary.\n\
You can also type: STEP, SHOW, RELEASE, ALL, NEXT, PAUSE.\n\
\n\
Press ESC to close this summary."
}

fn centered_rect(width: u16, height: u16, area: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let x = area.x.saturating_add(area.width.saturating_sub(width) / 2);
    let y = area.y.saturating_add(area.height.saturating_sub(height) / 2);
    ratatui::layout::Rect {
        x,
        y,
        width,
        height,
    }
}

fn find_match_lines(lines: &[String], term: &str) -> Vec<usize> {
    let mut matches = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let ranges = find_match_ranges(line, term);
        for _ in ranges {
            matches.push(idx);
        }
    }
    matches
}

fn find_match_ranges(line: &str, term: &str) -> Vec<(usize, usize)> {
    if term.is_empty() {
        return Vec::new();
    }
    let line_lower = line.to_ascii_lowercase();
    let term_lower = term.to_ascii_lowercase();
    let mut ranges = Vec::new();
    let mut start = 0usize;
    while let Some(pos) = line_lower[start..].find(&term_lower) {
        let index = start + pos;
        ranges.push((index, term_lower.len()));
        start = index + term_lower.len().max(1);
        if start >= line_lower.len() {
            break;
        }
    }
    ranges
}
