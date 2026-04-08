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

#[derive(PartialEq, Clone, Copy)]
enum Tab { Colors, Themes, Aliases }

struct App {
    colors: [u8; 16],
    nicks: BTreeMap<String, String>,
    gnicks: BTreeMap<String, String>,
    abbrevs: BTreeMap<String, String>,
    bookmarks: BTreeMap<String, String>,
    color_idx: usize,
    palette_row: u8,
    palette_col: u8,
    tab: Tab,
    alias_tab: usize,
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
            tab: Tab::Colors, alias_tab: 0, alias_idx: 0, theme_idx: 0,
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

fn prompt_preview(app: &App) -> String {
    format!("  {}@{}: {} ({}) {} {}",
        style::fg("user", app.colors[0]),
        style::fg("host", app.colors[1]),
        style::fg("~/projects/bare", app.colors[2]),
        style::fg("main", app.colors[11]),
        style::fg(">", app.colors[3]),
        style::fg("echo hello", app.colors[4]),
    )
}

fn color_list_text(app: &App) -> String {
    COLOR_NAMES.iter().enumerate().map(|(i, name)| {
        let c = app.colors[i];
        let sample = style::fg("sample", c);
        if i == app.color_idx {
            format!("  {} {:<10}{:>3}  {}", style::bold(">"), style::bold(name), c, sample)
        } else {
            format!("    {:<10}{:>3}  {}", name, c, sample)
        }
    }).collect::<Vec<_>>().join("\n")
}

fn palette_text(app: &App) -> String {
    let sel = app.palette_idx();
    let cur = app.colors[app.color_idx];
    let mut lines = Vec::new();
    for row in 0..16u8 {
        let mut line = String::new();
        for col in 0..16u8 {
            let idx = row * 16 + col;
            if idx == sel {
                let fg: u8 = if idx < 8 || (idx >= 16 && idx < 52) { 15 } else { 0 };
                line += &style::fb(&format!("{:^3}", idx), fg, idx);
            } else if idx == cur {
                let fg: u8 = if idx < 8 || (idx >= 16 && idx < 52) { 15 } else { 0 };
                line += &style::fb(" * ", fg, idx);
            } else {
                line += &style::bg("   ", idx);
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

fn themes_text(app: &App, max_w: u16) -> String {
    // Calculate how many swatch chars we can fit
    // Each line: 2 (marker) + 12 (name) + swatches
    let avail = (max_w as usize).saturating_sub(16);
    let swatch_w = (avail / 12).max(1).min(3);

    THEME_NAMES.iter().enumerate().map(|(i, name)| {
        let marker = if i == app.theme_idx { ">" } else { " " };
        let swatch_str: String = " ".repeat(swatch_w);
        let mut swatches = String::new();
        for c in &THEMES[i][..12] {
            swatches += &style::bg(&swatch_str, *c);
        }
        if i == app.theme_idx {
            format!("{} {:<12}{}", marker, style::bold(name), swatches)
        } else {
            format!("{} {:<12}{}", marker, name, swatches)
        }
    }).collect::<Vec<_>>().join("\n")
}

fn aliases_text(app: &App) -> String {
    let list = app.alias_list();
    if list.is_empty() { return "  (no entries)".into(); }
    list.iter().enumerate().map(|(i, (k, v))| {
        let marker = if i == app.alias_idx { ">" } else { " " };
        if i == app.alias_idx {
            format!("{} {} = {}", marker, style::fg(&style::bold(k), 6), style::bold(v))
        } else {
            format!("{} {} = {}", marker, style::fg(k, 6), v)
        }
    }).collect::<Vec<_>>().join("\n")
}

fn main() {
    Crust::init();
    Crust::clear_screen();
    let mut app = App::new();

    let (w, h) = Crust::terminal_size();
    // Layout: row 1 = title, row 2 = preview, rows 4..h-1 = content, row h = help
    // Borders are drawn OUTSIDE pane area, so panes need 1px margin from edges
    let left_w = (w * 2 / 5).min(35);
    let left_x = 2u16;              // border at x=1 (left edge)
    let right_x = left_x + left_w + 3; // gap for left border-right + space + right border-left
    let right_w = w - right_x - 1;  // leave room for right border

    let mut title_pane = Pane::new(1, 1, w, 1, 208, 0);
    let mut preview_pane = Pane::new(1, 2, w, 1, 255, 0);
    let content_y = 4u16;           // content starts here (border at y-1=3)
    let content_h = h.saturating_sub(6);
    let mut left_pane = Pane::new(left_x, content_y, left_w, content_h, 255, 0);
    left_pane.border = true;
    let mut right_pane = Pane::new(right_x, content_y, right_w, content_h, 255, 0);
    right_pane.border = true;
    let mut help_pane = Pane::new(1, h, w, 1, 0, 245);

    loop {
        // Title
        let dirty = if app.dirty { " [modified]" } else { "" };
        title_pane.say(&format!(" bareconf{}", dirty));
        preview_pane.say(&prompt_preview(&app));

        // Left pane: always shows color list
        left_pane.fg = if app.tab == Tab::Colors { 208 } else { 240 };
        left_pane.set_text(&color_list_text(&app));
        left_pane.border_refresh();
        left_pane.refresh();

        // Right pane: depends on tab
        let rtitle = match app.tab {
            Tab::Colors => format!(" Palette: c_{} ", COLOR_NAMES[app.color_idx]),
            Tab::Themes => " Themes ".into(),
            Tab::Aliases => format!(" {} (Left/Right) ", app.alias_tab_name()),
        };
        right_pane.fg = if app.tab != Tab::Colors { 208 } else { 240 };
        let rcontent = match app.tab {
            Tab::Colors => palette_text(&app),
            Tab::Themes => themes_text(&app, right_w),
            Tab::Aliases => aliases_text(&app),
        };
        right_pane.set_text(&rcontent);
        right_pane.border_refresh();
        right_pane.refresh();

        // Overlay border titles
        let lfg = if app.tab == Tab::Colors { 208u8 } else { 240 };
        let rfg = if app.tab != Tab::Colors { 208u8 } else { 240 };
        crust::cursor::Cursor::set(left_pane.x, content_y - 1);
        print!("{}", style::fg(&format!(" {} ", if app.tab == Tab::Colors { style::bold("Colors") } else { "Colors".into() }), lfg));
        crust::cursor::Cursor::set(right_pane.x, content_y - 1);
        print!("{}", style::fg(&rtitle, rfg));
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Help bar
        let tabs = [("Colors", Tab::Colors), ("Themes", Tab::Themes), ("Aliases", Tab::Aliases)];
        let tab_str: Vec<String> = tabs.iter().map(|(n, t)| {
            if app.tab == *t { style::reverse(&format!(" {} ", n)) } else { format!(" {} ", n) }
        }).collect();
        help_pane.say(&format!(" {}  Up/Down:select  Enter:apply  s:save  q:quit", tab_str.join(" ")));

        // Input
        let key = match Input::getchr(None) { Some(k) => k, None => continue };
        match key.as_str() {
            "q" | "ESC" => { if app.dirty { app.save_config(); } break; }
            "s" => { app.save_config(); app.dirty = false; }
            "TAB" => {
                app.tab = match app.tab {
                    Tab::Colors => Tab::Themes, Tab::Themes => Tab::Aliases, Tab::Aliases => Tab::Colors,
                };
            }
            "S-TAB" => {
                app.tab = match app.tab {
                    Tab::Colors => Tab::Aliases, Tab::Themes => Tab::Colors, Tab::Aliases => Tab::Themes,
                };
            }
            "UP" => match app.tab {
                Tab::Colors => {
                    if app.color_idx > 0 {
                        app.color_idx -= 1;
                        let c = app.colors[app.color_idx];
                        app.palette_row = c / 16; app.palette_col = c % 16;
                    }
                }
                Tab::Themes => { if app.theme_idx > 0 { app.theme_idx -= 1; } }
                Tab::Aliases => { if app.alias_idx > 0 { app.alias_idx -= 1; } }
            },
            "DOWN" => match app.tab {
                Tab::Colors => {
                    if app.color_idx < 15 {
                        app.color_idx += 1;
                        let c = app.colors[app.color_idx];
                        app.palette_row = c / 16; app.palette_col = c % 16;
                    }
                }
                Tab::Themes => { if app.theme_idx < 5 { app.theme_idx += 1; } }
                Tab::Aliases => {
                    let len = app.alias_list().len();
                    if len > 0 && app.alias_idx + 1 < len { app.alias_idx += 1; }
                }
            },
            "LEFT" => match app.tab {
                Tab::Colors => { if app.palette_col > 0 { app.palette_col -= 1; } }
                Tab::Aliases => { if app.alias_tab > 0 { app.alias_tab -= 1; app.alias_idx = 0; } }
                _ => {}
            },
            "RIGHT" => match app.tab {
                Tab::Colors => { if app.palette_col < 15 { app.palette_col += 1; } }
                Tab::Aliases => { if app.alias_tab < 3 { app.alias_tab += 1; app.alias_idx = 0; } }
                _ => {}
            },
            "S-UP" => { if app.tab == Tab::Colors && app.palette_row > 0 { app.palette_row -= 1; } }
            "S-DOWN" => { if app.tab == Tab::Colors && app.palette_row < 15 { app.palette_row += 1; } }
            "ENTER" => match app.tab {
                Tab::Colors => { app.colors[app.color_idx] = app.palette_idx(); app.dirty = true; }
                Tab::Themes => { app.colors = THEMES[app.theme_idx]; app.dirty = true; }
                _ => {}
            },
            "C-D" | "DEL" => {
                if app.tab == Tab::Aliases {
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
