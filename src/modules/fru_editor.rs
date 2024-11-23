use serde::Deserialize;
use std::{
    fs::File, 
    io::{self, Write}
};
use crossterm::{
    event::{self, Event, KeyCode}, 
    execute, 
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};

use tui::{
    backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout, Margin}, style::{Color, Style}, text::{Span, Spans}, widgets::{Block, Borders, Paragraph}, Terminal
};


const VERSION: &str = "fru_gen utility v0.14

Copyright (C) 2024 Guanyan Wang
    
A utility to generate FRU files compatible with IPMI tool usage.

For more information, please contact: ninebro1211@gmail.com";



pub struct Line {
    immutable: String,
    editable: String,
    selected: bool,
}


#[derive(Debug, Deserialize)]
pub struct FRUEditor {
    interface_name: String
}

impl FRUEditor {
    pub fn new(interface_name: String) -> Self {
        FRUEditor {
            interface_name
        }
    }
}


pub trait UI {
    fn save_to_file(&self, lines: &[Line], filename: &str) -> io::Result<()>;
    fn run(&self, filename: &str) -> Result<(), io::Error>;
}


impl UI for FRUEditor {

    fn save_to_file(&self, lines: &[Line], filename: &str) -> io::Result<()>{
        let mut file = File::create(filename)?;
        for line in lines {
            writeln!(file,"{}\"{}\"", line.immutable, line.editable)?;
        }
        Ok(())    
    }

    /// 
    /// Display editable user interface.
    /// 
    /// # Parameters
    /// - `filename` (&str) : temp file name.
    /// 
    /// # Returns
    /// - IO ERROR or None
    ///     
    /// # Example
    /// ```
    /// let fru_editor: FRUEditor = FRUEditor::new("FRU Editor".to_string());
    /// fru_editor.run()?;
    /// ```
    fn run(&self, filename: &str) -> Result<(), io::Error>{

        enable_raw_mode()?;
        let mut stdout: io::Stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend: CrosstermBackend<io::Stdout> = CrosstermBackend::new(stdout);
        let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(backend)?;
        
        let areas: Vec<&str> = vec![
            "Chassis_type: ",
            "Chassis_Part_Number: ",
            "Chassis_Serial_Number: ",
            "Chassis_Extra: ",
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
            "Product_Extra: ",
        ];
        
        let chassis_type_table: Vec<&str> = vec![
            "Other",
            "Unknown",
            "Desktop",
            "Low Profile Desktop",
            "Pizza Box",
            "Mini Tower",
            "Tower",
            "Portable",
            "Laptop",
            "Notebook",
            "Lunch Box",
            "Main Server Chassis",
            "Expansion Chassis",
            "SubChassis",
            "Bus Expansion Chassis",
            "Peripheral Chassis",
            "RAID Chassis",
            "Rack Mount Chassis",
            "Sealed-case PC",
            "Multi-system Chassis",
            "Compact PCI",
            "Advanced TCA",
            "Blade",
            "Blade Enclosure",
            "Tablet",
            "Convertible",
            "Detachable",
            "IoT Gateway",
            "Embedded PC",
            "Mini PC",
            "Stick PC"
        ];

        
        let mut lines: Vec<Line> = areas
        .into_iter()
            .map(|immutable_data: &str| {
                Line {
                    immutable: immutable_data.to_string(),
                    editable : String::new(),
                    selected : false
                }
            }).collect();
            
        let mut cursor_x: usize = lines[0].immutable.len();
        let mut cursor_y: usize = 0;
        let mut cursor_visible: bool = true;
        
        
        loop {
            let editable_text_len = lines[cursor_y].editable.len();
            let output_content = if cursor_y == 0 {
                
                let column_size = 10;
                let columns: Vec<Vec<(usize, &str)>> = chassis_type_table
                    .chunks(column_size).enumerate().map(|(col_index, chunk)| {
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
                        table_content[i].push_str(&format!("[0x{:<02X}]: {:<25}", idx, item));
                    }
                }
                if editable_text_len > 0x3F {
                    format!("Lens of {}0x{:02X} (Exceed limitation 0x3F)\n\n=== Available type code ===\n{}", &lines[cursor_y].immutable, editable_text_len, table_content.join("\n"))
                } else {
                    format!("Lens of {}0x{:02X}\n\n=== Available type code ===\n{}", &lines[cursor_y].immutable, editable_text_len, table_content.join("\n"))
                    
                }
            } else {
                if editable_text_len > 0x3F {
                    format!("Lens of {}0x{:02X} (Exceed limitation 0x3F)", &lines[cursor_y].immutable, editable_text_len)
                } else {
                    format!("Lens of {}0x{:02X}", &lines[cursor_y].immutable, editable_text_len)
                }
            };

            for line in &mut lines {
                line.selected = false;
            }
            lines[cursor_y].selected = true;


            cursor_visible = !cursor_visible;
            if cursor_visible {
                let cursor_pos: usize = cursor_x - lines[cursor_y].immutable.len();
                lines[cursor_y].editable.insert(cursor_pos, '_');
            }
            
            terminal.draw(|frame| {
                

                // chunk[0] -- Title   block
                // chunk[1] -- Content block
                // chunk[2] -- Output  block
                let chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Percentage(50),
                            Constraint::Percentage(30)
                        ]
                        .as_ref(),
                    ).split(frame.size());
                
                    
                // Title block
                let paragarph = Paragraph::new(VERSION)
                    .style(Style::default().bg(Color::White).fg(Color::Black))
                    .block(Block::default().title("Versions").borders(Borders::ALL))
                    .alignment(Alignment::Center);
                frame.render_widget(paragarph, chunk[0]);


                let middle_background = Paragraph::new("")
                    .style(Style::default().bg(Color::White).fg(Color::Black))
                    .block(Block::default().borders(Borders::ALL).title(self.interface_name.to_string()));
                frame.render_widget(middle_background, chunk[1]);

                let line_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        (0..lines.len())
                            .map(|_| Constraint::Length(1))
                            .collect::<Vec<_>>()
                    ).split(chunk[1].inner(&Margin { vertical: (1), horizontal: (2) } ));
            
                // Content block
                for (line_index, line) in lines.iter().enumerate() {
                    let style = if line.selected == true {
                        Style::default().bg(Color::Gray).fg(Color::Black)
                    } else {
                        Style::default().bg(Color::White).fg(Color::Black)
                    };

                    let spans = Spans::from(vec![
                        Span::styled(&line.immutable, style),
                        Span::styled(&line.editable, style),
                    ]);

                    let line_paragraph = Paragraph::new(spans);
                    frame.render_widget(line_paragraph, line_chunks[line_index]);
                }


                // Output block
                let origin_title = &lines[cursor_y].immutable;
                let new_title = origin_title.strip_suffix(": ").unwrap_or(&origin_title);


                let paragraph = Paragraph::new(output_content.clone())
                    .style(Style::default().bg(Color::White).fg(Color::Black))
                    .block(Block::default().borders(Borders::ALL).title(new_title));
                frame.render_widget(paragraph, chunk[2]);
            })?;

            // Remove cursor symbol after rendering to maintain data integrity
            if cursor_visible {
                let cursor_pos: usize = cursor_x - lines[cursor_y].immutable.len();
                lines[cursor_y].editable.remove(cursor_pos);
            }

            if event::poll(std::time::Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => match key.code {
                        KeyCode::Char(c) => {

                            let editable_pos: usize = cursor_x - lines[cursor_y].immutable.len(); // Calculate the position once
                            lines[cursor_y].editable.insert(editable_pos, c); // Then, perform the insertion
                            cursor_x += 1;
                        }
                        KeyCode::Backspace => {
                            if cursor_x > lines[cursor_y].immutable.len() {
                                let editable_pos: usize = cursor_x - lines[cursor_y].immutable.len();
                                lines[cursor_y].editable.remove(editable_pos - 1);
                                cursor_x -= 1;
                            }
                        }

                        KeyCode::Enter => {
                            if cursor_y + 1 < lines.len() {
                                cursor_y += 1;
                                cursor_x = lines[cursor_y].immutable.len();
                            }
                        }
                        KeyCode::Up => {
                            if cursor_y > 0 {
                                cursor_y -= 1;
                                cursor_x = lines[cursor_y].immutable.len() + lines[cursor_y].editable.len();
                            }
                        }
                        KeyCode::Down => {
                            if cursor_y + 1 < lines.len() {
                                cursor_y += 1;
                                cursor_x = lines[cursor_y].immutable.len() + lines[cursor_y].editable.len();
                            }
                        }
                        KeyCode::Left => {
                            if cursor_x > lines[cursor_y].immutable.len() {
                                cursor_x -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if cursor_x < lines[cursor_y].immutable.len() + lines[cursor_y].editable.len() {
                                cursor_x += 1;
                            }
                        }
                        KeyCode::Esc => {
                            self.save_to_file(&lines, filename)?;
                            break;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;


        Ok(())
    }
}