use std::{io::stdout, time::Instant};

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
    let cells_removed = remove_board_cells(&mut initial_board_string, remove_cell_seed, 20, 30);

    let mut board = Board::new(initial_board_string);
    terminal::enable_raw_mode().unwrap();

    let mut stdout = stdout();
    stdout.queue(Clear(terminal::ClearType::All)).unwrap();
    stdout.queue(Print(format!("Board seed: {board_seed}, removal seed: {remove_cell_seed}"))).unwrap();
    stdout.queue(cursor::MoveTo(0, 1)).unwrap();
    board.draw_board(&mut stdout);

    let start_time = std::time::Instant::now();
    if board.solve_board(&mut stdout).is_err() {
        println!("The solver was unable to complete the board.");
    }
    let end_time = std::time::Instant::now();

    board.draw_board(&mut stdout);
    terminal::disable_raw_mode().unwrap();

    board.validate_board();
    let duration = end_time - start_time;
    println!("Duration: {}ms", duration.as_millis());
    println!("hints: {}", 81-cells_removed);

    /* let mut total_completed = 0.0;
    let mut total_average_time = 0.0;

    for i in 0..1 {
        let (completed_this_round, average_time) = benchmark(i * 200);
        total_completed += completed_this_round;
        total_average_time += average_time;
    }

    println!("Total completed: {total_completed} / 1000, average time: {}ms", total_average_time / 4.0); */
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

/// Takes a completed board and randomly removes cells from it
fn remove_board_cells(board_string_representation: &mut String, seed: u64, minimum_hints: i32, maximum_hints: i32) -> i32 {
    assert!(minimum_hints < maximum_hints, "User specified minimum hints is greater than or equal to maximum hints");
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let test = 81 - rng.gen_range(minimum_hints..maximum_hints);
    for _ in 0..test {
        let mut rand_index = rng.gen_range(0..81);
        while board_string_representation.get(rand_index..rand_index+1) == Some("0") {
            rand_index = rng.gen_range(0..81);
        }
        board_string_representation.replace_range(rand_index..rand_index+1, "0");
    }

    test
}

/* fn benchmark(seed: u64) -> (f32, f32){
    let (completed_count, completed_times): (Vec<_>, Vec<_>) = (0..1000).map(|i| {
        let mut initial_board_string = generate_board(i + seed);
        remove_board_cells(&mut initial_board_string, i, 20, 40);

        let start_time = Instant::now();
        let mut board = Board::new(initial_board_string);
        let solve = board.solve_board(None);
        println!("{i}");
        if solve.is_ok() {
            (1.0, (Instant::now() - start_time).as_millis() as f32)
        } else {
            (0.0,0.0)
        }
    }).unzip();

    let total_completed = completed_count.iter().sum::<f32>();
    let total_time = completed_times.iter().sum::<f32>();
    (total_completed, total_time / total_completed)
} */
