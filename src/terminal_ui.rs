use crate::loader::Stat;
use anyhow::Result;
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::{error::TryRecvError, UnboundedReceiver as Receiver};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment},
    style::{Style, Stylize as _},
    terminal::{Frame, Terminal},
    widgets::{block::Title, Block, RenderDirection, Sparkline},
};

#[derive(Debug)]
enum AppState {
    Running,
    Stopped,
}
struct App {
    rx: Receiver<Stat>,
    data: Vec<u64>,
    state: AppState,
}

impl App {
    fn new(rx: Receiver<Stat>) -> Self {
        Self {
            rx,
            data: Vec::with_capacity(200),
            state: AppState::Running,
        }
    }

    fn on_tick(&mut self) -> Result<()> {
        let mut count = 0;
        loop {
            match self.rx.try_recv() {
                Err(TryRecvError::Empty) => break,
                Err(_) => {
                    self.state = AppState::Stopped;
                    return Ok(());
                }
                Ok(Stat { size }) => count += size,
            }
        }
        self.data.insert(0, count as _);
        Ok(())
    }
}

pub fn start_terminal(rx: Receiver<Stat>) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_secs(1);
    let app = App::new(rx);
    let res = run_app(&mut terminal, app, tick_rate);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(res?)
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick()?;
            last_tick = Instant::now();
        }
    }
}

fn ui(frame: &mut Frame, app: &App) {
    let area = frame.size();
    let (min, avg, max) = app
        .data
        .iter()
        .copied()
        .fold((u64::MAX, 0, 0), |(min, avg, max), value| {
            (min.min(value), avg + value, max.max(value))
        });
    let avg = avg / app.data.len().max(1) as u64;
    let last = match app.data.first() {
        Some(val) => *val,
        None => 0,
    };
    let min = match min {
        min if min == u64::MAX => 0,
        min => min,
    };
    let chart = Sparkline::default()
        .block(
            Block::bordered()
                .title("Rate")
                .title(Title::from(format!("|state: {:?}", app.state)).alignment(Alignment::Left))
                .title(Title::from(format!("|min: {min}")).alignment(Alignment::Left))
                .title(Title::from(format!("|avg: {avg}")).alignment(Alignment::Left))
                .title(Title::from(format!("|max: {max}")).alignment(Alignment::Left))
                .title(Title::from(format!("|last: {last}")).alignment(Alignment::Left)),
        )
        .data(&app.data)
        .direction(RenderDirection::RightToLeft)
        .style(Style::default().red());
    frame.render_widget(chart, area);
}
