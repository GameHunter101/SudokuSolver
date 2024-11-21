use std::io::stdout;

use crossterm::{
    cursor, style::Print, terminal::{self, Clear}, QueueableCommand
};
use rand::prelude::*;

pub mod board;
use board::Board;
use rand_chacha::ChaCha8Rng;

fn main() {
    // String representation of a sudoku board. The numbers in the string correspond to cells in
    // the board, going left to right, top to bottom.
    /* let initial_board_string =
    "530070000600195000098000060800060003400803001700020006060000280000419005000080079"; */

    // Problem seeds:
    // Board seed: 12499731774094038275, removal seed: 8137985501619016255
    let board_seed =thread_rng().gen();
    let remove_cell_seed = thread_rng().gen();
    let mut initial_board_string = generate_board(board_seed);
    remove_board_cells(&mut initial_board_string, remove_cell_seed);

    let mut board = Board::new(initial_board_string);
    terminal::enable_raw_mode().unwrap();

    let mut stdout = stdout();
    stdout.queue(Clear(terminal::ClearType::All)).unwrap();
    stdout.queue(Print(format!("Board seed: {board_seed}, removal seed: {remove_cell_seed}"))).unwrap();
    stdout.queue(cursor::MoveTo(0, 1)).unwrap();
    board.draw_board(&mut stdout);

    let start_time = std::time::Instant::now();
    if board.solve_board().is_err() {
        println!("The solver was unable to complete the board.");
    }
    let end_time = std::time::Instant::now();

    board.draw_board(&mut stdout);
    terminal::disable_raw_mode().unwrap();

    board.validate_board();
    let duration = end_time - start_time;
    println!("Duration: {}ms", duration.as_millis());
}

/// Base generation derived from https://gamedev.stackexchange.com/a/138228
/// Uses various shifting techniques from https://pi.math.cornell.edu/~mec/Summer2009/Mahmood/Symmetry.html
fn generate_board(seed: u64) -> String {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // Original board generation
    let mut rows = [[0_u8; 9]; 9];
    let mut base_row: [u8; 9] = (0..9).collect::<Vec<_>>().try_into().unwrap();
    base_row.shuffle(&mut rng);
    rows[0] = base_row;
    for i in 1..9 {
        let mut rotated_previous_row = rows[i - 1];
        if i % 3 == 0 {
            rotated_previous_row.rotate_left(1);
            rows[i] = rotated_previous_row;
        } else {
            rotated_previous_row.rotate_left(3);
            rows[i] = rotated_previous_row;
        }
    }

    // Shuffles the indices for the columns within the stack that they exist in in order to
    // preserve sudoku rules
    let shuffled_column_order = (0..3)
        .flat_map(|i| {
            let mut order = [3 * i, 3 * i + 1, 3 * i + 2];
            order.shuffle(&mut rng);
            order
        })
        .collect::<Vec<usize>>();
    rows.iter_mut().for_each(|row| {
        *row = (0..9)
            .map(|i| row[shuffled_column_order[i]])
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    });

    // Shuffles the numbers themselves, e.g. 1->5, 2->3, 9->1. This preserves the sudoku rules
    let mut shuffled_numbers = (1..=9).collect::<Vec<_>>();
    shuffled_numbers.shuffle(&mut rng);

    // Shuffles the indices for the rows within the band that they exist in in order to
    // preserve sudoku rules
    let shuffled_row_order = (0..3).flat_map(|i| {
        let mut order = [3 * i, 3 * i + 1, 3 * i + 2];
        order.shuffle(&mut rng);
        order
    });

    shuffled_row_order
        .flat_map(|i| rows[i].map(|cell| shuffled_numbers[cell as usize].to_string()))
        .collect()
}

fn remove_board_cells(board_string_representation: &mut String, seed: u64) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    
    for _ in 0..rng.gen_range(50..81-17) {
        let rand_index = rng.gen_range(0..81);
        board_string_representation.replace_range(rand_index..rand_index+1, "0");
    }
}
