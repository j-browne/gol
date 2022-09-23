use egui::{Color32, ColorImage};
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    size: [usize; 2],
    data: Vec<bool>,
}

impl Board {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        if let Some(length) = usize::checked_mul(width, height) {
            let data = vec![false; length];
            Self {
                size: [width, height],
                data,
            }
        } else {
            panic!("board size of `({width}, {height})` is too large");
        }
    }

    #[must_use]
    pub fn resize(&self, width: usize, height: usize) -> Self {
        let mut new = Self::new(width, height);
        let [width, height] = [
            usize::min(width, self.size[0]),
            usize::min(height, self.size[1]),
        ];
        for y in 0..height {
            for x in 0..width {
                new[(y, x)] = self[(y, x)];
            }
        }
        new
    }

    #[must_use]
    pub fn size(&self) -> &[usize; 2] {
        &self.size
    }

    #[must_use]
    pub fn data(&self) -> &Vec<bool> {
        &self.data
    }

    #[must_use]
    pub fn get(&self, (y, x): (usize, usize)) -> Option<bool> {
        let [width, height] = self.size;
        (x < width && y < height).then(|| self.data[y * width + x])
    }

    #[must_use]
    pub fn get_mut(&mut self, (y, x): (usize, usize)) -> Option<&mut bool> {
        let [width, height] = self.size;
        (x < width && y < height).then(|| &mut self.data[y * width + x])
    }

    fn live_neighbors(&self, (y, x): (usize, usize)) -> usize {
        #[allow(clippy::range_minus_one)]
        let x_iter = if x == 0 {
            0..=1
        } else if x == self.size[0] - 1 {
            (self.size[0] - 2)..=(self.size[0] - 1)
        } else {
            (x - 1)..=(x + 1)
        };
        #[allow(clippy::range_minus_one)]
        let y_iter = if y == 0 {
            0..=1
        } else if y == self.size[0] - 1 {
            (self.size[1] - 2)..=(self.size[1] - 1)
        } else {
            (y - 1)..=(y + 1)
        };

        y_iter
            .flat_map(|j| {
                x_iter.clone().map(move |i| {
                    if self[(j, i)] && !(i == x && j == y) {
                        1
                    } else {
                        0
                    }
                })
            })
            .sum()
    }

    #[must_use]
    pub fn next(&self) -> Self {
        let mut next = Board::new(self.size[0], self.size[1]);
        for j in 0..self.size[1] {
            for i in 0..self.size[0] {
                let live_neighbors = self.live_neighbors((j, i));
                if live_neighbors == 3 || (live_neighbors == 2 && self[(j, i)]) {
                    next[(j, i)] = true;
                }
            }
        }
        next
    }

    pub fn iter(&self) -> impl Iterator<Item = ((usize, usize), bool)> + '_ {
        (0..self.size[1]).flat_map(move |j| (0..self.size[0]).map(move |i| ((j, i), self[(j, i)])))
    }
}

impl Index<(usize, usize)> for Board {
    type Output = bool;

    fn index(&self, (y, x): (usize, usize)) -> &Self::Output {
        // check that x and y are within the bounds of the board.
        // since we check that `width * height` does not overflow in the constructor,
        // we know that `y * width + x` does not overflow
        let [width, height] = self.size;
        assert!(x < width);
        assert!(y < height);

        &self.data[y * width + x]
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, (y, x): (usize, usize)) -> &mut Self::Output {
        // check that x and y are within the bounds of the board.
        // since we check that `width * height` does not overflow in the constructor,
        // we know that `y * width + x` does not overflow
        let [width, height] = self.size;
        assert!(x < width);
        assert!(y < height);

        &mut self.data[y * width + x]
    }
}

impl From<&Board> for ColorImage {
    fn from(board: &Board) -> ColorImage {
        let Board { size, data } = board;
        #[allow(clippy::clone_on_copy)]
        let size = size.clone();
        let pixels = data
            .iter()
            .map(|b| if *b { Color32::BLACK } else { Color32::WHITE })
            .collect();
        ColorImage { size, pixels }
    }
}

#[cfg(test)]
mod test {
    use super::Board;

    #[test]
    fn new_board() {
        let _board = Board::new(2, 3);
        let _board = Board::new(1000, 1000);
    }

    #[test]
    #[should_panic]
    fn new_board_panic() {
        let _board = Board::new(usize::MAX, 2);
    }

    #[test]
    fn live_neighbors() {
        let mut board = Board::new(3, 3);
        board[(0, 1)] = true;
        board[(1, 2)] = true;
        board[(2, 0)] = true;
        board[(2, 1)] = true;
        board[(2, 2)] = true;

        assert_eq!(board.live_neighbors((0, 0)), 1);
        assert_eq!(board.live_neighbors((0, 1)), 1);
        assert_eq!(board.live_neighbors((0, 2)), 2);
        assert_eq!(board.live_neighbors((1, 0)), 3);
        assert_eq!(board.live_neighbors((1, 1)), 5);
        assert_eq!(board.live_neighbors((1, 2)), 3);
        assert_eq!(board.live_neighbors((2, 0)), 1);
        assert_eq!(board.live_neighbors((2, 1)), 3);
        assert_eq!(board.live_neighbors((2, 2)), 2);
    }
}
