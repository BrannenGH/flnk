use crate::link::link_files::link_files;
use crate::link::link_options::LinkOptions;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

enum AppState {
    SelectSource,
    SelectDestination,
    Confirm,
    Complete,
}

struct App {
    state: AppState,
    files: StatefulList<PathBuf>,
    current_path: PathBuf,
    source: Option<PathBuf>,
    destination: Option<PathBuf>,
    linked_files: Vec<PathBuf>,
}

impl App {
    fn new() -> App {
        let current_path = PathBuf::from(".");
        let files = StatefulList::with_items(list_directory(&current_path).unwrap_or_default());
        App {
            state: AppState::SelectSource,
            files,
            current_path,
            source: None,
            destination: None,
            linked_files: Vec::new(),
        }
    }

    fn update_directory(&mut self) {
        self.files =
            StatefulList::with_items(list_directory(&self.current_path).unwrap_or_default());
        if self.files.items.is_empty() {
            self.files.state.select(None);
        } else {
            self.files.state.select(Some(0));
        }
    }
}

fn list_directory(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = vec![];
    if path != Path::new("/") {
        entries.push(PathBuf::from(".."));
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            entries.push(entry.path());
        }
    }
    Ok(entries)
}

pub fn run_ui(targets: &[String]) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    if !targets.is_empty() {
        app.source = Some(PathBuf::from(&targets[0]));
        app.state = AppState::SelectDestination;
    }
    if targets.len() > 1 {
        app.destination = Some(PathBuf::from(&targets[1]));
        app.state = AppState::Confirm;
    }

    let res = run_app(&mut terminal, app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.files.next(),
                KeyCode::Up => app.files.previous(),
                KeyCode::Enter => {
                    if let Some(selected) = app.files.state.selected() {
                        let selected_path = &app.files.items[selected];
                        if selected_path == &PathBuf::from("..") {
                            if let Some(parent) = app.current_path.parent() {
                                app.current_path = parent.to_path_buf();
                                app.update_directory();
                            }
                        } else {
                            match app.state {
                                AppState::SelectSource => {
                                    app.source = Some(selected_path.clone());
                                    app.state = AppState::SelectDestination;
                                }
                                AppState::SelectDestination => {
                                    app.destination = Some(selected_path.clone());
                                    app.state = AppState::Confirm;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                KeyCode::Char('y') => {
                    if let AppState::Confirm = app.state {
                        if let (Some(source), Some(dest)) = (&app.source, &app.destination) {
                            let opts = LinkOptions::default();
                            match link_files(
                                source.to_str().unwrap(),
                                dest.to_str().unwrap(),
                                Some(&opts),
                            ) {
                                Ok(linked) => {
                                    app.linked_files = linked;
                                    app.state = AppState::Complete;
                                }
                                Err(e) => {
                                    app.linked_files.clear();
                                    app.state = AppState::Complete;
                                    // Store error for display
                                    app.linked_files
                                        .push(PathBuf::from(format!("Error: {}", e)));
                                }
                            }
                        }
                    }
                }
                KeyCode::Char('n') => {
                    if let AppState::Confirm = app.state {
                        app.state = AppState::SelectSource;
                        app.source = None;
                        app.destination = None;
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.area());

    let (title, items) = match app.state {
        AppState::SelectSource => (
            "Select source directory",
            app.files
                .items
                .iter()
                .map(|p| {
                    ListItem::new(
                        p.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                    )
                })
                .collect::<Vec<_>>(),
        ),
        AppState::SelectDestination => (
            "Select destination directory",
            app.files
                .items
                .iter()
                .map(|p| {
                    ListItem::new(
                        p.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                    )
                })
                .collect::<Vec<_>>(),
        ),
        AppState::Confirm => (
            "Confirm Selection",
            vec![ListItem::new("Press 'y' to confirm or 'n' to start over")],
        ),
        AppState::Complete => (
            "Operation Complete",
            app.linked_files
                .iter()
                .map(|p| ListItem::new(format!("Linked: {}", p.display())))
                .collect::<Vec<_>>(),
        ),
    };

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    if matches!(
        app.state,
        AppState::SelectSource | AppState::SelectDestination
    ) {
        f.render_stateful_widget(items, chunks[1], &mut app.files.state);
    } else {
        f.render_widget(items, chunks[1]);
    }

    let status = match app.state {
        AppState::Complete => "Press 'q' to quit",
        _ => "Use ↑↓ to navigate, Enter to select, 'q' to quit",
    };

    let footer = Paragraph::new(status)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
