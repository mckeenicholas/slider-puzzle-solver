use rand::{seq::SliceRandom, thread_rng};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Up,
    Left,
    Down,
    Right,
}

impl Move {
    pub fn as_offset(&self) -> (isize, isize) {
        match self {
            Move::Up => (1, 0),
            Move::Left => (0, 1),
            Move::Down => (-1, 0),
            Move::Right => (0, -1),
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Move::Up => Move::Down,
            Move::Down => Move::Up,
            Move::Left => Move::Right,
            Move::Right => Move::Left,
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            Move::Up => "Up",
            Move::Left => "Left",
            Move::Down => "Down",
            Move::Right => "Right",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct Puzzle {
    size: usize,
    board: Vec<Vec<u32>>,
    x_pos: usize,
    y_pos: usize,
}

impl Puzzle {
    pub fn new(size: usize) -> Self {
        let mut board = Vec::new();
        let mut value = 1;

        for i in 0..size {
            let mut row = Vec::new();
            for j in 0..size {
                if i == size - 1 && j == size - 1 {
                    row.push(0); // The empty space is represented by 0
                } else {
                    row.push(value);
                    value += 1;
                }
            }
            board.push(row);
        }

        Self {
            size,
            board,
            x_pos: size - 1,
            y_pos: size - 1,
        }
    }

    pub fn apply_move(&mut self, movement: Move) -> bool {
        let (dx, dy) = movement.as_offset();

        let new_x = self.x_pos as isize + dx;
        let new_y = self.y_pos as isize + dy;

        if new_x >= 0 && new_x < self.size as isize && new_y >= 0 && new_y < self.size as isize {
            let new_x = new_x as usize;
            let new_y = new_y as usize;

            self.board[self.x_pos][self.y_pos] = self.board[new_x][new_y];
            self.board[new_x][new_y] = 0;

            self.x_pos = new_x;
            self.y_pos = new_y;
            true
        } else {
            false
        }
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();

        // Flatten the board
        let mut flattened: Vec<u32> = self
            .board
            .iter()
            .flat_map(|row| row.iter())
            .cloned()
            .collect();

        loop {
            flattened.shuffle(&mut rng);

            // Reconstruct the board
            for i in 0..self.size {
                for j in 0..self.size {
                    self.board[i][j] = flattened[i * self.size + j];
                    if self.board[i][j] == 0 {
                        self.x_pos = i;
                        self.y_pos = j;
                    }
                }
            }

            if Self::is_solvable(&flattened, self.size, self.x_pos) {
                break;
            }
        }
    }

    pub fn is_current_state_solvable(&self) -> bool {
        // Convert the 2D board to a 1D array for easier processing
        let flat_board: Vec<u32> = self
            .board
            .iter()
            .flat_map(|row| row.iter().cloned())
            .collect();

        Self::is_solvable(&flat_board, self.size, self.x_pos)
    }

    fn is_solvable(flattened: &[u32], size: usize, empty_row: usize) -> bool {
        let inversions = Self::count_inversions(flattened);

        if size % 2 == 1 {
            // Odd-sized puzzle: solvable if inversions count is even
            inversions % 2 == 0
        } else {
            // Even-sized puzzle: solvable if (inversions + empty row index) is odd
            (inversions + empty_row) % 2 == 1
        }
    }

    fn count_inversions(flattened: &[u32]) -> usize {
        flattened
            .iter()
            .enumerate()
            .filter(|&(_, &val)| val != 0)
            .map(|(i, &val)| {
                flattened[i + 1..]
                    .iter()
                    .filter(|&&next| next != 0 && next < val)
                    .count()
            })
            .sum()
    }

    pub fn is_solved(&self) -> bool {
        let mut expected = 1;

        for i in 0..self.size {
            for j in 0..self.size {
                if i == self.size - 1 && j == self.size - 1 {
                    if self.board[i][j] != 0 {
                        return false;
                    }
                } else {
                    if self.board[i][j] != expected {
                        return false;
                    }
                    expected += 1;
                }
            }
        }

        true
    }

    pub fn solve(&self) -> Result<Vec<Move>, &'static str> {
        let mut path = Vec::new();
        let mut bound = self.heuristic();
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 1000000;
    
        if !self.is_current_state_solvable() {
            return Err("Puzzle is not solvable");
        }
    
        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err("Maximum iterations exceeded");
            }
    
            let result = self.ida_star_search(0, bound, &mut path, None);
            match result {
                Ok(solution) => return Ok(solution),
                Err(new_bound) => {
                    if new_bound == usize::MAX {
                        return Err("No solution found");
                    }
                    if new_bound <= bound {
                        return Err("No progress possible");
                    }
                    bound = new_bound;
                }
            }
        }
    }

    fn ida_star_search(
        &self,
        g: usize,
        bound: usize,
        path: &mut Vec<Move>,
        last_move: Option<Move>,
    ) -> Result<Vec<Move>, usize> {
        let f = g + self.heuristic();
        if f > bound {
            return Err(f);
        }
        if self.is_solved() {
            return Ok(path.clone());
        }

        let mut min = usize::MAX;
        let moves = [Move::Up, Move::Down, Move::Left, Move::Right];

        for &dir in &moves {
            if let Some(last) = last_move {
                if dir == last.opposite() {
                    continue;
                }
            }

            if let Some(new_puzzle) = self.try_move(dir) {
                // Add cycle detection
                let is_cycle = path.windows(2).any(|window| {
                    if let [prev, next] = window {
                        *prev == dir.opposite() && *next == dir
                    } else {
                        false
                    }
                });

                if is_cycle {
                    continue;
                }

                path.push(dir);
                
                // Add depth limit to prevent stack overflow
                if path.len() > self.size * self.size * 4 {
                    path.pop();
                    continue;
                }

                match new_puzzle.ida_star_search(g + 1, bound, path, Some(dir)) {
                    Ok(solution) => return Ok(solution),
                    Err(t) => {
                        if t < min {
                            min = t;
                        }
                    }
                }
                path.pop();
            }
        }

        Err(min)
    }

    fn try_move(&self, dir: Move) -> Option<Self> {
        let mut new_puzzle = self.clone(); // Clone the current puzzle
        if new_puzzle.apply_move(dir) {
            Some(new_puzzle)
        } else {
            None
        }
    }

    fn heuristic(&self) -> usize {
        self.manhattan_distance() + 2 * self.linear_conflicts()
    }

    fn manhattan_distance(&self) -> usize {
        let mut distance = 0;
        for i in 0..self.size {
            for j in 0..self.size {
                let value = self.board[i][j];
                if value != 0 {
                    let target_x = (value - 1) / self.size as u32;
                    let target_y = (value - 1) % self.size as u32;
                    distance += (i as isize - target_x as isize).unsigned_abs();
                    distance += (j as isize - target_y as isize).unsigned_abs();
                }
            }
        }
        distance
    }

    fn linear_conflicts(&self) -> usize {
        let mut conflicts = 0;

        // Row conflicts
        for row in 0..self.size {
            let mut max_seen = 0;
            for col in 0..self.size {
                let value = self.board[row][col];
                if value != 0 && (value - 1) / self.size as u32 == row as u32 {
                    if value > max_seen {
                        max_seen = value;
                    } else {
                        conflicts += 1;
                    }
                }
            }
        }

        // Column conflicts
        for col in 0..self.size {
            let mut max_seen = 0;
            for row in 0..self.size {
                let value = self.board[row][col];
                if value != 0 && (value - 1) % self.size as u32 == col as u32 {
                    if value > max_seen {
                        max_seen = value;
                    } else {
                        conflicts += 1;
                    }
                }
            }
        }

        conflicts
    }
}

impl fmt::Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.board {
            for &val in row {
                write!(f, "{:2} ", val)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
