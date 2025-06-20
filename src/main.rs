use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::border,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Prefecture {
    name_en: String,
    name_jp: String,
    region: String,
    map_pos: (u16, u16), // (row, col) position on ASCII map
    map_char: String,    // character representation on map
    capital: String,
    population: u32,
    area_km2: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserProgress {
    prefecture_levels: HashMap<String, u8>, // prefecture name -> level (0-5)
}

#[derive(Debug)]
struct TravelStats {
    total_prefectures: usize,
    total_score: u32,
    level_counts: [usize; 6], // counts for each level 0-5
    region_stats: HashMap<String, (usize, usize)>, // region -> (visited, total)
}

impl Default for UserProgress {
    fn default() -> Self {
        Self {
            prefecture_levels: HashMap::new(),
        }
    }
}

struct JTermApp {
    prefectures: Vec<Prefecture>,
    user_progress: UserProgress,
    selected_index: usize,
    show_help: bool,
    show_map: bool,
    show_stats: bool,
    show_detail: bool,
    list_state: ratatui::widgets::ListState,
    map_scroll: u16,
    map_selected_index: usize,
}

impl JTermApp {
    fn new() -> io::Result<Self> {
        let prefectures = get_prefectures();
        let user_progress = load_user_progress()?;
        
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Ok(Self {
            prefectures,
            user_progress,
            selected_index: 0,
            show_help: false,
            show_map: false,
            show_stats: false,
            show_detail: false,
            list_state,
            map_scroll: 0,
            map_selected_index: 0,
        })
    }

    fn get_level_color(level: u8) -> Color {
        match level {
            0 => Color::Gray,
            1 => Color::Blue,
            2 => Color::Cyan,
            3 => Color::Green,
            4 => Color::Yellow,
            5 => Color::Red,
            _ => Color::White,
        }
    }

    fn get_level_text(level: u8) -> &'static str {
        match level {
            0 => "Never been there",
            1 => "Passed there",
            2 => "Alighted there", 
            3 => "Visited there",
            4 => "Stayed there",
            5 => "Lived there",
            _ => "Unknown",
        }
    }

    fn set_prefecture_level(&mut self, level: u8) {
        let index = if self.show_map {
            self.map_selected_index
        } else {
            self.selected_index
        };
        
        if let Some(prefecture) = self.prefectures.get(index) {
            self.user_progress.prefecture_levels.insert(prefecture.name_en.clone(), level);
        }
    }

    fn get_prefecture_level(&self, prefecture_name: &str) -> u8 {
        self.user_progress.prefecture_levels.get(prefecture_name).copied().unwrap_or(0)
    }

    fn save_progress(&self) -> io::Result<()> {
        save_user_progress(&self.user_progress)
    }

    fn render_map(&self) -> Vec<String> {
        let mut map_lines = Vec::new();
        let mut prefecture_index = 0;
        
        // Hokkaido
        map_lines.push("╭─────────────── HOKKAIDO REGION ─────────────────╮".to_string());
        let hokkaido_level = self.get_prefecture_level("Hokkaido");
        let hokkaido_color = match hokkaido_level {
            0 => "⬜", 1 => "🟦", 2 => "🟦", 
            3 => "🟩", 4 => "🟨", 5 => "🟥", _ => "⬜"
        };
        let hokkaido_indicator = if prefecture_index == self.map_selected_index { "►" } else { " " };
        map_lines.push(format!(" {} {} Hokkaido (北海道) - Level {} ", hokkaido_indicator, hokkaido_color, hokkaido_level));
        prefecture_index += 1;
        map_lines.push("╰─────────────────────────────────────────────────╯".to_string());
        map_lines.push("".to_string());

        // Tohoku Region
        map_lines.push("╭─────────────── TOHOKU REGION ───────────────────╮".to_string());
        let tohoku_prefectures = [
            ("Aomori", "青森"), ("Iwate", "岩手"), ("Akita", "秋田"),
            ("Miyagi", "宮城"), ("Yamagata", "山形"), ("Fukushima", "福島")
        ];
        
        for (name_en, name_jp) in &tohoku_prefectures {
            let level = self.get_prefecture_level(name_en);
            let color = match level {
                0 => "⬜", 1 => "🟦", 2 => "🟦", 
                3 => "🟩", 4 => "🟨", 5 => "🟥", _ => "⬜"
            };
            let indicator = if prefecture_index == self.map_selected_index { "►" } else { " " };
            map_lines.push(format!(" {} {} {:<8} ({}) - Level {} ", indicator, color, name_en, name_jp, level));
            prefecture_index += 1;
        }
        map_lines.push("╰─────────────────────────────────────────────────╯".to_string());
        map_lines.push("".to_string());

        // Kanto Region
        map_lines.push("╭─────────────── KANTO REGION ────────────────────╮".to_string());
        let kanto_prefectures = [
            ("Ibaraki", "茨城"), ("Tochigi", "栃木"), ("Gunma", "群馬"),
            ("Saitama", "埼玉"), ("Tokyo", "東京"), ("Chiba", "千葉"), ("Kanagawa", "神奈川")
        ];
        
        for (name_en, name_jp) in &kanto_prefectures {
            let level = self.get_prefecture_level(name_en);
            let color = match level {
                0 => "⬜", 1 => "🟦", 2 => "🟦", 
                3 => "🟩", 4 => "🟨", 5 => "🟥", _ => "⬜"
            };
            let indicator = if prefecture_index == self.map_selected_index { "►" } else { " " };
            map_lines.push(format!(" {} {} {:<8} ({}) - Level {} ", indicator, color, name_en, name_jp, level));
            prefecture_index += 1;
        }
        map_lines.push("╰─────────────────────────────────────────────────╯".to_string());
        map_lines.push("".to_string());

        // Chubu Region
        map_lines.push("╭─────────────── CHUBU REGION ────────────────────╮".to_string());
        let chubu_prefectures = [
            ("Niigata", "新潟"), ("Toyama", "富山"), ("Ishikawa", "石川"),
            ("Fukui", "福井"), ("Yamanashi", "山梨"), ("Nagano", "長野"),
            ("Gifu", "岐阜"), ("Shizuoka", "静岡"), ("Aichi", "愛知")
        ];
        
        for (name_en, name_jp) in &chubu_prefectures {
            let level = self.get_prefecture_level(name_en);
            let color = match level {
                0 => "⬜", 1 => "🟦", 2 => "🟦", 
                3 => "🟩", 4 => "🟨", 5 => "🟥", _ => "⬜"
            };
            let indicator = if prefecture_index == self.map_selected_index { "►" } else { " " };
            map_lines.push(format!(" {} {} {:<8} ({}) - Level {} ", indicator, color, name_en, name_jp, level));
            prefecture_index += 1;
        }
        map_lines.push("╰─────────────────────────────────────────────────╯".to_string());
        map_lines.push("".to_string());

        // Kansai Region
        map_lines.push("╭─────────────── KANSAI REGION ───────────────────╮".to_string());
        let kansai_prefectures = [
            ("Mie", "三重"), ("Shiga", "滋賀"), ("Kyoto", "京都"),
            ("Osaka", "大阪"), ("Hyogo", "兵庫"), ("Nara", "奈良"), ("Wakayama", "和歌山")
        ];
        
        for (name_en, name_jp) in &kansai_prefectures {
            let level = self.get_prefecture_level(name_en);
            let color = match level {
                0 => "⬜", 1 => "🟦", 2 => "🟦", 
                3 => "🟩", 4 => "🟨", 5 => "🟥", _ => "⬜"
            };
            let indicator = if prefecture_index == self.map_selected_index { "►" } else { " " };
            map_lines.push(format!(" {} {} {:<8} ({}) - Level {} ", indicator, color, name_en, name_jp, level));
            prefecture_index += 1;
        }
        map_lines.push("╰─────────────────────────────────────────────────╯".to_string());
        map_lines.push("".to_string());

        // Chugoku Region
        map_lines.push("╭─────────────── CHUGOKU REGION ──────────────────╮".to_string());
        let chugoku_prefectures = [
            ("Tottori", "鳥取"), ("Shimane", "島根"), ("Okayama", "岡山"),
            ("Hiroshima", "広島"), ("Yamaguchi", "山口")
        ];
        
        for (name_en, name_jp) in &chugoku_prefectures {
            let level = self.get_prefecture_level(name_en);
            let color = match level {
                0 => "⬜", 1 => "🟦", 2 => "🟦", 
                3 => "🟩", 4 => "🟨", 5 => "🟥", _ => "⬜"
            };
            let indicator = if prefecture_index == self.map_selected_index { "►" } else { " " };
            map_lines.push(format!(" {} {} {:<8} ({}) - Level {} ", indicator, color, name_en, name_jp, level));
            prefecture_index += 1;
        }
        map_lines.push("╰─────────────────────────────────────────────────╯".to_string());
        map_lines.push("".to_string());

        // Shikoku Region
        map_lines.push("╭─────────────── SHIKOKU REGION ──────────────────╮".to_string());
        let shikoku_prefectures = [
            ("Tokushima", "徳島"), ("Kagawa", "香川"), ("Ehime", "愛媛"), ("Kochi", "高知")
        ];
        
        for (name_en, name_jp) in &shikoku_prefectures {
            let level = self.get_prefecture_level(name_en);
            let color = match level {
                0 => "⬜", 1 => "🟦", 2 => "🟦", 
                3 => "🟩", 4 => "🟨", 5 => "🟥", _ => "⬜"
            };
            let indicator = if prefecture_index == self.map_selected_index { "►" } else { " " };
            map_lines.push(format!(" {} {} {:<8} ({}) - Level {} ", indicator, color, name_en, name_jp, level));
            prefecture_index += 1;
        }
        map_lines.push("╰─────────────────────────────────────────────────╯".to_string());
        map_lines.push("".to_string());

        // Kyushu Region
        map_lines.push("╭─────────────── KYUSHU REGION ───────────────────╮".to_string());
        let kyushu_prefectures = [
            ("Fukuoka", "福岡"), ("Saga", "佐賀"), ("Nagasaki", "長崎"),
            ("Kumamoto", "熊本"), ("Oita", "大分"), ("Miyazaki", "宮崎"), ("Kagoshima", "鹿児島")
        ];
        
        for (name_en, name_jp) in &kyushu_prefectures {
            let level = self.get_prefecture_level(name_en);
            let color = match level {
                0 => "⬜", 1 => "🟦", 2 => "🟦", 
                3 => "🟩", 4 => "🟨", 5 => "🟥", _ => "⬜"
            };
            let indicator = if prefecture_index == self.map_selected_index { "►" } else { " " };
            map_lines.push(format!(" {} {} {:<8} ({}) - Level {} ", indicator, color, name_en, name_jp, level));
            prefecture_index += 1;
        }
        map_lines.push("╰─────────────────────────────────────────────────╯".to_string());
        map_lines.push("".to_string());

        // Okinawa
        map_lines.push("╭─────────────── OKINAWA REGION ──────────────────╮".to_string());
        let okinawa_level = self.get_prefecture_level("Okinawa");
        let okinawa_color = match okinawa_level {
            0 => "⬜", 1 => "🟦", 2 => "🟦", 
            3 => "🟩", 4 => "🟨", 5 => "🟥", _ => "⬜"
        };
        let okinawa_indicator = if prefecture_index == self.map_selected_index { "►" } else { " " };
        map_lines.push(format!(" {} {} Okinawa (沖縄) - Level {} ", okinawa_indicator, okinawa_color, okinawa_level));
        map_lines.push("╰─────────────────────────────────────────────────╯".to_string());

        map_lines
    }

    fn get_prefecture_line(&self, target_index: usize) -> usize {
        let mut line_number = 0;
        let mut prefecture_index = 0;
        
        // Hokkaido
        line_number += 1; // header
        if prefecture_index == target_index { return line_number; }
        line_number += 1; // prefecture line
        prefecture_index += 1;
        line_number += 2; // border + empty line
        
        // Tohoku
        line_number += 1; // header
        for _ in 0..6 { // 6 prefectures
            if prefecture_index == target_index { return line_number; }
            line_number += 1;
            prefecture_index += 1;
        }
        line_number += 2; // border + empty line
        
        // Kanto
        line_number += 1; // header
        for _ in 0..7 { // 7 prefectures
            if prefecture_index == target_index { return line_number; }
            line_number += 1;
            prefecture_index += 1;
        }
        line_number += 2; // border + empty line
        
        // Chubu
        line_number += 1; // header
        for _ in 0..9 { // 9 prefectures
            if prefecture_index == target_index { return line_number; }
            line_number += 1;
            prefecture_index += 1;
        }
        line_number += 2; // border + empty line
        
        // Kansai
        line_number += 1; // header
        for _ in 0..7 { // 7 prefectures
            if prefecture_index == target_index { return line_number; }
            line_number += 1;
            prefecture_index += 1;
        }
        line_number += 2; // border + empty line
        
        // Chugoku
        line_number += 1; // header
        for _ in 0..5 { // 5 prefectures
            if prefecture_index == target_index { return line_number; }
            line_number += 1;
            prefecture_index += 1;
        }
        line_number += 2; // border + empty line
        
        // Shikoku
        line_number += 1; // header
        for _ in 0..4 { // 4 prefectures
            if prefecture_index == target_index { return line_number; }
            line_number += 1;
            prefecture_index += 1;
        }
        line_number += 2; // border + empty line
        
        // Kyushu
        line_number += 1; // header
        for _ in 0..7 { // 7 prefectures
            if prefecture_index == target_index { return line_number; }
            line_number += 1;
            prefecture_index += 1;
        }
        line_number += 2; // border + empty line
        
        // Okinawa
        line_number += 1; // header
        if prefecture_index == target_index { return line_number; }
        line_number += 1;
        
        line_number
    }
    
    fn ensure_selected_visible(&mut self) {
        let selected_line = self.get_prefecture_line(self.map_selected_index);
        let terminal_height = 25; // Approximate visible lines in map view
        let scroll_top = self.map_scroll as usize;
        let scroll_bottom = scroll_top + terminal_height;
        
        // If selected line is above visible area, scroll up
        if selected_line < scroll_top {
            self.map_scroll = selected_line as u16;
        }
        // If selected line is below visible area, scroll down
        else if selected_line >= scroll_bottom {
            self.map_scroll = (selected_line + 1).saturating_sub(terminal_height) as u16;
        }
    }

    fn calculate_stats(&self) -> TravelStats {
        let mut level_counts = [0; 6]; // counts for levels 0-5
        let mut region_stats = HashMap::new();
        let mut total_score = 0;

        // Initialize region stats
        for prefecture in &self.prefectures {
            region_stats.entry(prefecture.region.clone()).or_insert((0, 0)); // (visited, total)
        }

        // Calculate statistics
        for prefecture in &self.prefectures {
            let level = self.get_prefecture_level(&prefecture.name_en);
            level_counts[level as usize] += 1;
            total_score += level as u32;

            let (visited, total) = region_stats.get_mut(&prefecture.region).unwrap();
            *total += 1;
            if level > 0 {
                *visited += 1;
            }
        }

        TravelStats {
            total_prefectures: self.prefectures.len(),
            total_score,
            level_counts,
            region_stats,
        }
    }
}

fn get_prefectures() -> Vec<Prefecture> {
    vec![
        // Hokkaido
        Prefecture { 
            name_en: "Hokkaido".to_string(), 
            name_jp: "北海道".to_string(), 
            region: "Hokkaido".to_string(), 
            map_pos: (2, 30), 
            map_char: "北".to_string(),
            capital: "Sapporo".to_string(),
            population: 5250000,
            area_km2: 83424,
        },
        
        // Tohoku
        Prefecture { name_en: "Aomori".to_string(), name_jp: "青森県".to_string(), region: "Tohoku".to_string(), map_pos: (8, 32), map_char: "青".to_string(), capital: "Aomori".to_string(), population: 1240000, area_km2: 9646 },
        Prefecture { name_en: "Iwate".to_string(), name_jp: "岩手県".to_string(), region: "Tohoku".to_string(), map_pos: (10, 34), map_char: "岩".to_string(), capital: "Morioka".to_string(), population: 1200000, area_km2: 15275 },
        Prefecture { name_en: "Miyagi".to_string(), name_jp: "宮城県".to_string(), region: "Tohoku".to_string(), map_pos: (12, 32), map_char: "宮".to_string(), capital: "Sendai".to_string(), population: 2300000, area_km2: 7282 },
        Prefecture { name_en: "Akita".to_string(), name_jp: "秋田県".to_string(), region: "Tohoku".to_string(), map_pos: (10, 30), map_char: "秋".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Yamagata".to_string(), name_jp: "山形県".to_string(), region: "Tohoku".to_string(), map_pos: (12, 30), map_char: "形".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Fukushima".to_string(), name_jp: "福島県".to_string(), region: "Tohoku".to_string(), map_pos: (14, 32), map_char: "福".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        
        // Kanto
        Prefecture { name_en: "Ibaraki".to_string(), name_jp: "茨城県".to_string(), region: "Kanto".to_string(), map_pos: (16, 34), map_char: "茨".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Tochigi".to_string(), name_jp: "栃木県".to_string(), region: "Kanto".to_string(), map_pos: (16, 32), map_char: "栃".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Gunma".to_string(), name_jp: "群馬県".to_string(), region: "Kanto".to_string(), map_pos: (16, 30), map_char: "群".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Saitama".to_string(), name_jp: "埼玉県".to_string(), region: "Kanto".to_string(), map_pos: (18, 30), map_char: "埼".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Chiba".to_string(), name_jp: "千葉県".to_string(), region: "Kanto".to_string(), map_pos: (18, 34), map_char: "千".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Tokyo".to_string(), name_jp: "東京都".to_string(), region: "Kanto".to_string(), map_pos: (18, 32), map_char: "東".to_string(), capital: "Tokyo".to_string(), population: 14094034, area_km2: 2194 },
        Prefecture { name_en: "Kanagawa".to_string(), name_jp: "神奈川県".to_string(), region: "Kanto".to_string(), map_pos: (20, 32), map_char: "神".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        
        // Chubu
        Prefecture { name_en: "Niigata".to_string(), name_jp: "新潟県".to_string(), region: "Chubu".to_string(), map_pos: (14, 28), map_char: "新".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Toyama".to_string(), name_jp: "富山県".to_string(), region: "Chubu".to_string(), map_pos: (18, 26), map_char: "富".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Ishikawa".to_string(), name_jp: "石川県".to_string(), region: "Chubu".to_string(), map_pos: (18, 24), map_char: "石".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Fukui".to_string(), name_jp: "福井県".to_string(), region: "Chubu".to_string(), map_pos: (20, 24), map_char: "井".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Yamanashi".to_string(), name_jp: "山梨県".to_string(), region: "Chubu".to_string(), map_pos: (20, 30), map_char: "梨".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Nagano".to_string(), name_jp: "長野県".to_string(), region: "Chubu".to_string(), map_pos: (18, 28), map_char: "長".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Gifu".to_string(), name_jp: "岐阜県".to_string(), region: "Chubu".to_string(), map_pos: (20, 26), map_char: "岐".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Shizuoka".to_string(), name_jp: "静岡県".to_string(), region: "Chubu".to_string(), map_pos: (22, 30), map_char: "静".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Aichi".to_string(), name_jp: "愛知県".to_string(), region: "Chubu".to_string(), map_pos: (22, 28), map_char: "愛".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        
        // Kansai
        Prefecture { name_en: "Mie".to_string(), name_jp: "三重県".to_string(), region: "Kansai".to_string(), map_pos: (24, 28), map_char: "三".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Shiga".to_string(), name_jp: "滋賀県".to_string(), region: "Kansai".to_string(), map_pos: (22, 26), map_char: "滋".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Kyoto".to_string(), name_jp: "京都府".to_string(), region: "Kansai".to_string(), map_pos: (22, 24), map_char: "京".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Osaka".to_string(), name_jp: "大阪府".to_string(), region: "Kansai".to_string(), map_pos: (24, 24), map_char: "大".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Hyogo".to_string(), name_jp: "兵庫県".to_string(), region: "Kansai".to_string(), map_pos: (24, 22), map_char: "兵".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Nara".to_string(), name_jp: "奈良県".to_string(), region: "Kansai".to_string(), map_pos: (24, 26), map_char: "奈".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Wakayama".to_string(), name_jp: "和歌山県".to_string(), region: "Kansai".to_string(), map_pos: (26, 24), map_char: "和".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        
        // Chugoku
        Prefecture { name_en: "Tottori".to_string(), name_jp: "鳥取県".to_string(), region: "Chugoku".to_string(), map_pos: (24, 20), map_char: "鳥".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Shimane".to_string(), name_jp: "島根県".to_string(), region: "Chugoku".to_string(), map_pos: (26, 18), map_char: "島".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Okayama".to_string(), name_jp: "岡山県".to_string(), region: "Chugoku".to_string(), map_pos: (26, 20), map_char: "岡".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Hiroshima".to_string(), name_jp: "広島県".to_string(), region: "Chugoku".to_string(), map_pos: (26, 22), map_char: "広".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Yamaguchi".to_string(), name_jp: "山口県".to_string(), region: "Chugoku".to_string(), map_pos: (28, 18), map_char: "口".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        
        // Shikoku
        Prefecture { name_en: "Tokushima".to_string(), name_jp: "徳島県".to_string(), region: "Shikoku".to_string(), map_pos: (28, 24), map_char: "徳".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Kagawa".to_string(), name_jp: "香川県".to_string(), region: "Shikoku".to_string(), map_pos: (28, 22), map_char: "香".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Ehime".to_string(), name_jp: "愛媛県".to_string(), region: "Shikoku".to_string(), map_pos: (28, 20), map_char: "媛".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Kochi".to_string(), name_jp: "高知県".to_string(), region: "Shikoku".to_string(), map_pos: (30, 22), map_char: "高".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        
        // Kyushu
        Prefecture { name_en: "Fukuoka".to_string(), name_jp: "福岡県".to_string(), region: "Kyushu".to_string(), map_pos: (30, 16), map_char: "岡".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Saga".to_string(), name_jp: "佐賀県".to_string(), region: "Kyushu".to_string(), map_pos: (32, 16), map_char: "佐".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Nagasaki".to_string(), name_jp: "長崎県".to_string(), region: "Kyushu".to_string(), map_pos: (32, 14), map_char: "崎".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Kumamoto".to_string(), name_jp: "熊本県".to_string(), region: "Kyushu".to_string(), map_pos: (32, 18), map_char: "熊".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Oita".to_string(), name_jp: "大分県".to_string(), region: "Kyushu".to_string(), map_pos: (30, 18), map_char: "分".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Miyazaki".to_string(), name_jp: "宮崎県".to_string(), region: "Kyushu".to_string(), map_pos: (34, 18), map_char: "崎".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        Prefecture { name_en: "Kagoshima".to_string(), name_jp: "鹿児島県".to_string(), region: "Kyushu".to_string(), map_pos: (34, 16), map_char: "鹿".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
        
        // Okinawa
        Prefecture { name_en: "Okinawa".to_string(), name_jp: "沖縄県".to_string(), region: "Okinawa".to_string(), map_pos: (40, 12), map_char: "沖".to_string(), capital: "TBD".to_string(), population: 1000000, area_km2: 5000 },
    ]
}

fn get_data_dir() -> io::Result<PathBuf> {
    let mut path = dirs::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Could not find home directory")
    })?;
    path.push(".jterm");
    
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    
    Ok(path)
}

fn load_user_progress() -> io::Result<UserProgress> {
    let data_dir = get_data_dir()?;
    let progress_file = data_dir.join("progress.json");
    
    if progress_file.exists() {
        let contents = fs::read_to_string(progress_file)?;
        serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(UserProgress::default())
    }
}

fn save_user_progress(progress: &UserProgress) -> io::Result<()> {
    let data_dir = get_data_dir()?;
    let progress_file = data_dir.join("progress.json");
    
    let contents = serde_json::to_string_pretty(progress)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    fs::write(progress_file, contents)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = JTermApp::new()?;
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut JTermApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('h') | KeyCode::F(1) => app.show_help = !app.show_help,
                KeyCode::Char('m') => {
                    app.show_map = !app.show_map;
                    app.show_stats = false;
                },
                KeyCode::Char('s') => {
                    app.show_stats = !app.show_stats;
                    app.show_map = false;
                },
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.show_map {
                        if app.map_scroll > 0 {
                            app.map_scroll -= 1;
                        }
                    } else if app.selected_index > 0 {
                        app.selected_index -= 1;
                        app.list_state.select(Some(app.selected_index));
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.show_map {
                        app.map_scroll += 1;
                    } else if app.selected_index < app.prefectures.len() - 1 {
                        app.selected_index += 1;
                        app.list_state.select(Some(app.selected_index));
                    }
                }
                KeyCode::Left => {
                    if app.show_map && app.map_selected_index > 0 {
                        app.map_selected_index -= 1;
                        app.ensure_selected_visible();
                    }
                }
                KeyCode::Right => {
                    if app.show_map && app.map_selected_index < app.prefectures.len() - 1 {
                        app.map_selected_index += 1;
                        app.ensure_selected_visible();
                    }
                }
                KeyCode::Enter => {
                    app.show_detail = !app.show_detail;
                }
                KeyCode::Esc => {
                    app.show_detail = false;
                }
                KeyCode::Char('0') => {
                    app.set_prefecture_level(0);
                    app.save_progress()?;
                }
                KeyCode::Char('1') => {
                    app.set_prefecture_level(1);
                    app.save_progress()?;
                }
                KeyCode::Char('2') => {
                    app.set_prefecture_level(2);
                    app.save_progress()?;
                }
                KeyCode::Char('3') => {
                    app.set_prefecture_level(3);
                    app.save_progress()?;
                }
                KeyCode::Char('4') => {
                    app.set_prefecture_level(4);
                    app.save_progress()?;
                }
                KeyCode::Char('5') => {
                    app.set_prefecture_level(5);
                    app.save_progress()?;
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &JTermApp) {
    if app.show_map {
        render_map_view(f, app);
    } else if app.show_stats {
        render_stats_view(f, app);
    } else {
        render_list_view(f, app);
    }
    
    // Render detail popup if active
    if app.show_detail {
        render_detail_popup(f, app);
    }
}

fn render_list_view(f: &mut Frame, app: &JTermApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(f.area());

    let prefecture_items: Vec<ListItem> = app
        .prefectures
        .iter()
        .map(|prefecture| {
            let level = app.get_prefecture_level(&prefecture.name_en);
            let color = JTermApp::get_level_color(level);
            
            ListItem::new(format!(
                "{} ({}) - Level {}",
                prefecture.name_en, prefecture.name_jp, level
            ))
            .style(Style::default().fg(color))
        })
        .collect();

    let prefectures_list = List::new(prefecture_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title("Japanese Prefectures")
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray));

    f.render_stateful_widget(prefectures_list, chunks[0], &mut app.list_state.clone());

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    if let Some(selected_prefecture) = app.prefectures.get(app.selected_index) {
        let level = app.get_prefecture_level(&selected_prefecture.name_en);
        let level_text = JTermApp::get_level_text(level);
        let color = JTermApp::get_level_color(level);

        let info_text = format!(
            "Prefecture: {}\nJapanese: {}\nRegion: {}\n\nCurrent Level: {} - {}\n\nPress 0-5 to set experience level",
            selected_prefecture.name_en,
            selected_prefecture.name_jp,
            selected_prefecture.region,
            level,
            level_text
        );

        let info_paragraph = Paragraph::new(info_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(border::ROUNDED)
                    .title("Prefecture Info")
            )
            .style(Style::default().fg(color))
            .wrap(Wrap { trim: true });

        f.render_widget(info_paragraph, right_chunks[0]);
    }

    let help_text = if app.show_help {
        "Controls:\n\n↑/↓ or j/k: Navigate\nEnter: Show prefecture details\n0-5: Set experience level\nm: Toggle map view\ns: Toggle stats view\nh/F1: Toggle this help\nq: Quit\n\nLevels:\n0: Never been there (Gray)\n1: Passed there (Blue)\n2: Alighted there (Cyan)\n3: Visited there (Green)\n4: Stayed there (Yellow)\n5: Lived there (Red)"
    } else {
        "Press 'h' for help, 'm' for map, 's' for stats\nEnter for details, 0-5 for levels"
    };

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title("Help")
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, right_chunks[1]);
}

fn render_stats_view(f: &mut Frame, app: &JTermApp) {
    let stats = app.calculate_stats();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.area());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    // Overall stats
    let visited_count = stats.total_prefectures - stats.level_counts[0];
    let completion_percentage = (visited_count as f64 / stats.total_prefectures as f64 * 100.0) as u32;

    // Create colorful progress bar segments
    let bar_width: usize = 20;
    let filled_segments = (completion_percentage / 5) as usize;
    
    // Create different colored segments based on completion level
    let progress_bar = if completion_percentage >= 80 {
        format!("🟩{}🟨{}🟧{}🟥{}", 
            "█".repeat((filled_segments.min(16)).saturating_sub(12).max(0)),
            "█".repeat((filled_segments.min(12)).saturating_sub(8).max(0)), 
            "█".repeat((filled_segments.min(8)).saturating_sub(4).max(0)),
            "█".repeat(filled_segments.min(4)))
    } else if completion_percentage >= 60 {
        format!("🟨{}🟧{}🟥{}", 
            "█".repeat((filled_segments.min(12)).saturating_sub(8).max(0)),
            "█".repeat((filled_segments.min(8)).saturating_sub(4).max(0)),
            "█".repeat(filled_segments.min(4)))
    } else if completion_percentage >= 40 {
        format!("🟧{}🟥{}", 
            "█".repeat((filled_segments.min(8)).saturating_sub(4).max(0)),
            "█".repeat(filled_segments.min(4)))
    } else if completion_percentage >= 20 {
        format!("🟥{}", "█".repeat(filled_segments.min(4)))
    } else {
        "🟩🟨🟧🟥".to_string()
    };

    let overall_text = format!(
        "📊 TRAVEL STATISTICS\n\n\
        Total Prefectures: {}\n\
        Visited: {} / {} ({}%)\n\
        Total Score: {}\n\
        Max Possible: {}\n\n\
        {}{}  {}%",
        stats.total_prefectures,
        visited_count,
        stats.total_prefectures,
        completion_percentage,
        stats.total_score,
        stats.total_prefectures * 5,
        progress_bar,
        "░".repeat(bar_width.saturating_sub(filled_segments)),
        completion_percentage
    );

    let overall_paragraph = Paragraph::new(overall_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title("Overall Progress")
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    f.render_widget(overall_paragraph, top_chunks[0]);

    // Level breakdown
    let level_text = format!(
        "📈 EXPERIENCE BREAKDOWN\n\n\
        🏠 Lived there (5): {}\n\
        🏨 Stayed there (4): {}\n\
        🚶 Visited there (3): {}\n\
        🚂 Alighted there (2): {}\n\
        🚗 Passed there (1): {}\n\
        ❌ Never been (0): {}\n\n\
        Most Common: Level {}",
        stats.level_counts[5],
        stats.level_counts[4],
        stats.level_counts[3],
        stats.level_counts[2],
        stats.level_counts[1],
        stats.level_counts[0],
        stats.level_counts.iter().enumerate().max_by_key(|(_, count)| *count).unwrap().0
    );

    let level_paragraph = Paragraph::new(level_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title("Level Breakdown")
        )
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true });

    f.render_widget(level_paragraph, top_chunks[1]);

    // Regional breakdown
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(chunks[1]);

    let mut region_lines = vec!["🗾 REGIONAL PROGRESS\n".to_string()];
    
    // Define region order for better geographical organization
    let region_order = vec!["Hokkaido", "Tohoku", "Kanto", "Chubu", "Kansai", "Chugoku", "Shikoku", "Kyushu"];
    
    for region_name in region_order {
        if let Some((visited, total)) = stats.region_stats.get(region_name) {
            let percentage = (*visited as f64 / *total as f64 * 100.0) as u32;
            let bar_filled = (percentage / 8) as usize; // Smaller bars for better fit
            let bar_empty = 12 - bar_filled; // 12-char wide bars
            
            // Add region emoji for visual distinction
            let region_emoji = match region_name {
                "Hokkaido" => "❄️",
                "Tohoku" => "🌸",
                "Kanto" => "🏙️",
                "Chubu" => "🏔️",
                "Kansai" => "🏛️",
                "Chugoku" => "🌊",
                "Shikoku" => "🍊",
                "Kyushu" => "🌋",
                _ => "🗾",
            };
            
            region_lines.push(format!(
                "{} {}: {}/{} ({}%)",
                region_emoji, region_name, visited, total, percentage
            ));
            
            // Color-coded progress bars based on completion
            let bar_color = if percentage >= 80 { "🟢" } else if percentage >= 60 { "🟡" } else if percentage >= 40 { "🟠" } else { "🔴" };
            
            region_lines.push(format!(
                "{}{}{}",
                bar_color,
                "█".repeat(bar_filled),
                "░".repeat(bar_empty)
            ));
            region_lines.push("".to_string()); // Add spacing
        }
    }

    let region_text = region_lines.join("\n");

    let region_paragraph = Paragraph::new(region_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title("Regional Breakdown")
        )
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: true });

    f.render_widget(region_paragraph, bottom_chunks[0]);

    // Help section
    let help_text = if app.show_help {
        "Stats View Controls:\n\n↑/↓ or j/k: Navigate (still works)\n0-5: Set experience level\ns: Back to list view\nm: Map view\nh/F1: Toggle this help\nq: Quit\n\nYour progress is automatically saved!"
    } else {
        "Press 's' for list view\nPress 'm' for map view\nPress 'h' for help\n\nKeep exploring Japan! 🗾"
    };

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title("Help")
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, bottom_chunks[1]);
}

fn render_detail_popup(f: &mut Frame, app: &JTermApp) {
    let area = f.area();
    
    // Create a centered popup area
    let popup_width = 60;
    let popup_height = 20;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    
    let popup_area = ratatui::layout::Rect {
        x,
        y,
        width: popup_width,
        height: popup_height,
    };
    
    // Clear the background
    f.render_widget(ratatui::widgets::Clear, popup_area);
    
    let display_index = if app.show_map { app.map_selected_index } else { app.selected_index };
    if let Some(prefecture) = app.prefectures.get(display_index) {
        let level = app.get_prefecture_level(&prefecture.name_en);
        let level_text = JTermApp::get_level_text(level);
        let color = JTermApp::get_level_color(level);
        
        let detail_text = format!(
            "🏛️ PREFECTURE DETAILS\n\n\
            Name: {} ({})\n\
            Region: {}\n\
            Capital: {}\n\
            Population: {}\n\
            Area: {} km²\n\
            Population Density: {:.1} people/km²\n\n\
            Travel Experience:\n\
            Level {}: {}\n\n\
            Press ESC to close\n\
            Press 0-5 to change level",
            prefecture.name_en,
            prefecture.name_jp,
            prefecture.region,
            prefecture.capital,
            prefecture.population,
            prefecture.area_km2,
            prefecture.population as f64 / prefecture.area_km2 as f64,
            level,
            level_text
        );
        
        let popup_block = ratatui::widgets::Paragraph::new(detail_text)
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_set(border::ROUNDED)
                    .title("Prefecture Information")
                    .title_style(ratatui::style::Style::default().fg(color))
            )
            .style(ratatui::style::Style::default().fg(color))
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(popup_block, popup_area);
    }
}

fn render_map_view(f: &mut Frame, app: &JTermApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(f.area());

    // Render the map with scrolling
    let map_lines = app.render_map();
    let visible_lines: Vec<String> = map_lines
        .iter()
        .skip(app.map_scroll as usize)
        .cloned()
        .collect();
    let map_text = visible_lines.join("\n");
    
    let scroll_indicator = if app.map_scroll > 0 || visible_lines.len() > 25 {
        format!(" (Scroll: {} of {})", app.map_scroll + 1, map_lines.len().saturating_sub(25).max(1))
    } else {
        "".to_string()
    };
    
    let map_paragraph = Paragraph::new(map_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title(format!("Japan Map - Organized by Region{}", scroll_indicator))
        )
        .wrap(Wrap { trim: false });

    f.render_widget(map_paragraph, chunks[0]);

    // Right side info
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(chunks[1]);

    if let Some(selected_prefecture) = app.prefectures.get(app.map_selected_index) {
        let level = app.get_prefecture_level(&selected_prefecture.name_en);
        let level_text = JTermApp::get_level_text(level);
        let color = JTermApp::get_level_color(level);

        let info_text = format!(
            "Selected:\n{} ({})\n\nRegion: {}\n\nLevel: {} - {}\n\nKanji: {}\n\nPress 0-5 to set level",
            selected_prefecture.name_en,
            selected_prefecture.name_jp,
            selected_prefecture.region,
            level,
            level_text,
            selected_prefecture.map_char
        );

        let info_paragraph = Paragraph::new(info_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(border::ROUNDED)
                    .title("Prefecture Info")
            )
            .style(Style::default().fg(color))
            .wrap(Wrap { trim: true });

        f.render_widget(info_paragraph, right_chunks[0]);
    }

    let help_text = if app.show_help {
        "Map View Controls:\n\n↑/↓ or j/k: Scroll map\n←/→: Select prefecture\nEnter: Show prefecture details\n0-5: Set experience level\nm: Toggle to list view\ns: Stats view\nh/F1: Toggle this help\nq: Quit\n\nEmoji colors show visit levels:\n⬜ Never 🟦 Passed/Alighted\n🟩 Visited 🟨 Stayed 🟥 Lived"
    } else {
        "Press 'm' for list view\nPress 's' for stats\nPress 'h' for help\n\n↑/↓ scroll, ←/→ select\nEnter for details, 0-5 levels"
    };

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title("Help")
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, right_chunks[1]);
}
