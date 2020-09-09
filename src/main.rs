extern crate termion;

use std::io::{Write, stdout, stdin};
use std::thread;
use std::sync::mpsc::{channel, TryRecvError};
use std::time::Duration;
use std::collections::{HashSet, HashMap};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use rand::{thread_rng, Rng};

type CellGrid = HashSet<(isize, isize)>;

fn main() {
    let stdin = stdin();
    let stdout = stdout().into_raw_mode().unwrap();
    let mut screen = termion::screen::AlternateScreen::from(stdout);

    let (tx, rx) = channel();
    let key_handler = thread::spawn(move || {
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Ctrl('c') | Key::Char('q') => {
                    tx.send("quit").unwrap();
                    break;
                },
                Key::Char('+') => tx.send("speed+").unwrap(),
                Key::Char('-') => tx.send("speed-").unwrap(),
                Key::Char('\n') => tx.send("restart").unwrap(),
                _ => (),
            }
        }
    });

    let mut speed = 20.0;

    'app: loop {
        let mut cells = CellGrid::new();
        let (w, h) = termion::terminal_size().unwrap();
        let mut rng = thread_rng();
        let threshold = rng.gen_range(0, 100);
        for x in (1..=w).map(|x| x as isize) {
            for y in (1..=h).map(|x| x as isize) {
                if rng.gen_range(0, 100) < threshold {
                    cells.insert((x, y));
                }
            }
        }

        'restart: loop {
            match rx.try_recv() {
                Ok("quit") | Err(TryRecvError::Disconnected) => break 'app,
                Ok("restart") => break 'restart,
                Ok("speed+") => speed *= 1.2,
                Ok("speed-") => speed /= 1.2,
                _ => (),
            }

            let (w, h) = termion::terminal_size().unwrap();

            write!(screen, "{}", termion::cursor::Goto(1, 1)).unwrap();

            let mut cell_string = String::with_capacity((w * h) as usize);
            for y in 1..=h {
                for x in 1..=w {
                    if cells.contains(&(x as isize, y as isize)) {
                        cell_string.push('█');
                    } else {
                        cell_string.push(' ');
                    }
                }
            }
            write!(screen, "{}", cell_string).unwrap();
            screen.flush().unwrap();

            let handle = thread::spawn(move || {
                step(&cells)
            });

            thread::sleep(Duration::from_secs_f64(1.0 / speed));

            cells = handle.join().unwrap();
        }
    }

    key_handler.join().unwrap();

    screen.flush().unwrap();
}

fn step(last: &CellGrid) -> CellGrid {
    let mut check = HashMap::new();

    // for every cell in a living cell's neighbourhood, add 1 to the neighbour count
    for cell in last.iter() {
        for x in -1..=1 {
            for y in -1..=1 {
                let key = (cell.0 + x, cell.1 + y);
                let value = check.get(&key).unwrap_or(&0) + 1;
                check.insert(key, value);
            }
        }
    }

    let mut next = CellGrid::new();

    for (cell, &nbrs) in check.iter() {
        if last.contains(cell) && (nbrs == 3 || nbrs == 4) {
            next.insert(*cell);
        } else if nbrs == 3 {
            next.insert(*cell);
        }
    }

    next
}
