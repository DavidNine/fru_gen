use serde::Deserialize;
use std::{
    fs::File, 
    io::Write,
    io,
};
use crossterm::{
    event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};

use tui::{
    backend::CrosstermBackend, style::{Color, Style}, widgets::{Block, Borders, Paragraph}, Terminal
};

pub struct Line {
    immutable: String,
    editable: String,
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
        // 初始化終端
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        


        let mut lines = vec![
            Line {
                immutable: "Chassis_type: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Chassis_Part_Number: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Chassis_Serial_Number: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Chassis_Extra: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Board_Manufacturer: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Board_Product_Name: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Board_Serial_Number: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Board_Part_Number: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Board_Fruid: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Board_Extra: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Product_Manufacturer: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Product_Name: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Product_Part_Number: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Product_Version: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Product_Serial_Number: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Product_Asset_Tag: ".to_string(),
                editable: String::new(),
            },
            Line {
                immutable: "Product_Extra: ".to_string(),
                editable: String::new(),
            },

        ];

        let mut cursor_x = lines[0].immutable.len();
        let mut cursor_y = 0;
        let mut cursor_visible = true; // Track cursor visibility


        // 主循環
        loop {
            // Toggle cursor visibility to create flashing effect
            cursor_visible = !cursor_visible;

            // Insert cursor symbol temporarily at the current position if visible
            if cursor_visible {
                let cursor_pos = cursor_x - lines[cursor_y].immutable.len();
                lines[cursor_y].editable.insert(cursor_pos, '_');
            }

            terminal.draw(|f| {
                let size = f.size();
                let block = Block::default().title(self.interface_name.clone()).borders(Borders::ALL);
                
                let content: String = lines
                    .iter()
                    .map(|line| format!("{}{}", line.immutable, line.editable))
                    .collect::<Vec<String>>()
                    .join("\n");

                let paragraph = Paragraph::new(content)
                    .style(Style::default().fg(Color::White))
                    .block(block);
                f.render_widget(paragraph, size);
            })?;

            // Remove cursor symbol after rendering to maintain data integrity
            if cursor_visible {
                let cursor_pos = cursor_x - lines[cursor_y].immutable.len();
                lines[cursor_y].editable.remove(cursor_pos);
            }


            // 處理鍵盤輸入
            if event::poll(std::time::Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => match key.code {
                        KeyCode::Char(c) => {

                            let editable_pos = cursor_x - lines[cursor_y].immutable.len(); // Calculate the position once
                            lines[cursor_y].editable.insert(editable_pos, c); // Then, perform the insertion
                            cursor_x += 1;
                        }
                        KeyCode::Backspace => {
                            if cursor_x > lines[cursor_y].immutable.len() {
                                let editable_pos = cursor_x - lines[cursor_y].immutable.len();
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