use chrono::Utc;
use rand::prelude::IndexedRandom;
use rand::rng;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub const MIN_WIDTH: u16 = 11;
pub const MIN_HEIGHT: u16 = 11;
pub const SIZE_ERROR_MESSAGE: &str = "Window is too small for the game. Please resize it before you can play.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameOutcome {
    Continue,
    Win,
    Loss,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub width: u16,
    pub height: u16,
    pub snake: Vec<Point>, // index 0 is head
    pub direction: Direction,
    pub apple: Point,
    pub score: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub name: String,
    pub score: u32,
}

pub fn random_direction() -> Direction {
    let mut r = rng();
    *[Direction::Up, Direction::Down, Direction::Left, Direction::Right]
        .choose(&mut r)
        .expect("direction pool is non-empty")
}

fn rotate_offset(direction: Direction, x: i32, y: i32) -> (i32, i32) {
    match direction {
        Direction::Right => (x, y),
        Direction::Left => (-x, y),
        Direction::Up => (y, -x),
        Direction::Down => (y, x),
    }
}

pub fn terminal_size_ok(width: u16, height: u16) -> bool {
    width >= MIN_WIDTH && height >= MIN_HEIGHT
}

pub fn board_center(width: u16, height: u16) -> Point {
    Point {
        x: (width as i32) / 2,
        y: (height as i32) / 2,
    }
}

fn step(direction: Direction) -> (i32, i32) {
    match direction {
        Direction::Up => (0, -1),
        Direction::Down => (0, 1),
        Direction::Left => (-1, 0),
        Direction::Right => (1, 0),
    }
}

pub fn build_start_snake(head: Point, direction: Direction) -> Vec<Point> {
    // Keeps head at center while preserving a 9-segment contiguous shape that fits minimum board sizes.
    let base: [(i32, i32); 9] = [
        (0, 0),
        (-1, 0),
        (-2, 0),
        (-3, 0),
        (-3, 1),
        (-2, 1),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    base.iter()
        .map(|(x, y)| {
            let (rx, ry) = rotate_offset(direction, *x, *y);
            Point {
                x: head.x + rx,
                y: head.y + ry,
            }
        })
        .collect()
}

pub fn choose_apple(width: u16, height: u16, snake: &[Point]) -> Option<Point> {
    let mut free = Vec::new();
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let p = Point { x, y };
            if !snake.contains(&p) {
                free.push(p);
            }
        }
    }
    let mut r = rng();
    free.choose(&mut r).copied()
}

pub fn start_state(width: u16, height: u16) -> Option<GameState> {
    let direction = random_direction();
    let head = board_center(width, height);
    let snake = build_start_snake(head, direction);
    if snake.iter().any(|p| p.x < 0 || p.y < 0 || p.x >= width as i32 || p.y >= height as i32) {
        return None;
    }
    let apple = choose_apple(width, height, &snake)?;
    Some(GameState {
        width,
        height,
        snake,
        direction,
        apple,
        score: 0,
    })
}

pub fn is_inside(state: &GameState, p: Point) -> bool {
    p.x >= 0 && p.y >= 0 && p.x < state.width as i32 && p.y < state.height as i32
}

pub fn apply_direction(state: &mut GameState, requested: Option<Direction>) {
    if let Some(next) = requested {
        state.direction = next;
    }
}

pub fn tick(state: &mut GameState) -> GameOutcome {
    let (dx, dy) = step(state.direction);
    let head = state.snake[0];
    let new_head = Point {
        x: head.x + dx,
        y: head.y + dy,
    };

    if !is_inside(state, new_head) {
        return GameOutcome::Loss;
    }

    let eats_apple = new_head == state.apple;

    let mut candidate = Vec::with_capacity(state.snake.len() + 1);
    candidate.push(new_head);
    candidate.extend(state.snake.iter().copied());

    if !eats_apple {
        candidate.pop();
    }

    if candidate[1..].contains(&new_head) {
        return GameOutcome::Loss;
    }

    state.snake = candidate;

    if eats_apple {
        state.score += 1;
        let board_cells = state.width as usize * state.height as usize;
        if state.snake.len() == board_cells {
            return GameOutcome::Win;
        }
        if let Some(p) = choose_apple(state.width, state.height, &state.snake) {
            state.apple = p;
        } else {
            return GameOutcome::Win;
        }
    }

    GameOutcome::Continue
}

pub fn log_size_crash(path: &Path, width: u16, height: u16, reason: &str) -> std::io::Result<()> {
    let ts = Utc::now().to_rfc3339();
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "timestamp_gmt={ts}, width={width}, height={height}, reason={reason}")?;
    Ok(())
}

pub fn read_leaderboard(path: &Path) -> std::io::Result<Vec<LeaderboardEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in content.lines() {
        let mut parts = line.split(',');
        let name = parts.next().unwrap_or("").trim().to_string();
        let score = parts
            .next()
            .unwrap_or("0")
            .trim()
            .parse::<u32>()
            .unwrap_or(0);
        if !name.is_empty() {
            out.push(LeaderboardEntry { name, score });
        }
    }
    Ok(out)
}

pub fn previous_high(entries: &[LeaderboardEntry]) -> u32 {
    entries.first().map(|e| e.score).unwrap_or(0)
}

pub fn record_new_high(path: &Path, mut entries: Vec<LeaderboardEntry>, name: &str, score: u32) -> std::io::Result<Vec<LeaderboardEntry>> {
    let clean_name = name.chars().take(5).collect::<String>();
    entries.insert(
        0,
        LeaderboardEntry {
            name: clean_name,
            score,
        },
    );
    entries.truncate(10);

    let mut out = String::new();
    for e in &entries {
        out.push_str(&format!("{},{}\n", e.name, e.score));
    }
    fs::write(path, out)?;
    Ok(entries)
}

pub fn snake_char(index: usize) -> char {
    if index == 0 {
        return ':';
    }
    const BODY: [char; 8] = ['=', ')', '(', 'O', 'O', 'O', 'o', '.'];
    BODY[(index - 1) % BODY.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_size_constraint_is_enforced() {
        assert!(!terminal_size_ok(10, 11));
        assert!(!terminal_size_ok(11, 10));
        assert!(terminal_size_ok(11, 11));
    }

    #[test]
    fn start_snake_has_required_length() {
        let snake = build_start_snake(Point { x: 5, y: 5 }, Direction::Right);
        assert_eq!(snake.len(), 9);
        assert_eq!(snake[0], Point { x: 5, y: 5 });
        assert!(snake
            .iter()
            .all(|p| p.x >= 0 && p.y >= 0 && p.x < 11 && p.y < 11));
    }

    #[test]
    fn leaderboard_updates_only_with_explicit_write_path() {
        let path = std::env::temp_dir().join("snake_leaderboard_test.csv");
        let _ = std::fs::remove_file(&path);

        let entries = read_leaderboard(&path).expect("read should work");
        assert_eq!(previous_high(&entries), 0);

        let updated = record_new_high(&path, entries, "abcdeZ", 3).expect("write should work");
        assert_eq!(updated[0].name, "abcde");
        assert_eq!(updated[0].score, 3);

        let _ = std::fs::remove_file(&path);
    }
}
