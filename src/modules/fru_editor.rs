use super::{area::Area, board_area::{Board, parse_mfg_time}, chassis_area::Chassis, product_area::Product};
use crate::{parse_chassis_type, CHASSIS_TYPE_TABLE, ConfigField};
use chrono::{Duration, TimeZone, Utc};
use crossterm::{
    event::{self, EnableMouseCapture, DisableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Write},
};

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

const VERSION: &str = "fru_gen utility v1.0.0

Copyright (C) 2026 Guanyan Wang
    
A utility to generate FRU files compatible with IPMI tool usage.

For more information, please contact: ninebro1211@gmail.com";

pub struct Line {
    immutable: String,
    editable: String,
    selected: bool,
    // Settings
    enabled: bool,
    reserved_bytes: usize,
}

impl Line {
    pub fn enabled(&self) -> bool { self.enabled }
    pub fn reserved_bytes(&self) -> usize { self.reserved_bytes }
}

#[derive(PartialEq, Debug)]
pub enum Page {
    Editor,
    Settings,
}

#[derive(Debug, Deserialize)]
pub struct FRUEditor {
    interface_name: String,
}

pub enum EventOutcome {
    Continue,
    Save,
    Exit,
}

impl FRUEditor {
    pub fn new(interface_name: String) -> Self {
        FRUEditor { interface_name }
    }

    fn immutable_width(lines: &[Line], cursor_y: usize) -> usize {
        lines[cursor_y].immutable.len()
    }

    fn editable_cursor_pos(lines: &[Line], cursor_x: usize, cursor_y: usize) -> usize {
        cursor_x - Self::immutable_width(lines, cursor_y)
    }

    fn build_output_hint(lines: &[Line], cursor_y: usize, chassis_type_table: &[&str]) -> String {
        let editable_text_len = lines[cursor_y].editable.len();
        let length_info = if editable_text_len > 0x3F {
            format!("Length: 0x{:02X} (EXCEEDS 0x3F!)", editable_text_len)
        } else {
            format!("Length: 0x{:02X}", editable_text_len)
        };

        if cursor_y == 0 {
            let column_size = 16;
            let columns: Vec<Vec<(usize, &str)>> = chassis_type_table
                .chunks(column_size)
                .enumerate()
                .map(|(col_index, chunk)| {
                    chunk
                        .iter()
                        .enumerate()
                        .map(|(row_index, item)| (col_index * column_size + row_index, *item))
                        .collect()
                })
                .collect();

            let mut table_content = vec![String::new(); column_size];
            for col in columns {
                for (i, (idx, item)) in col.iter().enumerate() {
                    table_content[i].push_str(&format!("[0x{:<02X}]: {:<20} ", idx, item));
                }
            }

            format!(
                "{}\nHint: Enter code (e.g., 0x11) or name.\n\n=== Type Codes ===\n{}",
                length_info,
                table_content.join("\n")
            )
        } else if cursor_y == 4 {
            format!(
                "{}\nHint: Enter timestamp (YYYYMMDDHHMMSS) or minutes since 1996.",
                length_info
            )
        } else {
            format!(
                "{}\nHint: Stored as plain text string.",
                length_info
            )
        }
    }

    fn sanitize_label(label: &str) -> &str {
        label.strip_suffix(": ").unwrap_or(label)
    }

    fn selected_field_hex(lines: &[Line], cursor_y: usize) -> String {
        let label = Self::sanitize_label(&lines[cursor_y].immutable);
        let value = &lines[cursor_y].editable;
        let line = &lines[cursor_y];

        if !line.enabled {
            return format!("Selected Field: {label}\nStatus        : DISABLED (will not be in binary)");
        }

        // Special handling for non-string fields
        // 0: Chassis_type (1 byte)
        // 4: Board_Mfg_Date_Time (3 bytes)
        if cursor_y == 0 || cursor_y == 4 {
            let code = if cursor_y == 4 {
                parse_mfg_time(value)
            } else {
                if value.starts_with("0x") || value.starts_with("0X") {
                    u32::from_str_radix(&value[2..], 16).unwrap_or(0)
                } else {
                    value.parse::<u32>().unwrap_or(0)
                }
            };

            let mut extra_info = String::new();
            let mut hex_val = format!("{:02X}", code & 0xFF);
            if cursor_y == 4 {
                hex_val = format!("{:06X}", code & 0xFFFFFF);
                if code == 0 {
                    extra_info = " (Unspecified)".to_string();
                } else {
                    let epoch = Utc.with_ymd_and_hms(1996, 1, 1, 0, 0, 0).single().unwrap();
                    let date = epoch + Duration::minutes(code as i64);
                    extra_info = format!(" ({})", date.format("%Y-%m-%d %H:%M"));
                }
            }
            return format!(
                "Selected Field: {label}\nInput value : {value}\nHex value   : {}{}",
                hex_val,
                extra_info
            );
        }

        // String fields with dynamic reservation
        let text_hex = if value.is_empty() {
            "<empty>".to_string()
        } else {
            value
                .as_bytes()
                .iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<_>>()
                .join(" ")
        };

        let mut bytes = value.as_bytes().to_vec();
        if line.reserved_bytes > 0 && bytes.len() < line.reserved_bytes {
            bytes.resize(line.reserved_bytes, b' ');
        }
        let len = bytes.len().min(0x3F);
        let mut encoded = vec![0xC0 | len as u8];
        encoded.extend_from_slice(&bytes[..len]);

        let fru_hex = encoded
            .iter()
            .map(|byte| format!("{:02X}", byte))
            .collect::<Vec<_>>()
            .join(" ");

        format!("Selected Field: {label}\nText bytes  : {text_hex}\nFRU bytes   : {fru_hex}")
    }

    fn build_hex_view(lines: &[Line], cursor_y: usize) -> String {
        let field_preview = Self::selected_field_hex(lines, cursor_y);

        if lines
            .iter()
            .filter(|line| !line.editable.is_empty())
            .any(|line| line.editable.len() > 0x3F)
        {
            return format!("{field_preview}\n\nFull FRU preview unavailable: one or more fields exceed 0x3F bytes.");
        }

        let chassis_type_code = parse_chassis_type(&lines[0].editable);
        let chassis = Chassis::new(
            chassis_type_code,
            lines[1].editable.clone(),
            lines[2].editable.clone(),
            lines[3].editable.clone(),
        );
        let board = Board::new(
            lines[4].editable.clone(),
            lines[5].editable.clone(),
            lines[6].editable.clone(),
            lines[7].editable.clone(),
            lines[8].editable.clone(),
            lines[9].editable.clone(),
            lines[10].editable.clone(),
        );
        let product = Product::new(
            lines[11].editable.clone(),
            lines[12].editable.clone(),
            lines[13].editable.clone(),
            lines[14].editable.clone(),
            lines[15].editable.clone(),
            lines[16].editable.clone(),
            lines[17].editable.clone(),
            lines[18].editable.clone(),
        );

        let get_configs = |range: std::ops::Range<usize>| -> Vec<super::area::FieldConfig> {
            lines[range].iter().map(|l| super::area::FieldConfig {
                enabled: l.enabled,
                reserved_bytes: l.reserved_bytes,
            }).collect()
        };

        let chassis_bytes = chassis.transfer_with_config(&get_configs(0..4));
        let board_bytes = board.transfer_with_config(&get_configs(4..11));
        let product_bytes = product.transfer_with_config(&get_configs(11..19));

        let mut fru_data = vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut current_offset = 1u8;

        if !chassis_bytes.is_empty() {
            fru_data[2] = current_offset;
            fru_data.extend(&chassis_bytes);
            current_offset += (chassis_bytes.len() / 8) as u8;
        }

        if !board_bytes.is_empty() {
            fru_data[3] = current_offset;
            fru_data.extend(&board_bytes);
            current_offset += (board_bytes.len() / 8) as u8;
        }

        if !product_bytes.is_empty() {
            fru_data[4] = current_offset;
            fru_data.extend(&product_bytes);
        }

        fru_data[7] = ((0x100u16 - (fru_data.iter().take(7).map(|&b| b as u16).sum::<u16>() % 256)) % 256) as u8;

        while fru_data.len() < 256 {
            fru_data.push(0x00);
        }

        // Format the fru_data as hex dump (hexdump -C style)
        let mut hex_dump = String::new();
        for (i, chunk) in fru_data.chunks(16).enumerate() {
            let offset = i * 16;
            
            let mut hex_part = String::new();
            let mut ascii_part = String::new();
            
            for (j, &byte) in chunk.iter().enumerate() {
                hex_part.push_str(&format!("{:02X} ", byte));
                if j == 7 {
                    hex_part.push(' ');
                }
                
                if byte >= 32 && byte <= 126 {
                    ascii_part.push(byte as char);
                } else {
                    ascii_part.push('.');
                }
            }
            
            hex_dump.push_str(&format!("{:04X}  {:<49} |{}|\n", offset, hex_part, ascii_part));
        }

        format!("{field_preview}\n\nFull FRU Data Preview (256 bytes):\n{hex_dump}")
    }

    fn handle_key_event(
        &self,
        event: event::KeyEvent,
        lines: &mut [Line],
        cursor_x: &mut usize,
        cursor_y: &mut usize,
        hint_scroll: &mut u16,
        current_page: &mut Page,
    ) -> EventOutcome {
        if event.code == KeyCode::Tab {
            *current_page = if *current_page == Page::Editor {
                Page::Settings
            } else {
                Page::Editor
            };
            *cursor_x = lines[*cursor_y].immutable.len();
            return EventOutcome::Continue;
        }

        if event.modifiers.contains(KeyModifiers::CONTROL) {
            match event.code {
                KeyCode::Char('s') | KeyCode::Char('S') => return EventOutcome::Save,
                _ => {}
            }
        }

        if *current_page == Page::Settings {
            match event.code {
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    lines[*cursor_y].enabled = !lines[*cursor_y].enabled;
                }
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    if lines[*cursor_y].reserved_bytes < 0x3F {
                        lines[*cursor_y].reserved_bytes += 1;
                    }
                }
                KeyCode::Char('-') | KeyCode::Char('_') => {
                    if lines[*cursor_y].reserved_bytes > 0 {
                        lines[*cursor_y].reserved_bytes -= 1;
                    }
                }
                KeyCode::Up => {
                    if *cursor_y > 0 {
                        *cursor_y -= 1;
                        *hint_scroll = 0;
                    }
                }
                KeyCode::Down => {
                    if *cursor_y + 1 < lines.len() {
                        *cursor_y += 1;
                        *hint_scroll = 0;
                    }
                }
                KeyCode::Esc => return EventOutcome::Exit,
                _ => {}
            }
            return EventOutcome::Continue;
        }

        match event.code {
            KeyCode::Char(c) => {
                let editable_pos = Self::editable_cursor_pos(lines, *cursor_x, *cursor_y);
                lines[*cursor_y].editable.insert(editable_pos, c);
                *cursor_x += 1;
                EventOutcome::Continue
            }
            KeyCode::Backspace => {
                let immutable_width = Self::immutable_width(lines, *cursor_y);
                if *cursor_x > immutable_width {
                    let editable_pos = Self::editable_cursor_pos(lines, *cursor_x, *cursor_y);
                    lines[*cursor_y].editable.remove(editable_pos - 1);
                    *cursor_x -= 1;
                }
                EventOutcome::Continue
            }
            KeyCode::Enter => {
                if *cursor_y + 1 < lines.len() {
                    *cursor_y += 1;
                    *cursor_x =
                        Self::immutable_width(lines, *cursor_y) + lines[*cursor_y].editable.len();
                    *hint_scroll = 0;
                }
                EventOutcome::Continue
            }
            KeyCode::Up => {
                if *cursor_y > 0 {
                    *cursor_y -= 1;
                    *cursor_x =
                        Self::immutable_width(lines, *cursor_y) + lines[*cursor_y].editable.len();
                    *hint_scroll = 0;
                }
                EventOutcome::Continue
            }
            KeyCode::Down => {
                if *cursor_y + 1 < lines.len() {
                    *cursor_y += 1;
                    *cursor_x =
                        Self::immutable_width(lines, *cursor_y) + lines[*cursor_y].editable.len();
                    *hint_scroll = 0;
                }
                EventOutcome::Continue
            }
            KeyCode::PageUp => {
                if *hint_scroll > 0 {
                    *hint_scroll -= 1;
                }
                EventOutcome::Continue
            }
            KeyCode::PageDown => {
                *hint_scroll += 1;
                EventOutcome::Continue
            }
            KeyCode::Left => {
                if *cursor_x > Self::immutable_width(lines, *cursor_y) {
                    *cursor_x -= 1;
                }
                EventOutcome::Continue
            }
            KeyCode::Right => {
                if *cursor_x
                    < Self::immutable_width(lines, *cursor_y) + lines[*cursor_y].editable.len()
                {
                    *cursor_x += 1;
                }
                EventOutcome::Continue
            }
            KeyCode::Esc => EventOutcome::Exit,
            _ => EventOutcome::Continue,
        }
    }
}

pub trait UI {
    fn save_to_file(&self, lines: &[Line], filename: &str) -> io::Result<()>;
    fn run(&self, filename: &str, initial_data: Option<HashMap<String, ConfigField>>) -> Result<Option<Vec<Line>>, io::Error>;
}

impl UI for FRUEditor {
    fn save_to_file(&self, lines: &[Line], filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        for line in lines {
            let key = line.immutable.strip_suffix(": ").unwrap_or(&line.immutable);
            writeln!(file, "{}: \"{}\"", key, line.editable)?;
        }
        Ok(())
    }

    fn run(&self, filename: &str, initial_data: Option<HashMap<String, ConfigField>>) -> Result<Option<Vec<Line>>, io::Error> {
        enable_raw_mode()?;
        let mut stdout: io::Stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend: CrosstermBackend<io::Stdout> = CrosstermBackend::new(stdout);
        let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(backend)?;

        let areas: Vec<&str> = vec![
            "Chassis_type: ",
            "Chassis_Part_Number: ",
            "Chassis_Serial_Number: ",
            "Chassis_Extra: ",
            "Board_Mfg_Date_Time: ",
            "Board_Manufacturer: ",
            "Board_Product_Name: ",
            "Board_Serial_Number: ",
            "Board_Part_Number: ",
            "Board_Fruid: ",
            "Board_Extra: ",
            "Product_Manufacturer: ",
            "Product_Name: ",
            "Product_Part_Number: ",
            "Product_Version: ",
            "Product_Serial_Number: ",
            "Product_Asset_Tag: ",
            "Product_Fruid: ",
            "Product_Extra: ",
        ];

        let mut lines: Vec<Line> = areas
            .into_iter()
            .map(|immutable_data: &str| {
                let key = immutable_data.strip_suffix(": ").unwrap_or(immutable_data).to_lowercase();
                
                let is_code = key.contains("type") || key.contains("mfg");
                let default_reserve = if is_code { 0 } else { 32 };

                let (editable, enabled, reserved_bytes) = if let Some(ref data) = initial_data {
                    if let Some(field) = data.get(&key) {
                        (field.value(), true, field.reserve_bytes().unwrap_or(default_reserve))
                    } else {
                        (String::new(), false, default_reserve)
                    }
                } else {
                    (String::new(), true, default_reserve)
                };
                
                Line {
                    immutable: immutable_data.to_string(),
                    editable,
                    selected: false,
                    enabled,
                    reserved_bytes,
                }
            })
            .collect();

        let mut cursor_x: usize = lines[0].immutable.len() + lines[0].editable.len();
        let mut cursor_y: usize = 0;
        let mut cursor_visible: bool = true;
        let mut hint_scroll: u16 = 0;
        let mut hex_scroll: u16 = 0;
        let mut current_page = Page::Editor;
        let mut saved_lines: Option<Vec<Line>> = None;

        loop {
            let output_content = if current_page == Page::Editor {
                Self::build_output_hint(&lines, cursor_y, CHASSIS_TYPE_TABLE)
            } else {
                let line = &lines[cursor_y];
                format!(
                    "Field: {}\n\nEnabled: {}\nReserved Bytes: {}\n\nInstructions:\n'e'     : Toggle Enable/Disable\n'+' / '-': Inc/Dec Reserved Bytes",
                    line.immutable.trim(),
                    if line.enabled { "YES" } else { "NO" },
                    line.reserved_bytes
                )
            };

            let hex_content = Self::build_hex_view(&lines, cursor_y);

            for line in &mut lines {
                line.selected = false;
            }
            lines[cursor_y].selected = true;

            cursor_visible = !cursor_visible;
            if cursor_visible && current_page == Page::Editor {
                let cursor_pos = Self::editable_cursor_pos(&lines, cursor_x, cursor_y);
                lines[cursor_y].editable.insert(cursor_pos, '_');
            }

            let mut hex_view_area = tui::layout::Rect::default();

            terminal.draw(|frame| {
                let chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Percentage(50),
                            Constraint::Percentage(30),
                        ]
                        .as_ref(),
                    )
                    .split(frame.size());

                let title = format!("{} - [{:?}]", self.interface_name, current_page);
                let paragarph = Paragraph::new(VERSION)
                    .style(Style::default().fg(Color::Cyan))
                    .block(
                        Block::default()
                            .title("About")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray)),
                    )
                    .alignment(Alignment::Center);
                frame.render_widget(paragarph, chunk[0]);

                let middle_background = Paragraph::new("")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Green))
                            .title(title),
                    );
                frame.render_widget(middle_background, chunk[1]);

                let middle_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(52), Constraint::Percentage(48)].as_ref())
                    .split(chunk[1].inner(&Margin {
                        vertical: (1),
                        horizontal: (2),
                    }));

                hex_view_area = middle_chunks[1];

                let line_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        (0..lines.len())
                            .map(|_| Constraint::Length(1))
                            .collect::<Vec<_>>(),
                    )
                    .split(middle_chunks[0]);

                for (line_index, line) in lines.iter().enumerate() {
                    let mut style = if line.selected == true {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Reset)
                    };

                    if !line.enabled {
                        style = style.add_modifier(Modifier::DIM);
                    }

                    let content = if current_page == Page::Editor {
                        format!("{}{}", line.immutable, line.editable)
                    } else {
                        format!("{} [Enabled: {:<3}] [Reserve: {:>2}]", 
                            line.immutable, 
                            if line.enabled { "YES" } else { "NO" },
                            line.reserved_bytes
                        )
                    };

                    let line_paragraph = Paragraph::new(content).style(style);
                    frame.render_widget(line_paragraph, line_chunks[line_index]);
                }

                let hex_paragraph = Paragraph::new(hex_content.clone())
                    .scroll((hex_scroll, 0))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title("Hex View")
                    );
                frame.render_widget(hex_paragraph, middle_chunks[1]);

                let bottom_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                    .split(chunk[2]);

                let origin_title = &lines[cursor_y].immutable;
                let new_title = origin_title.strip_suffix(": ").unwrap_or(&origin_title);
                let hint_title = format!("{} (PgUp/PgDn)", new_title);

                let hint_paragraph = Paragraph::new(output_content.clone())
                    .scroll((hint_scroll, 0))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Blue))
                            .title(hint_title)
                    );
                frame.render_widget(hint_paragraph, bottom_chunks[0]);

                let nav_hint = "\
↑/↓    : Select field
Enter  : Next field
Tab    : Switch Page
←/→    : Move cursor
Bs     : Delete
Ctrl+S : Save
Esc    : Exit
PgUp/Dn: Scroll hint";

                let nav_paragraph = Paragraph::new(nav_hint)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title("Controls")
                    );
                frame.render_widget(nav_paragraph, bottom_chunks[1]);
            })?;

            if cursor_visible && current_page == Page::Editor {
                let cursor_pos = Self::editable_cursor_pos(&lines, cursor_x, cursor_y);
                lines[cursor_y].editable.remove(cursor_pos);
            }

            if event::poll(std::time::Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => match self.handle_key_event(
                        key,
                        &mut lines,
                        &mut cursor_x,
                        &mut cursor_y,
                        &mut hint_scroll,
                        &mut current_page,
                    ) {
                        EventOutcome::Save => {
                            self.save_to_file(&lines, filename)?;
                            // Store the settings for potential use by caller
                            saved_lines = Some(lines.iter().map(|l| Line {
                                immutable: l.immutable.clone(),
                                editable: l.editable.clone(),
                                selected: false,
                                enabled: l.enabled,
                                reserved_bytes: l.reserved_bytes,
                            }).collect());
                        }
                        EventOutcome::Exit => break,
                        EventOutcome::Continue => {}
                    },
                    Event::Mouse(mouse) => {
                        if mouse.column >= hex_view_area.left() && mouse.column < hex_view_area.right() &&
                           mouse.row >= hex_view_area.top() && mouse.row < hex_view_area.bottom() {
                            match mouse.kind {
                                MouseEventKind::ScrollDown => {
                                    hex_scroll += 1;
                                }
                                MouseEventKind::ScrollUp => {
                                    if hex_scroll > 0 {
                                        hex_scroll -= 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
        terminal.show_cursor()?;

        Ok(saved_lines)
    }
}
