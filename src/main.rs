use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, queue};
use rust_terminal_snake_game::{
    apply_direction, log_size_crash, previous_high, read_leaderboard, record_new_high, snake_char,
    start_state, terminal_size_ok, Direction, GameOutcome, SIZE_ERROR_MESSAGE,
};
use std::io::{self, stdout, Stdout, Write};
use std::path::Path;
use std::time::{Duration, Instant};

const TICK_MS: u64 = 120;

enum RunResult {
    Win(u32),
    Loss(u32),
    RuntimeTooSmall(u16, u16),
}

struct TerminalGuard {
    stdout: Stdout,
}

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        Ok(Self { stdout })
    }

    fn out(&mut self) -> &mut Stdout {
        &mut self.stdout
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(self.stdout, Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn render_frame(out: &mut Stdout, state: &rust_terminal_snake_game::GameState) -> io::Result<()> {
    queue!(out, MoveTo(0, 0), Clear(ClearType::All))?;

    for (idx, part) in state.snake.iter().enumerate() {
        if part.x >= 0 && part.y >= 0 {
            queue!(
                out,
                MoveTo(part.x as u16, part.y as u16),
                Print(snake_char(idx))
            )?;
        }
    }

    queue!(
        out,
        MoveTo(state.apple.x as u16, state.apple.y as u16),
        Print('*')
    )?;

    out.flush()?;
    Ok(())
}

fn wait_for_start(out: &mut Stdout, width: u16, height: u16) -> io::Result<()> {
    let msg = "Press any key to start.";
    let x = width.saturating_sub(msg.len() as u16) / 2;
    let y = height / 2;
    queue!(out, MoveTo(0, 0), Clear(ClearType::All), MoveTo(x, y), Print(msg))?;
    out.flush()?;

    loop {
        if let Event::Key(k) = event::read()? {
            if k.kind == KeyEventKind::Press {
                break;
            }
        }
    }
    Ok(())
}

fn key_to_direction(code: KeyCode) -> Option<Direction> {
    match code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    }
}

fn run_game(width: u16, height: u16) -> io::Result<RunResult> {
    let mut terminal = TerminalGuard::new()?;
    wait_for_start(terminal.out(), width, height)?;

    let mut state = start_state(width, height)
        .ok_or_else(|| io::Error::other("Unable to allocate initial apple"))?;
    render_frame(terminal.out(), &state)?;

    let mut pending_direction: Option<Direction> = None;
    let tick_duration = Duration::from_millis(TICK_MS);
    let mut last_tick = Instant::now();

    loop {
        let (cur_w, cur_h) = terminal::size()?;
        if cur_w < width || cur_h < height {
            return Ok(RunResult::RuntimeTooSmall(cur_w, cur_h));
        }

        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(k) = event::read()? {
                if k.kind == KeyEventKind::Press {
                    pending_direction = key_to_direction(k.code).or(pending_direction);
                }
            }
        }

        if last_tick.elapsed() < tick_duration {
            continue;
        }
        last_tick = Instant::now();

        apply_direction(&mut state, pending_direction.take());
        let outcome = rust_terminal_snake_game::tick(&mut state);
        match outcome {
            GameOutcome::Continue => {}
            GameOutcome::Win => {
                render_frame(terminal.out(), &state)?;
                return Ok(RunResult::Win(state.score));
            }
            GameOutcome::Loss => return Ok(RunResult::Loss(state.score)),
        }

        render_frame(terminal.out(), &state)?;
    }
}

fn read_name() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().chars().take(5).collect::<String>())
}

fn print_congrats() {
    println!("+------------------+");
    println!("| Congratulations! |");
    println!("+------------------+");
}

fn print_leaderboard(entries: &[rust_terminal_snake_game::LeaderboardEntry]) {
    println!("Leaderboard:");
    for (idx, e) in entries.iter().enumerate() {
        println!("{}: {},{}", idx + 1, e.name, e.score);
    }
}

fn main() -> io::Result<()> {
    let log_path = Path::new("snake.log");
    let leaderboard_path = Path::new("leaderboard.csv");

    let (width, height) = terminal::size()?;
    if !terminal_size_ok(width, height) {
        println!("{SIZE_ERROR_MESSAGE}");
        let _ = log_size_crash(log_path, width, height, "startup_terminal_too_small");
        std::process::exit(1);
    }

    let result = run_game(width, height)?;

    match result {
        RunResult::RuntimeTooSmall(w, h) => {
            println!("{SIZE_ERROR_MESSAGE}");
            let _ = log_size_crash(log_path, w, h, "runtime_terminal_too_small");
            std::process::exit(1);
        }
        RunResult::Win(score) | RunResult::Loss(score) => {
            print_congrats();
            let entries = read_leaderboard(leaderboard_path).unwrap_or_default();
            let high = previous_high(&entries);
            if score > high {
                println!("New all-time high score! Enter name (max 5 chars):");
                let name = read_name()?;
                let updated = record_new_high(leaderboard_path, entries, &name, score)?;
                print_leaderboard(&updated);
            }
        }
    }

    Ok(())
}
