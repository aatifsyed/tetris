use array_macro::array;
use std::ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// choice: static array, not hashmap of coords, probably better optimised
// choice: static array, could've used array2D etc
// choice: row-wise, because we'll be searching and clearing rows
// choice: generic CellT, not e.g bitvec because a likely product extension is
//         coloring individual blocks etc
pub struct Grid<const WIDTH: usize, const HEIGHT: usize, CellT> {
    rows: [[CellT; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default + Clone,
{
    fn empty_row() -> [CellT; WIDTH] {
        array![CellT::default(); WIDTH]
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> Default for Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default + Clone,
{
    fn default() -> Self {
        Self {
            rows: array![Self::empty_row(); HEIGHT],
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone, Copy)]
#[error("would clobber non-default cell at row {row_n}, column {col_n} (this is the first clobber, there may be more)")]
pub struct WouldClobber {
    row_n: usize,
    col_n: usize,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> ops::Add for &Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default + PartialEq + Clone,
{
    type Output = Result<Grid<WIDTH, HEIGHT, CellT>, WouldClobber>;

    fn add(self, rhs: Self) -> Self::Output {
        // todo: this function is doing too much
        let mut result = Grid::<WIDTH, HEIGHT, CellT>::default();
        let mut clobbered = None;
        for (((row_n, col_n, lhs), rhs), dest) in self
            .rows
            .iter()
            .enumerate()
            .flat_map(|(row_n, row)| {
                row.iter()
                    .enumerate()
                    .map(move |(col_n, cell)| (row_n, col_n, cell))
            })
            .zip(rhs.rows.iter().flatten())
            .zip(result.rows.iter_mut().flatten())
        {
            match (is_default(lhs), is_default(rhs)) {
                (false, false) => {
                    clobbered.replace(WouldClobber { row_n, col_n });
                }
                (false, true) => *dest = lhs.clone(),
                (true, false) => *dest = rhs.clone(),
                (true, true) => (),
            }
        }
        match clobbered {
            Some(err) => Err(err),
            None => Ok(result),
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> ops::Shr<usize> for Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default,
{
    type Output = Self;

    fn shr(mut self, rhs: usize) -> Self::Output {
        for row in self.rows.iter_mut() {
            if let Some(rightmost_cell) = row.last_mut() {
                *rightmost_cell = Default::default()
            }
            if WIDTH > 1 {
                row.rotate_right(rhs)
            }
        }
        self
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default + Clone,
{
    pub fn shift_up(mut self, rhs: usize) -> Self {
        if HEIGHT >= 1 {
            self.rows[0] = Self::empty_row();
            self.rows.rotate_left(rhs);
        }
        self
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default + Clone + PartialEq,
{
    pub fn drop(self, rhs: Self) -> Self {
        use ops::Add;
        match self.add(&rhs) {
            Ok(masked) => masked,
            // HEIGHT = 3; WIDTH = 4
            // 0  . . . .   . . . .
            // 1  . # # . + . . . . => WouldClober
            // 2  # # . .   # . . .
            Err(WouldClobber { row_n, .. }) => {
                self.shift_up(HEIGHT - row_n).add(&rhs).expect("fucked it")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CellState {
    // The reason we do a song and dance with `Default` above is because
    // putting information in `Occupied` is now trivial - a likely extension for
    // the business (e.g adding colours)
    Occupied,
    #[default]
    Unoccupied,
}

#[cfg(test)]
mod tests {
    use std::ops::{Add, Shr};

    use super::*;

    macro_rules! grid {
        ($([$($cell:tt)* $(,)?]),* $(,)?) => {
            Grid {
                rows:
                    [ // begin grid
                        $([ // begin row
                            $(
                                grid!(@cell $cell),
                            )*
                        ]),* // end row
                    ] // end grid
                }
        };
        (@cell #) => {
            CellState::Occupied
        };
        (@cell .) => {
            CellState::Unoccupied
        };
    }

    #[test]
    fn bump_empty() {
        let _: Grid<0, 0, CellState> = grid![].shift_up(1);
    }

    #[test]
    fn bump_single_row_clears_it() {
        assert_eq!(grid![[#]].shift_up(1), grid![[.]]);
    }

    #[test]
    fn bumped_row_falls_off_edge() {
        assert_eq!(grid![[#],[#]].shift_up(1), grid![[#],[.]])
    }

    #[test]
    fn clobbering() {
        assert_eq!(
            grid![
                [. . . .], // row 0
                [. . . .], // row 1
                [. . . .], // row 2
            ]
            .add(&grid![
                [. . . .],
                [. . . .],
                [. . . .],
            ]),
            Ok(Grid::default())
        )
    }

    #[test]
    fn drop_single() {
        assert_eq!(grid!([.]).drop(grid!([#])), grid!([#]))
    }

    #[test]
    fn drop_overlapping_terminates() {
        assert_eq!(grid!([#]).drop(grid!([#])), grid!([#]))
    }

    #[test]
    fn drop_onto_another_block() {
        assert_eq!(
            grid![
                [.],
                [.],
                [#],
            ]
            .drop(grid![
                [.],
                [.],
                [#],
            ]),
            grid![
                [.],
                [#],
                [#],
            ]
        )
    }

    #[test]
    fn drop_onto_another_block_with_overhang() {
        assert_eq!(
            grid![
                [. .],
                [. .],
                [. #],
            ]
            .drop(grid![
                [. .],
                [. .],
                [# #],
            ]),
            grid![
                [. .],
                [# #],
                [. #],
            ]
        )
    }

    #[test]
    fn bump_right() {
        assert_eq!(
            grid![
                [# .]
            ]
            .shr(1),
            grid![[. #]]
        )
    }
}
