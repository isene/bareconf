use crust::{Crust, Pane, Input, style};
use std::collections::BTreeMap;
use std::path::PathBuf;

const COLOR_NAMES: [&str; 16] = [
    "user", "host", "cwd", "prompt", "cmd", "nick", "gnick", "path",
    "switch", "bookmark", "colon", "git", "stamp", "tabsel", "tabopt", "suggest",
];
const COLOR_DESCS: [&str; 16] = [
    "Username", "Hostname", "Directory", "Prompt >", "Commands", "Nicks",
    "Gnicks", "Paths", "Switches", "Bookmarks", "Colon cmds", "Git branch",
    "Timestamps", "Tab select", "Tab options", "Suggestions",
];
const THEME_NAMES: [&str; 6] = ["default", "solarized", "dracula", "gruvbox", "nord", "monokai"];
const THEMES: [[u8; 16]; 6] = [
    [2, 2, 81, 208, 48, 6, 33, 3, 6, 5, 4, 208, 245, 7, 245, 240],
    [64, 64, 37, 136, 33, 37, 33, 136, 37, 125, 33, 166, 245, 7, 245, 240],
    [84, 84, 141, 212, 84, 117, 189, 228, 117, 212, 189, 215, 245, 7, 245, 240],
    [142, 142, 214, 208, 142, 108, 109, 223, 108, 175, 109, 208, 245, 7, 245, 240],
    [110, 110, 111, 173, 110, 110, 111, 222, 110, 139, 111, 173, 245, 7, 245, 240],
    [148, 148, 81, 208, 148, 81, 141, 228, 81, 197, 141, 208, 245, 7, 245, 240],
];

struct App {
    colors: [u8; 16],
    nicks: BTreeMap<String, String>,
    gnicks: BTreeMap<String, String>,
    abbrevs: BTreeMap<String, String>,
    bookmarks: BTreeMap<String, String>,
    color_idx: usize,
    palette_row: u8,
    palette_col: u8,
    tab: usize,          // 0=colors, 1=themes, 2=aliases
    alias_tab: usize,    // 0=nicks, 1=gnicks, 2=abbrevs, 3=bookmarks
    alias_idx: usize,
    theme_idx: usize,
    dirty: bool,
    config_path: PathBuf,
}

impl App {
    fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let config_path = PathBuf::from(&home).join(".barerc");
        let mut app = App {
            colors: THEMES[0], nicks: BTreeMap::new(), gnicks: BTreeMap::new(),
            abbrevs: BTreeMap::new(), bookmarks: BTreeMap::new(),
            color_idx: 0, palette_row: 0, palette_col: 0,
            tab: 0, alias_tab: 0, alias_idx: 0, theme_idx: 0,
            dirty: false, config_path,
        };
        app.load_config();
        let c = app.colors[0];
        app.palette_row = c / 16;
        app.palette_col = c % 16;
        app
    }

    fn palette_idx(&self) -> u8 {
        self.palette_row * 16 + self.palette_col
    }

    fn load_config(&mut self) {
        let content = match std::fs::read_to_string(&self.config_path) {
            Ok(c) => c, Err(_) => return,
        };
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((key, val)) = line.split_once('=') {
                let (key, val) = (key.trim(), val.trim());
                if let Some(n) = key.strip_prefix("nick.") {
                    self.nicks.insert(n.into(), val.into());
                } else if let Some(n) = key.strip_prefix("gnick.") {
                    self.gnicks.insert(n.into(), val.into());
                } else if let Some(n) = key.strip_prefix("abbrev.") {
                    self.abbrevs.insert(n.into(), val.into());
                } else if let Some(n) = key.strip_prefix("bm.") {
                    self.bookmarks.insert(n.into(), val.into());
                } else if let Some(cn) = key.strip_prefix("c_") {
                    if let Ok(v) = val.parse::<u8>() {
                        if let Some(i) = COLOR_NAMES.iter().position(|&n| n == cn) {
                            self.colors[i] = v;
                        }
                    }
                }
            }
        }
    }

    fn save_config(&self) {
        let mut out = String::new();
        for (k, v) in &self.nicks { out += &format!("nick.{} = {}\n", k, v); }
        for (k, v) in &self.gnicks { out += &format!("gnick.{} = {}\n", k, v); }
        for (k, v) in &self.abbrevs { out += &format!("abbrev.{} = {}\n", k, v); }
        for (k, v) in &self.bookmarks { out += &format!("bm.{} = {}\n", k, v); }
        for (i, n) in COLOR_NAMES.iter().enumerate() {
            out += &format!("c_{} = {}\n", n, self.colors[i]);
        }
        let _ = std::fs::write(&self.config_path, out);
    }

    fn alias_list(&self) -> Vec<(String, String)> {
        match self.alias_tab {
            0 => self.nicks.iter().map(|(k,v)| (k.clone(), v.clone())).collect(),
            1 => self.gnicks.iter().map(|(k,v)| (k.clone(), v.clone())).collect(),
            2 => self.abbrevs.iter().map(|(k,v)| (k.clone(), v.clone())).collect(),
            3 => self.bookmarks.iter().map(|(k,v)| (k.clone(), v.clone())).collect(),
            _ => vec![],
        }
    }

    fn alias_tab_name(&self) -> &str {
        ["Nicks", "Gnicks", "Abbrevs", "Bookmarks"][self.alias_tab]
    }
}

fn build_prompt_preview(app: &App) -> String {
    format!("{}@{}: {} ({}) {} echo hello",
        style::fg("user", app.colors[0]),
        style::fg("host", app.colors[1]),
        style::fg("~/projects/bare", app.colors[2]),
        style::fg("main", app.colors[11]),
        style::fg(">", app.colors[3]),
    )
}

fn build_color_list(app: &App) -> String {
    let mut lines = Vec::new();
    for (i, name) in COLOR_NAMES.iter().enumerate() {
        let c = app.colors[i];
        let marker = if i == app.color_idx { ">" } else { " " };
        let sample = style::fg("sample", c);
        let line = if i == app.color_idx {
            format!("{} {:<10} {:>3}  {}", style::bold(marker), style::bold(name), c, sample)
        } else {
            format!("{} {:<10} {:>3}  {}", marker, name, c, sample)
        };
        lines.push(line);
    }
    lines.join("\n")
}

fn build_palette(app: &App) -> String {
    let mut lines = Vec::new();
    let sel = app.palette_idx();
    let cur = app.colors[app.color_idx];
    for row in 0..16u8 {
        let mut line = String::new();
        for col in 0..16u8 {
            let idx = row * 16 + col;
            if idx == sel {
                line += &format!("{}", style::fb(&format!("{:^3}", idx), 0, idx));
            } else if idx == cur {
                let fg = if idx < 8 { 15 } else { 0 };
                line += &format!("{}", style::fb(" * ", fg, idx));
            } else {
                line += &format!("{}", style::bg("   ", idx));
            }
        }
        lines.push(line);
    }
    lines.push(String::new());
    lines.push(format!("{}: {} (current: {}, selecting: {})",
        style::bold(COLOR_NAMES[app.color_idx]),
        COLOR_DESCS[app.color_idx], cur, sel));
    lines.join("\n")
}

fn build_themes(app: &App) -> String {
    let mut lines = Vec::new();
    for (i, name) in THEME_NAMES.iter().enumerate() {
        let marker = if i == app.theme_idx { ">" } else { " " };
        let mut swatches = String::new();
        for c in &THEMES[i][..12] {
            swatches += &format!("{}", style::bg("  ", *c));
        }
        let line = if i == app.theme_idx {
            format!("{} {:<12} {}", marker, style::bold(name), swatches)
        } else {
            format!("{} {:<12} {}", marker, name, swatches)
        };
        lines.push(line);
    }
    lines.join("\n")
}

fn build_aliases(app: &App) -> String {
    let list = app.alias_list();
    if list.is_empty() {
        return "  (no entries)".into();
    }
    let mut lines = Vec::new();
    for (i, (k, v)) in list.iter().enumerate() {
        let marker = if i == app.alias_idx { ">" } else { " " };
        let line = if i == app.alias_idx {
            format!("{} {} = {}", marker, style::fg(&style::bold(k), 6), style::bold(v))
        } else {
            format!("{} {} = {}", marker, style::fg(k, 6), v)
        };
        lines.push(line);
    }
    lines.join("\n")
}

fn main() {
    Crust::init();
    let mut app = App::new();

    // Create panes
    let (w, h) = Crust::terminal_size();
    let left_w = w * 2 / 5;
    let right_w = w - left_w - 1;

    // Title/preview pane (full width, top)
    let mut title_pane = Pane::new(1, 1, w, 2, 255, 0);

    // Left pane: color list
    let mut left_pane = Pane::new(1, 4, left_w, h - 5, 255, 0);
    left_pane.border = true;

    // Right pane: palette/themes/aliases
    let mut right_pane = Pane::new(left_w + 2, 4, right_w - 1, h - 5, 255, 0);
    right_pane.border = true;

    // Help bar (bottom)
    let mut help_pane = Pane::new(1, h, w, 1, 0, 245);

    let tab_names = ["Colors+Palette", "Themes", "Aliases"];

    loop {
        // Update title
        let preview = build_prompt_preview(&app);
        let dirty = if app.dirty { " [modified]" } else { "" };
        title_pane.set_text(&format!(" bareconf{}\n {}", dirty, preview));
        title_pane.refresh();

        // Update left pane
        let ltitle = if app.tab == 0 {
            format!(" {} ", style::bold("Colors"))
        } else {
            " Colors ".into()
        };
        left_pane.border = true;
        left_pane.fg = if app.tab == 0 { 208 } else { 240 };
        left_pane.set_text(&build_color_list(&app));
        left_pane.border_refresh();
        left_pane.refresh();

        // Update right pane
        let (rtitle, rcontent) = match app.tab {
            0 => {
                let t = format!(" {} - c_{} ", style::bold("Palette"), COLOR_NAMES[app.color_idx]);
                (t, build_palette(&app))
            }
            1 => {
                (format!(" {} ", style::bold("Themes")), build_themes(&app))
            }
            2 => {
                let tn = app.alias_tab_name();
                (format!(" {} (Left/Right: switch) ", style::bold(tn)), build_aliases(&app))
            }
            _ => (String::new(), String::new()),
        };
        right_pane.fg = if app.tab != 0 { 208 } else { 240 };
        right_pane.set_text(&rcontent);
        right_pane.border_refresh();
        right_pane.refresh();

        // Print border titles manually
        crust::cursor::Cursor::set(left_pane.x, left_pane.y - 1);
        let lfg = if app.tab == 0 { 208u8 } else { 240u8 };
        let rfg = if app.tab != 0 { 208u8 } else { 240u8 };
        print!("{}", style::fg(&ltitle, lfg));
        crust::cursor::Cursor::set(right_pane.x, right_pane.y - 1);
        print!("{}", style::fg(&rtitle, rfg));
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Help bar
        let tabs: Vec<String> = tab_names.iter().enumerate().map(|(i, n)| {
            if i == app.tab { style::reverse(&format!(" {} ", n)) } else { format!(" {} ", n) }
        }).collect();
        help_pane.set_text(&format!(" {}  Arrows:move Enter:apply s:save q:quit", tabs.join(" ")));
        help_pane.refresh();

        // Input
        let key = match Input::getchr(None) {
            Some(k) => k,
            None => continue,
        };
        match key.as_str() {
            "q" | "ESC" => {
                if app.dirty { app.save_config(); }
                break;
            }
            "s" => { app.save_config(); app.dirty = false; }
            "TAB" => { app.tab = (app.tab + 1) % 3; }
            "S-TAB" => { app.tab = if app.tab == 0 { 2 } else { app.tab - 1 }; }
            "UP" => match app.tab {
                0 => { if app.color_idx > 0 { app.color_idx -= 1; let c = app.colors[app.color_idx]; app.palette_row = c / 16; app.palette_col = c % 16; } }
                1 => { if app.theme_idx > 0 { app.theme_idx -= 1; } }
                2 => { if app.alias_idx > 0 { app.alias_idx -= 1; } }
                _ => {}
            },
            "DOWN" => match app.tab {
                0 => { if app.color_idx < 15 { app.color_idx += 1; let c = app.colors[app.color_idx]; app.palette_row = c / 16; app.palette_col = c % 16; } }
                1 => { if app.theme_idx < 5 { app.theme_idx += 1; } }
                2 => { let len = app.alias_list().len(); if len > 0 && app.alias_idx + 1 < len { app.alias_idx += 1; } }
                _ => {}
            },
            "LEFT" => match app.tab {
                0 => { if app.palette_col > 0 { app.palette_col -= 1; } }
                2 => { if app.alias_tab > 0 { app.alias_tab -= 1; app.alias_idx = 0; } }
                _ => {}
            },
            "RIGHT" => match app.tab {
                0 => { if app.palette_col < 15 { app.palette_col += 1; } }
                2 => { if app.alias_tab < 3 { app.alias_tab += 1; app.alias_idx = 0; } }
                _ => {}
            },
            "S-UP" => {
                if app.tab == 0 && app.palette_row > 0 { app.palette_row -= 1; }
            }
            "S-DOWN" => {
                if app.tab == 0 && app.palette_row < 15 { app.palette_row += 1; }
            }
            "ENTER" => match app.tab {
                0 => { app.colors[app.color_idx] = app.palette_idx(); app.dirty = true; }
                1 => { app.colors = THEMES[app.theme_idx]; app.dirty = true; }
                _ => {}
            },
            "C-D" => {
                if app.tab == 2 {
                    let list = app.alias_list();
                    if let Some((key, _)) = list.get(app.alias_idx) {
                        let key = key.clone();
                        match app.alias_tab {
                            0 => { app.nicks.remove(&key); }
                            1 => { app.gnicks.remove(&key); }
                            2 => { app.abbrevs.remove(&key); }
                            3 => { app.bookmarks.remove(&key); }
                            _ => {}
                        }
                        app.dirty = true;
                        let len = app.alias_list().len();
                        if app.alias_idx >= len && len > 0 { app.alias_idx = len - 1; }
                    }
                }
            }
            _ => {}
        }
    }

    Crust::cleanup();
}
