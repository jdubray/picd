use crate::app::App;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::path::Path;
use std::time::Duration;

pub fn run(markdown_path: &Path) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = match App::new(markdown_path) {
        Ok(app) => app,
        Err(e) => {
            // Restore terminal before returning error
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            return Err(e);
        }
    };

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Poll for events with timeout to allow clipboard checking
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.select_prev();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.select_next();
                        }
                        KeyCode::Enter => {
                            app.save_to_selected()?;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Check clipboard for new images
        app.check_clipboard();

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Placeholder list
            Constraint::Length(3), // Clipboard status
            Constraint::Length(3), // Help bar
        ])
        .split(f.area());

    // Header
    let header_text = format!(
        " pictd-md - {} ({} remaining)",
        app.markdown_path.display(),
        app.remaining_count()
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Placeholder list
    render_placeholder_list(f, app, chunks[1]);

    // Clipboard status
    let clipboard_status = if app.clipboard_image.is_some() {
        let (w, h) = app.clipboard_dimensions.unwrap_or((0, 0));
        Line::from(vec![
            Span::raw(" Clipboard: "),
            Span::styled(
                format!("IMAGE READY {}x{}", w, h),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::raw(" Clipboard: "),
            Span::styled(
                "No image",
                Style::default().fg(Color::DarkGray),
            ),
        ])
    };
    let clipboard_block = Paragraph::new(clipboard_status)
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    f.render_widget(clipboard_block, chunks[2]);

    // Help bar
    let help_text = Line::from(vec![
        Span::styled(" ↑↓ ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("navigate  "),
        Span::styled("Enter ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("save  "),
        Span::styled("q ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("quit  "),
        Span::styled("| ", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.status_message, Style::default().fg(Color::Yellow)),
    ]);
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[3]);
}

fn render_placeholder_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .placeholders
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let prefix = if i == app.selected_index { "> " } else { "  " };
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(&p.relative_path, style),
                Span::styled(
                    format!("  (line {})", p.line_number),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Image Placeholders "),
        );

    f.render_widget(list, area);
}
