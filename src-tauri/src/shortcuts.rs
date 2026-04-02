use tauri_plugin_global_shortcut::Shortcut;
use unicode_normalization::UnicodeNormalization;

pub fn normalize_shortcut(shortcut: &str) -> String {
    let parts: Vec<&str> = shortcut
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();

    if parts.is_empty() {
        return String::new();
    }

    let mut normalized = Vec::with_capacity(parts.len());

    for (index, part) in parts.iter().enumerate() {
        let is_main_key = index == parts.len() - 1;
        if is_main_key {
            normalized.push(normalize_main_key(part));
        } else {
            normalized.push(normalize_modifier(part));
        }
    }

    normalized.join("+")
}

pub fn parse_shortcut_string(shortcut: &str) -> Result<(String, Shortcut), String> {
    let normalized = normalize_shortcut(shortcut);
    let parsed = normalized
        .parse::<Shortcut>()
        .map_err(|error| format!("Invalid shortcut format: {error}"))?;

    Ok((normalized, parsed))
}

fn normalize_modifier(part: &str) -> String {
    match part.trim().to_ascii_lowercase().as_str() {
        "cmdorctrl" | "commandorcontrol" | "cmdorcontrol" => "CmdOrCtrl".to_string(),
        "cmd" | "command" | "meta" => "Cmd".to_string(),
        "ctrl" | "control" => "Ctrl".to_string(),
        "alt" | "option" | "opt" => "Alt".to_string(),
        "shift" => "Shift".to_string(),
        _ => part.trim().to_string(),
    }
}

fn normalize_main_key(part: &str) -> String {
    let trimmed = part.trim();
    let lower = trimmed.to_ascii_lowercase();

    match lower.as_str() {
        "space" | "spacebar" => return "Space".to_string(),
        "enter" | "return" => return "Enter".to_string(),
        "esc" | "escape" => return "Escape".to_string(),
        "tab" => return "Tab".to_string(),
        "backspace" => return "Backspace".to_string(),
        "delete" | "del" => return "Delete".to_string(),
        "up" | "arrowup" => return "Up".to_string(),
        "down" | "arrowdown" => return "Down".to_string(),
        "left" | "arrowleft" => return "Left".to_string(),
        "right" | "arrowright" => return "Right".to_string(),
        "home" => return "Home".to_string(),
        "end" => return "End".to_string(),
        "pageup" => return "PageUp".to_string(),
        "pagedown" => return "PageDown".to_string(),
        _ => {}
    }

    let mut chars = trimmed.chars();
    if let (Some(character), None) = (chars.next(), chars.next()) {
        if character.is_ascii_alphanumeric() {
            return character.to_ascii_uppercase().to_string();
        }

        let ascii_fallback = trimmed
            .nfd()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect::<String>();

        if ascii_fallback.len() == 1 {
            return ascii_fallback.to_ascii_uppercase();
        }
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::{normalize_shortcut, parse_shortcut_string};

    #[test]
    fn normalizes_option_modified_letters_back_to_ascii_keys() {
        assert_eq!(normalize_shortcut("CmdOrCtrl+Alt+Ç"), "CmdOrCtrl+Alt+C");
        assert_eq!(normalize_shortcut("CmdOrCtrl+Alt+é"), "CmdOrCtrl+Alt+E");
    }

    #[test]
    fn preserves_valid_shortcuts() {
        assert_eq!(normalize_shortcut("CmdOrCtrl+Shift+V"), "CmdOrCtrl+Shift+V");
        assert_eq!(normalize_shortcut("Alt+Space"), "Alt+Space");
    }

    #[test]
    fn parses_normalized_shortcuts() {
        let (normalized, _) =
            parse_shortcut_string("CmdOrCtrl+Alt+Ç").expect("shortcut should parse");

        assert_eq!(normalized, "CmdOrCtrl+Alt+C");
    }
}
