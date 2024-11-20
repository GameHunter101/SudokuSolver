use std::io::stdout;

use board::Board;
use crossterm::{
    terminal::{self, Clear},
    QueueableCommand,
};

pub mod board;

fn main() {
    // String representation of a sudoku board. The numbers in the string correspond to cells in
    // the board, going left to right, top to bottom.
    let initial_board_string =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    let mut board = Board::new(initial_board_string);
    terminal::enable_raw_mode().unwrap();

    let mut stdout = stdout();
    stdout.queue(Clear(terminal::ClearType::All)).unwrap();

    let start_time = std::time::Instant::now();
    board.solve_board();
    let end_time = std::time::Instant::now();

    board.draw_board(&mut stdout);
    terminal::disable_raw_mode().unwrap();

    board.validate_board();
    let duration = end_time - start_time;
    println!("Duration: {}ms", duration.as_millis());
}
