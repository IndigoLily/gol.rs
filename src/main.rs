extern crate termion;

use std::io::{Write, stdout, stdin};
use std::thread;
use std::sync::mpsc::{channel, TryRecvError};
use std::time::Duration;
use std::collections::HashSet as Set;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use rand::{thread_rng, Rng};

fn main() {
    let stdin = stdin();
    let stdout = stdout().into_raw_mode().unwrap();
    let mut screen = termion::screen::AlternateScreen::from(stdout);

    let (tx, rx) = channel();
    let key_handler = thread::spawn(move || {
        for c in stdin.keys() {
            if let Key::Ctrl('c') | Key::Char('q') = c.unwrap() {
                tx.send("quit").unwrap();
                break;
            }
        }
    });

    let mut cells: Set<(isize, isize)> = Set::new();
    let (w, h) = termion::terminal_size().unwrap();
    let mut rng = thread_rng();
    for x in (1..=w).map(|x| x as isize) {
        for y in (1..=h).map(|x| x as isize) {
            if rng.gen_range(0, 100) < 50 {
                cells.insert((x, y));
            }
        }
    }

    loop {
        if let Ok("quit") | Err(TryRecvError::Disconnected) = rx.try_recv() {
            break;
        }

        let (w, h) = termion::terminal_size().unwrap();

        write!(screen, "{}", termion::cursor::Goto(1, 1)).unwrap();

        let mut cell_string = String::with_capacity((w * h) as usize);
        for y in 1..=h {
            for x in 1..=w {
                if let Some(_) = cells.get(&(x as isize, y as isize)) {
                    cell_string.push('█');
                } else {
                    cell_string.push(' ');
                }
            }
        }
        write!(screen, "{}", cell_string).unwrap();
        screen.flush().unwrap();

        let handle = thread::spawn(move || {
            let mut next = Set::new();
            let mut check = Set::new();

            for cell in cells.iter() {
                for x in -1..=1 {
                    for y in -1..=1 {
                        check.insert((cell.0 + x, cell.1 + y));
                    }
                }
            }

            for cell in check {
                let mut nbrs = 0;
                for x in -1..=1 {
                    for y in -1..=1 {
                        if let Some(_) = cells.get(&(cell.0 + x, cell.1 + y)) {
                            nbrs += 1;
                        }
                    }
                }

                if let Some(_) = cells.get(&cell) {
                    if nbrs == 3 || nbrs == 4 {
                        next.insert(cell);
                    }
                } else {
                    if nbrs == 3 {
                        next.insert(cell);
                    }
                }
            }

            next
        });

        thread::sleep(Duration::from_millis(1000 / 30));

        cells = handle.join().unwrap();
    }

    key_handler.join().unwrap();

    screen.flush().unwrap();
}
