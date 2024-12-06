use std::{
    collections::HashSet,
    fmt::Display,
    io::{Stdout, Write},
};

use crossterm::{style, QueueableCommand};
use rand::{rngs::ThreadRng, seq::IteratorRandom, thread_rng, Rng};

pub const VERTICAL_LINE: &str = "│";
pub const DOWN_T_CONNECTOR: &str = "┬";
pub const UP_T_CONNECTOR: &str = "┴";
pub const HORIZONTAL_LINE: &str = "─";
pub const TOP_LEFT_CONNECTOR: &str = "┌";
pub const TOP_RIGHT_CONNECTOR: &str = "┐";
pub const BOTTOM_LEFT_CONNECTOR: &str = "└";
pub const BOTTOM_RIGHT_CONNECTOR: &str = "┘";
pub const PLUS_CONNECTOR: &str = "┼";
pub const RIGHT_T_CONNECTOR: &str = "├";
pub const LEFT_T_CONNECTOR: &str = "┤";

pub struct Board {
    cells: [u8; 81],
}

#[derive(Debug)]
pub struct SudokuRow {
    pub cells: [u8; 9],
}

#[allow(clippy::format_collect)]
impl Display for SudokuRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .cells
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let cell = if *cell == 0 {
                    " ".to_string()
                } else {
                    cell.to_string()
                };
                let prefix = if i % 3 == 0 {
                    VERTICAL_LINE.to_string() + " "
                } else {
                    String::new()
                };
                if i < self.cells.len() - 1 {
                    return format!("{prefix}{cell} ");
                }
                format!("{cell} {VERTICAL_LINE}")
            })
            .collect::<String>();
        write!(f, "{string}")
    }
}

#[derive(Debug)]
pub struct SudokuColumn {
    pub cells: [u8; 9],
}

#[derive(Debug)]
pub struct SudokuTile {
    pub cells: [u8; 9],
}

#[derive(Debug)]
pub struct BoardMove {
    position: [usize; 2],
    new_value: u8,
    cascades: Vec<[usize; 2]>,
}

impl BoardMove {
    pub fn position(&self) -> [usize; 2] {
        self.position
    }

    pub fn new_value(&self) -> u8 {
        self.new_value
    }
}

impl Board {
    /// Board constructor
    pub fn new(string_representation: String) -> Board {
        let cells: [u8; 81] = string_representation
            .chars()
            .map(|char| char.to_string().parse::<u8>().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Board { cells }
    }

    /// Retrieves a single row
    pub fn get_row(&self, row: usize) -> SudokuRow {
        SudokuRow {
            cells: self
                .cells
                .iter()
                .enumerate()
                .filter(|(i, _)| i / 9 == row)
                .map(|(_, cell)| *cell)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    /// Retrieves a single column
    pub fn get_column(&self, column: usize) -> SudokuColumn {
        SudokuColumn {
            cells: self
                .cells
                .iter()
                .enumerate()
                .filter(|(i, _)| i % 9 == column)
                .map(|(_, cell)| *cell)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    /// Retrieves a single 3x3 tile
    pub fn get_tile(&self, tile: (usize, usize)) -> SudokuTile {
        SudokuTile {
            cells: self
                .cells
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    let tile_row = i / 27;
                    let tile_column = (i % 9) / 3;
                    tile_row == tile.0 && tile_column == tile.1
                })
                .map(|(_, cell)| *cell)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    /// Formats the board and prints it out to the console
    pub fn draw_board(&self, stdout: &mut Stdout) {
        let temp_row = self.get_row(0).to_string();
        let (_, temp_row_mid) = temp_row.split_at(VERTICAL_LINE.len());
        let (temp_row_mid, _) = temp_row_mid.split_at(temp_row_mid.len() - VERTICAL_LINE.len());

        stdout
            .queue(style::Print(format!(
                "{TOP_LEFT_CONNECTOR}{}{TOP_RIGHT_CONNECTOR}\n",
                temp_row_mid
                    .chars()
                    .map(|char| {
                        if char.to_string() == *VERTICAL_LINE {
                            DOWN_T_CONNECTOR.to_string()
                        } else {
                            HORIZONTAL_LINE.to_string()
                        }
                    })
                    .collect::<String>(),
            )))
            .unwrap();
        for i in 0..9 {
            if i % 3 == 0 && i != 0 && i != 8 {
                stdout
                    .queue(style::Print(format!(
                        "{RIGHT_T_CONNECTOR}{}{LEFT_T_CONNECTOR}\n",
                        temp_row_mid
                            .chars()
                            .map(|char| {
                                if char.to_string() == *VERTICAL_LINE {
                                    PLUS_CONNECTOR.to_string()
                                } else {
                                    HORIZONTAL_LINE.to_string()
                                }
                            })
                            .collect::<String>(),
                    )))
                    .unwrap();
            }
            stdout
                .queue(style::Print(format!("{}\n", self.get_row(i))))
                .unwrap();
        }
        stdout
            .queue(style::Print(format!(
                "{BOTTOM_LEFT_CONNECTOR}{}{BOTTOM_RIGHT_CONNECTOR}\n",
                temp_row_mid
                    .chars()
                    .map(|char| {
                        if char.to_string() == *VERTICAL_LINE {
                            UP_T_CONNECTOR.to_string()
                        } else {
                            HORIZONTAL_LINE.to_string()
                        }
                    })
                    .collect::<String>(),
            )))
            .unwrap();
        stdout.flush().unwrap();
    }

    /// Entropy is defined as all the states that a cell could be in which it is considered valid.
    /// The entropy is calculated through a series of hash set differences, an extremely quick
    /// operation that perfectly fits the rules of sudoku
    pub fn calculate_entropy_at_cell(&self, row: usize, col: usize) -> Option<Vec<u8>> {
        let current_index = row * 9 + col;
        if self.cells[current_index] != 0 {
            return None;
        }

        let possible_options: HashSet<u8> = (0..=9).collect();

        let current_row_set: HashSet<u8> = self.get_row(row).cells.into_iter().collect();
        let current_column_set: HashSet<u8> = self.get_column(col).cells.into_iter().collect();
        let current_tile_set: HashSet<u8> = self
            .get_tile((row / 3, col / 3))
            .cells
            .into_iter()
            .collect();

        let options: Vec<u8> = possible_options
            .difference(&current_row_set)
            .cloned()
            .collect::<HashSet<u8>>()
            .difference(&current_column_set)
            .cloned()
            .collect::<HashSet<u8>>()
            .difference(&current_tile_set)
            .cloned()
            .collect();
        Some(options)
    }

    /// Searches for a cell with the least entropy. The lowest entropy equates to the highest confidence
    pub fn find_least_entropy(&self) -> Option<((usize, usize), Vec<u8>)> {
        let mut min_pos = (10, 10);
        let mut min_entropy = (0..=9).collect::<Vec<u8>>();
        for row in 0..9 {
            for col in 0..9 {
                let current_entropy = self.calculate_entropy_at_cell(row, col);
                if let Some(entropy) = current_entropy {
                    if entropy.len() < min_entropy.len() {
                        min_entropy = entropy;
                        min_pos = (row, col);
                    }
                }
            }
        }
        if min_pos == (10, 10) {
            return None;
        }
        Some((min_pos, min_entropy))
    }

    /// Solves the sudoku puzzle. Iteratively searches for the cell with least entropy, promptly
    /// collapsing it to a single possibility. Producing a wrong result is not impossible
    /// TODO: Implement some form of backtracking to solve cases where wave function colapse gets
    /// stuck
    pub fn solve_board(&mut self, stdout: &mut Stdout) -> Result<(), &str> {
        let mut previous_moves: Vec<BoardMove> = Vec::with_capacity(81);

        let mut least_entropy_result = self.find_least_entropy();
        let mut rng = thread_rng();
        while least_entropy_result.is_some() {
            let ((row, col), min_entropy) = least_entropy_result.as_ref().unwrap();

            if min_entropy.is_empty() {
                self.draw_board(stdout);
                least_entropy_result = self.backtrack(&mut previous_moves, &mut rng);
                continue;
            } else {
                let cell_index = row * 9 + col;
                if min_entropy.len() == 1 {
                    self.cells[cell_index] = min_entropy[0];
                    let last_move = previous_moves.last_mut();
                    if let Some(last) = last_move {
                        last.cascades.push([*row, *col]);
                    }
                } else {
                    let mut valid_options = Vec::with_capacity(min_entropy.len());
                    for value in min_entropy {
                        self.cells[cell_index] = *value;
                        let next_entropy = self.find_least_entropy();

                        if let Some(entropy) = next_entropy {
                            if !entropy.1.is_empty() {
                                valid_options.push((*value, entropy));
                            }
                        } else {
                            return Ok(());
                        }
                    }
                    let choice = valid_options.into_iter().reduce(|acc, (val, entropy_data)| {
                        if entropy_data.1.len() < acc.1.1.len() {
                            return (val, entropy_data);
                        }
                        acc
                    }).unwrap();
                    previous_moves.push(BoardMove {
                        position: [*row, *col],
                        new_value: choice.0,
                        cascades: Vec::new(),
                    });
                };
                least_entropy_result = self.find_least_entropy();
            }
        }
        Ok(())
    }

    /// Validates the resulting board to make sure it follows the sudoku rules
    pub fn validate_board(&self) {
        for i in 0..3 {
            for j in 0..3 {
                let tile = self.get_tile((i, j));
                let tile_set: HashSet<u8> = tile.cells.into_iter().collect();

                let row = self.get_row(i);
                let row_set: HashSet<u8> = row.cells.into_iter().collect();

                let column = self.get_column(i);
                let column_set: HashSet<u8> = column.cells.into_iter().collect();

                if tile.cells.len() != tile_set.len()
                    || row.cells.len() != row_set.len()
                    || column.cells.len() != column_set.len()
                {
                    println!("The solution is invalid!");
                    return;
                }
            }
        }
        println!("The board is valid!");
    }

    /// Backtracking moves when a mistake is made. Re-evaluates the entropy at the previous point,
    /// excluding the value that it took when executing the first time. Filters out values that
    /// lead to invalid board states.
    pub fn backtrack(
        &mut self,
        previous_moves: &mut Vec<BoardMove>,
        rng: &mut ThreadRng,
    ) -> Option<((usize, usize), Vec<u8>)> {
        assert!(!previous_moves.is_empty());
        // println!("Here: {}", previous_moves.len());

        let last_move = previous_moves.pop().unwrap();

        for cascade in &last_move.cascades {
            self.cells[cascade[0] * 9 + cascade[1]] = 0;
        }

        let last_move_position = last_move.position;
        let last_move_position_index = last_move_position[0] * 9 + last_move_position[1];
        self.cells[last_move_position_index] = 0;

        let mut last_cell_entropy: HashSet<u8> = self
            .calculate_entropy_at_cell(last_move_position[0], last_move_position[1])
            .unwrap()
            .into_iter()
            .collect();

        last_cell_entropy.remove(&last_move.new_value);

        let cell_subsitute_opt = last_cell_entropy
            .iter()
            .map(|possible_value| {
                self.cells[last_move_position_index] = *possible_value;
                (possible_value, self.find_least_entropy())
            })
            .filter(|(_, x)| {
                if x.is_none() {
                    return false;
                }

                !x.as_ref().unwrap().1.is_empty()
            })
            .map(|(val, next_data)| (*val, next_data.unwrap()))
            .choose(rng);

        if let Some((substitute_val, next_data)) = cell_subsitute_opt {
            println!("Substitute for {last_move:?}: {substitute_val}, next pos and entropy: {next_data:?}");
            let new_move = BoardMove {
                position: last_move_position,
                new_value: substitute_val,
                cascades: Vec::new(),
            };
            previous_moves.push(new_move);
            return Some(next_data);
        } else {
            println!("Recursing");
            self.backtrack(previous_moves, rng);
        }
        None
    }
}
