use crust::{Crust, Pane, Input};
use crust::style;
use std::collections::BTreeMap;
use std::path::PathBuf;

const COLOR_NAMES: [&str; 18] = [
    "user", "host", "cwd", "prompt", "cmd", "nick", "gnick", "path",
    "switch", "bookmark", "colon", "git", "stamp", "tabsel", "tabopt", "suggest",
    "user_root", "host_root",
];
const COLOR_DESCS: [&str; 18] = [
    "Username", "Hostname", "Directory", "Prompt >", "Commands", "Nicks",
    "Gnicks", "Paths", "Switches", "Bookmarks", "Colon cmds", "Git branch",
    "Timestamps", "Tab select", "Tab options", "Suggestions",
    "Root username", "Root hostname",
];
const THEME_NAMES: [&str; 6] = ["default", "solarized", "dracula", "gruvbox", "nord", "monokai"];
const THEMES: [[u8; 18]; 6] = [
    [2, 2, 81, 208, 48, 6, 33, 3, 6, 5, 4, 208, 245, 7, 245, 240, 196, 196],
    [64, 64, 37, 136, 33, 37, 33, 136, 37, 125, 33, 166, 245, 7, 245, 240, 196, 196],
    [84, 84, 141, 212, 84, 117, 189, 228, 117, 212, 189, 215, 245, 7, 245, 240, 196, 196],
    [142, 142, 214, 208, 142, 108, 109, 223, 108, 175, 109, 208, 245, 7, 245, 240, 196, 196],
    [110, 110, 111, 173, 110, 110, 111, 222, 110, 139, 111, 173, 245, 7, 245, 240, 196, 196],
    [148, 148, 81, 208, 148, 81, 141, 228, 81, 197, 141, 208, 245, 7, 245, 240, 196, 196],
];

#[derive(Clone)]
enum ItemKind {
    Color(usize),
    Theme,
    Bool(&'static str, bool),
    Number(&'static str, u64),
    Choice(&'static str, Vec<String>, String),
    Alias(String, String),
}

struct Category {
    name: String,
    items: Vec<Item>,
}

#[derive(Clone)]
struct Item {
    label: String,
    kind: ItemKind,
}

struct App {
    top: Pane,
    left: Pane,
    right: Pane,
    status: Pane,
    colors: [u8; 18],
    nicks: BTreeMap<String, String>,
    gnicks: BTreeMap<String, String>,
    abbrevs: BTreeMap<String, String>,
    bookmarks: BTreeMap<String, String>,
    categories: Vec<Category>,
    cat_index: usize,
    item_index: usize,
    theme_idx: usize,
    dirty: bool,
    config_path: PathBuf,
}

impl App {
    fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let config_path = PathBuf::from(&home).join(".barerc");
        let (cols, rows) = Crust::terminal_size();
        let split = 25u16;
        let lw = split - 1;
        let rx = split + 3;
        let rw = cols.saturating_sub(rx).saturating_sub(1);

        let mut app = App {
            top: Pane::new(1, 1, cols, 1, 0, 236),
            left: Pane::new(2, 3, lw, rows - 4, 255, 0),
            right: Pane::new(rx, 3, rw, rows - 4, 252, 0),
            status: Pane::new(1, rows, cols, 1, 252, 236),
            colors: THEMES[0],
            nicks: BTreeMap::new(), gnicks: BTreeMap::new(),
            abbrevs: BTreeMap::new(), bookmarks: BTreeMap::new(),
            categories: vec![], cat_index: 0, item_index: 0, theme_idx: 0,
            dirty: false, config_path,
        };
        app.left.border = true;
        app.right.border = true;
        app.load_config();
        app.build_categories();
        app.load_bool_settings();
        app
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
                // Boolean settings are loaded after build_categories
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
        // Save settings (booleans, numbers, choices)
        for cat in &self.categories {
            for item in &cat.items {
                match &item.kind {
                    ItemKind::Bool(key, val) => {
                        out += &format!("{} = {}\n", key, if *val { "true" } else { "false" });
                    }
                    ItemKind::Number(key, val) => {
                        out += &format!("{} = {}\n", key, val);
                    }
                    ItemKind::Choice(key, _, current) => {
                        out += &format!("{} = {}\n", key, current);
                    }
                    _ => {}
                }
            }
        }
        let _ = std::fs::write(&self.config_path, out);
    }

    fn load_bool_settings(&mut self) {
        let content = match std::fs::read_to_string(&self.config_path) {
            Ok(c) => c, Err(_) => return,
        };
        for line in content.lines() {
            let line = line.trim();
            if let Some((key, val)) = line.split_once('=') {
                let (key, val) = (key.trim(), val.trim());
                for cat in &mut self.categories {
                    for item in &mut cat.items {
                        match &mut item.kind {
                            ItemKind::Bool(k, ref mut v) => {
                                if *k == key { *v = val == "true" || val == "1"; }
                            }
                            ItemKind::Number(k, ref mut v) => {
                                if *k == key { if let Ok(n) = val.parse() { *v = n; } }
                            }
                            ItemKind::Choice(k, _, ref mut current) => {
                                if *k == key { *current = val.to_string(); }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn build_categories(&mut self) {
        let mut cats = vec![
            Category { name: "Theme".into(), items: vec![
                Item { label: "Theme".into(), kind: ItemKind::Theme },
            ]},
            Category { name: "Prompt Colors".into(), items:
                [0, 1, 16, 17, 2, 3, 11].iter().map(|&i| Item {
                    label: COLOR_DESCS[i].into(), kind: ItemKind::Color(i),
                }).collect(),
            },
            Category { name: "UI Colors".into(), items:
                (4..16).map(|i| Item {
                    label: COLOR_DESCS[i].into(), kind: ItemKind::Color(i),
                }).collect(),
            },
        ];
        // Alias categories
        cats.push(Category { name: "Settings".into(), items: vec![
            Item { label: "Show tips".into(), kind: ItemKind::Bool("show_tips", true) },
            Item { label: "Auto-correct".into(), kind: ItemKind::Bool("auto_correct", false) },
            Item { label: "Auto-pair".into(), kind: ItemKind::Bool("auto_pair", true) },
            Item { label: "Right prompt".into(), kind: ItemKind::Bool("rprompt", true) },
            Item { label: "Git branch".into(), kind: ItemKind::Bool("show_git_branch", false) },
            Item { label: "Fuzzy complete".into(), kind: ItemKind::Bool("completion_fuzzy", false) },
            Item { label: "Complete limit".into(), kind: ItemKind::Number("completion_limit", 10) },
            Item { label: "Slow cmd (sec)".into(), kind: ItemKind::Number("slow_command_threshold", 0) },
            Item { label: "History dedup".into(), kind: ItemKind::Choice("history_dedup",
                vec!["off".into(), "full".into(), "smart".into()], "smart".into()) },
        ]});

        let mut nick_items: Vec<Item> = self.nicks.iter()
            .map(|(k,v)| Item { label: k.clone(), kind: ItemKind::Alias("nick".into(), v.clone()) }).collect();
        if nick_items.is_empty() { nick_items.push(Item { label: "(empty)".into(), kind: ItemKind::Alias("nick".into(), String::new()) }); }
        cats.push(Category { name: "Nicks".into(), items: nick_items });

        let mut gnick_items: Vec<Item> = self.gnicks.iter()
            .map(|(k,v)| Item { label: k.clone(), kind: ItemKind::Alias("gnick".into(), v.clone()) }).collect();
        if gnick_items.is_empty() { gnick_items.push(Item { label: "(empty)".into(), kind: ItemKind::Alias("gnick".into(), String::new()) }); }
        cats.push(Category { name: "Gnicks".into(), items: gnick_items });

        let mut abbrev_items: Vec<Item> = self.abbrevs.iter()
            .map(|(k,v)| Item { label: k.clone(), kind: ItemKind::Alias("abbrev".into(), v.clone()) }).collect();
        if abbrev_items.is_empty() { abbrev_items.push(Item { label: "(empty)".into(), kind: ItemKind::Alias("abbrev".into(), String::new()) }); }
        cats.push(Category { name: "Abbrevs".into(), items: abbrev_items });

        let mut bm_items: Vec<Item> = self.bookmarks.iter()
            .map(|(k,v)| Item { label: k.clone(), kind: ItemKind::Alias("bm".into(), v.clone()) }).collect();
        if bm_items.is_empty() { bm_items.push(Item { label: "(empty)".into(), kind: ItemKind::Alias("bm".into(), String::new()) }); }
        cats.push(Category { name: "Bookmarks".into(), items: bm_items });

        self.categories = cats;
    }

    fn render(&mut self) {
        let dirty_mark = if self.dirty { " [modified]" } else { "" };
        // Prompt preview
        let preview = format!("  {}@{}: {} ({}) {} {}",
            style::fg("user", self.colors[0]), style::fg("host", self.colors[1]),
            style::fg("~/bare", self.colors[2]), style::fg("main", self.colors[11]),
            style::fg(">", self.colors[3]), style::fg("echo hello", self.colors[4]));
        self.top.say(&format!(" bareconf{}  {}", dirty_mark, preview));

        // Left: category list
        let mut lines = Vec::new();
        for (i, cat) in self.categories.iter().enumerate() {
            if i == self.cat_index {
                lines.push(style::reverse(&format!(" {} ", cat.name)));
            } else {
                lines.push(format!(" {} ", cat.name));
            }
        }
        self.left.set_text(&lines.join("\n"));
        self.left.ix = 0;
        self.left.full_refresh();

        // Right: items for selected category
        self.render_items();

        // Status
        let cat_len = self.categories.get(self.cat_index).map(|c| c.items.len()).unwrap_or(0);
        self.status.say(&format!(
            " {}/{}  j/k:item  J/K:category  h/l:change  Enter:edit  W:save  q:quit",
            self.item_index + 1, cat_len));
    }

    fn render_items(&mut self) {
        let Some(cat) = self.categories.get(self.cat_index) else { return };
        let mut lines = Vec::new();

        lines.push(style::fg(&style::bold(&cat.name), 81));
        lines.push(style::fg(&"\u{2500}".repeat(40), 245));
        lines.push(String::new());

        for (i, item) in cat.items.iter().enumerate() {
            let selected = i == self.item_index;
            let label = format!("{:<18}", item.label);

            let line = match &item.kind {
                ItemKind::Color(ci) => {
                    let c = self.colors[*ci];
                    let swatch = style::fg("\u{2588}\u{2588}\u{2588}", c);
                    let val = format!("{} {:>3}", swatch, c);
                    let al = if selected { "\u{25C0} " } else { "  " };
                    let ar = if selected { " \u{25B6}" } else { "  " };
                    format!("  {}{}{}{}",
                        if selected { style::underline(&label) } else { label }, al, val, ar)
                }
                ItemKind::Theme => {
                    let val = style::fg(THEME_NAMES[self.theme_idx], 220);
                    let al = if selected { "\u{25C0} " } else { "  " };
                    let ar = if selected { " \u{25B6}" } else { "  " };
                    format!("  {}{}{}{}",
                        if selected { style::underline(&label) } else { label }, al, val, ar)
                }
                ItemKind::Bool(_, val) => {
                    let display = if *val { style::fg("ON", 82) } else { style::fg("OFF", 196) };
                    let al = if selected { "\u{25C0} " } else { "  " };
                    let ar = if selected { " \u{25B6}" } else { "  " };
                    format!("  {}{}{}{}",
                        if selected { style::underline(&label) } else { label }, al, display, ar)
                }
                ItemKind::Number(_, val) => {
                    let al = if selected { "\u{25C0} " } else { "  " };
                    let ar = if selected { " \u{25B6}" } else { "  " };
                    format!("  {}{}{}{}",
                        if selected { style::underline(&label) } else { label }, al, val, ar)
                }
                ItemKind::Choice(_, _, current) => {
                    let al = if selected { "\u{25C0} " } else { "  " };
                    let ar = if selected { " \u{25B6}" } else { "  " };
                    format!("  {}{}{}{}",
                        if selected { style::underline(&label) } else { label },
                        al, style::fg(current, 81), ar)
                }
                ItemKind::Alias(_, val) => {
                    let v = if val.is_empty() { style::fg("-", 245) } else { val.clone() };
                    format!("  {} = {}", style::fg(&item.label, 6), v)
                }
            };
            lines.push(line);
        }

        // Color palette preview for color items
        if let Some(item) = cat.items.get(self.item_index) {
            if let ItemKind::Color(_) = &item.kind {
                lines.push(String::new());
                lines.push(style::fg("Color palette:", 245));
                // 16 rows of 16 colors, matching crush exactly
                for row in 0..16u8 {
                    let mut palette_line = String::from("  ");
                    for col in 0..16u8 {
                        let c = row * 16 + col;
                        palette_line.push_str(&style::fg("\u{2588}", c));
                    }
                    lines.push(palette_line);
                }
            }
        }

        // Theme preview, matching crush
        if let Some(item) = cat.items.get(self.item_index) {
            if let ItemKind::Theme = &item.kind {
                let t = THEME_NAMES[self.theme_idx];
                let colors = &THEMES[self.theme_idx];
                lines.push(String::new());
                lines.push(style::fg(&format!("Theme: {}", t), 220));
                lines.push(String::new());
                lines.push(format!("  {} {} {}",
                    style::fg("user", colors[0]), style::fg("@", 245),
                    style::fg("host", colors[1])));
                lines.push(format!("  {} {}",
                    style::fg("~/projects", colors[2]),
                    style::fg("(main)", colors[11])));
                lines.push(format!("  {} {}",
                    style::fg(">", colors[3]),
                    style::fg("ls -la | grep foo", colors[4])));
            }
        }

        self.right.set_text(&lines.join("\n"));
        self.right.ix = 0;
        self.right.full_refresh();
    }

    fn move_down(&mut self) {
        let len = self.categories.get(self.cat_index).map(|c| c.items.len()).unwrap_or(0);
        if self.item_index + 1 < len { self.item_index += 1; }
    }
    fn move_up(&mut self) {
        if self.item_index > 0 { self.item_index -= 1; }
    }
    fn next_category(&mut self) {
        if self.cat_index + 1 < self.categories.len() { self.cat_index += 1; self.item_index = 0; }
    }
    fn prev_category(&mut self) {
        if self.cat_index > 0 { self.cat_index -= 1; self.item_index = 0; }
    }

    fn next_value(&mut self) {
        let Some(cat) = self.categories.get_mut(self.cat_index) else { return };
        let Some(item) = cat.items.get_mut(self.item_index) else { return };
        match &mut item.kind {
            ItemKind::Color(ci) => {
                self.colors[*ci] = self.colors[*ci].wrapping_add(1);
                self.dirty = true;
            }
            ItemKind::Theme => {
                self.theme_idx = (self.theme_idx + 1) % THEME_NAMES.len();
                self.colors = THEMES[self.theme_idx];
                self.dirty = true;
            }
            ItemKind::Bool(_, val) => { *val = !*val; self.dirty = true; }
            ItemKind::Number(_, val) => { *val += 1; self.dirty = true; }
            ItemKind::Choice(_, opts, current) => {
                let idx = opts.iter().position(|o| o == current).unwrap_or(0);
                *current = opts[(idx + 1) % opts.len()].clone();
                self.dirty = true;
            }
            _ => {}
        }
    }

    fn prev_value(&mut self) {
        let Some(cat) = self.categories.get_mut(self.cat_index) else { return };
        let Some(item) = cat.items.get_mut(self.item_index) else { return };
        match &mut item.kind {
            ItemKind::Color(ci) => {
                self.colors[*ci] = self.colors[*ci].wrapping_sub(1);
                self.dirty = true;
            }
            ItemKind::Theme => {
                self.theme_idx = (self.theme_idx + THEME_NAMES.len() - 1) % THEME_NAMES.len();
                self.colors = THEMES[self.theme_idx];
                self.dirty = true;
            }
            ItemKind::Bool(_, val) => { *val = !*val; self.dirty = true; }
            ItemKind::Number(_, val) => { *val = val.saturating_sub(1); self.dirty = true; }
            ItemKind::Choice(_, opts, current) => {
                let idx = opts.iter().position(|o| o == current).unwrap_or(0);
                *current = opts[(idx + opts.len() - 1) % opts.len()].clone();
                self.dirty = true;
            }
            _ => {}
        }
    }

    fn edit_value(&mut self) {
        let Some(cat) = self.categories.get(self.cat_index) else { return };
        let Some(item) = cat.items.get(self.item_index) else { return };
        match &item.kind {
            ItemKind::Color(ci) => {
                let orig_bg = self.status.bg;
                self.status.bg = 18;
                let new_val = self.status.ask(
                    &format!("{} (0-255): ", COLOR_DESCS[*ci]),
                    &self.colors[*ci].to_string());
                self.status.bg = orig_bg;
                if let Ok(v) = new_val.parse::<u8>() {
                    self.colors[*ci] = v;
                    self.dirty = true;
                }
            }
            ItemKind::Theme => { self.next_value(); }
            ItemKind::Number(_, _) => {
                let orig_bg = self.status.bg;
                self.status.bg = 18;
                let label = item.label.clone();
                let cur = if let ItemKind::Number(_, v) = &item.kind { v.to_string() } else { String::new() };
                let new_val = self.status.ask(&format!("{}: ", label), &cur);
                self.status.bg = orig_bg;
                if let Ok(v) = new_val.parse::<u64>() {
                    if let Some(cat) = self.categories.get_mut(self.cat_index) {
                        if let Some(item) = cat.items.get_mut(self.item_index) {
                            if let ItemKind::Number(_, ref mut val) = item.kind { *val = v; }
                        }
                    }
                    self.dirty = true;
                }
            }
            _ => { self.next_value(); }
        }
    }
}

fn main() {
    Crust::init();
    let mut app = App::new();
    app.left.border_refresh();
    app.right.border_refresh();
    app.render();

    loop {
        let Some(key) = Input::getchr(None) else { continue };
        match key.as_str() {
            "q" | "ESC" => {
                if app.dirty {
                    app.status.say(&style::fg(" Save changes? (y/n)", 220));
                    if let Some(k) = Input::getchr(None) {
                        if k == "y" || k == "Y" { app.save_config(); }
                    }
                }
                break;
            }
            "j" | "DOWN" => { app.move_down(); app.render(); }
            "k" | "UP" => { app.move_up(); app.render(); }
            "J" | "PgDOWN" => { app.next_category(); app.render(); }
            "K" | "PgUP" => { app.prev_category(); app.render(); }
            "l" | "RIGHT" | "TAB" => { app.next_value(); app.render(); }
            "h" | "LEFT" | "S-TAB" => { app.prev_value(); app.render(); }
            "ENTER" => { app.edit_value(); app.render(); }
            "W" | "s" => {
                app.save_config();
                app.dirty = false;
                app.status.say(&style::fg(" Config saved", 82));
                std::thread::sleep(std::time::Duration::from_millis(500));
                app.render();
            }
            "RESIZE" => {
                let (cols, rows) = Crust::terminal_size();
                let split = 25u16;
                let lw = split - 1;
                let rx = split + 3;
                let rw = cols.saturating_sub(rx).saturating_sub(1);
                app.top = Pane::new(1, 1, cols, 1, 0, 236);
                app.left = Pane::new(2, 3, lw, rows - 4, 255, 0);
                app.right = Pane::new(rx, 3, rw, rows - 4, 252, 0);
                app.status = Pane::new(1, rows, cols, 1, 252, 236);
                app.left.border = true;
                app.right.border = true;
                Crust::clear_screen();
                app.left.border_refresh();
                app.right.border_refresh();
                app.render();
            }
            _ => {}
        }
    }

    Crust::cleanup();
}
