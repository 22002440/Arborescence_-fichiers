use std::{error::Error, io};
use std::process::Command;



use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use clap::{Parser, Arg};

#[derive(Parser, Debug)]
struct ShellCommand {
    #[arg(short, long)]
    lexicographic_sort: bool,

    #[arg(short, long)]
    filter: Option<String>,
}


enum InputMode {
    Normal,
    Editing,
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    cursor_position: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded output
    output: Vec<String>,
    /// Position dans l'historique de sortie à partir de laquelle afficher
    output_view_position: usize,
    ///commande possible
    possible_commands: &'static str,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            output: Vec::new(),
            cursor_position: 0,
            output_view_position: 0,
            possible_commands: "Possible commands:\n\
                               - cargo run --bin main -- usage option<path>\n\
                               - cargo run --bin --main -- --lexicographic-sort usage option<path>\n\
                               - cargo run --bin main  -- option<--lexicographic-sort> --filter jpg usage option<path>\n\
                               - cargo run --bin main -- duplicate",
        }
    }
}

impl App {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn submit_message(&mut self) {
        self.output.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

    fn run_shell_command(&mut self) {
        self.output.clear();
        if !self.input.trim().is_empty() {
            let output = Command::new("cmd")
                .arg("/C")
                .arg(&self.input)
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        let result = String::from_utf8_lossy(&output.stdout);
                        for line in result.lines() {
                            self.output.push(line.to_owned());
                        }
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr).into_owned();
                        self.output.push(format!("Error: {}", error));
                    }
                }
                Err(err) => {
                    self.output.push(format!("Error executing command: {}", err));
                }
            }

            self.input.clear();
            self.reset_cursor();
        }
    }

    fn scroll_output(&mut self, lines: isize) {
        // Faites défiler l'historique en ajustant la position de vue
        let new_position = self.output_view_position as isize + lines;
        self.output_view_position = new_position.clamp(0, self.output.len() as isize) as usize;
    }

}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Up => {
                        if app.output_view_position > 0 {
                            app.output_view_position -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if app.output_view_position < app.output.len() - 26 {
                            app.output_view_position += 1;
                        }
                    }
                    KeyCode::Right => {
                        if app.output_view_position < app.output.len() - 26 {
                            app.output_view_position += 25;
                        }
                        else {
                            app.output_view_position = app.output.len() - 26;
                        }
                    }
                    KeyCode::Left => {
                        if app.output_view_position > 25 {
                            app.output_view_position -= 25;
                        }
                        else {
                            app.output_view_position = 0;
                        }
                    }
                    _ => {}
                },
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        app.run_shell_command();
                    },
                    KeyCode::Char(to_insert) => {
                        app.enter_char(to_insert);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                    }
                    KeyCode::Left => {
                        app.move_cursor_left();
                    }
                    KeyCode::Right => {
                        app.move_cursor_right();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Up => {
                        if app.output_view_position > 0 {
                            app.output_view_position -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if app.output_view_position < app.output.len() - 1 {
                            app.output_view_position += 1;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1),
        Constraint::Length(29),
        Constraint::Length(7),
        Constraint::Min(1),
        
    ])
    .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                "Press ".into(),
                "q".bold(),
                " to exit, ".into(),
                "e".bold(),
                " to start editing.".bold(),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to record the message".into(),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Line::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[3]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            f.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                chunks[3].x + app.cursor_position as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[3].y + 1,
            )
        }
    }

    let visible_output: Vec<ListItem> = app
        .output
        .iter()
        .skip(app.output_view_position)
        .take(27) 
        .enumerate()
        .map(|(i, m)| {
            let content = Line::from(Span::raw(format!("{}: {}", i + app.output_view_position, m)));
            ListItem::new(content)
        })
        .collect();

    let output = List::new(visible_output).block(Block::default().borders(Borders::ALL).title("Output"));
    f.render_widget(output, chunks[1]);

    let commands = Paragraph::new(app.possible_commands)
        .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Possible Commands"));
    f.render_widget(commands, chunks[2]);
}
