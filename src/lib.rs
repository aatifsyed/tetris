//! This library provides a generic fixed-size matrix as a tetris primitive.
//! The board and shapes are represented by a [Grid], which has specialised operations for tetris.
//! The semantics of [Grid] are as follows:
//! - A `CellT` with a [Default] value is considered empty.
//! - A `CellT` with a non-[Default] value is considered occupied.
//!
//! [Grid]s may be scrolled as follows, with tetris semantics:
//! - [std::ops::Shr](struct.Grid.html#impl-Shr<usize>-for-Grid<WIDTH%2C%20HEIGHT%2C%20CellT>) will discard the rightmost column, and create a new empty leftmost column.
//! - [Grid::try_bump_down] will discard the lowermost column, and create a new empty top column.
//!   It will fail if the lowermost column contains any blocks.
//!
//! The following operations may be used to play the game:
//! - [std::ops::BitAnd](struct.Grid.html#impl-BitAnd%3CGrid%3CWIDTH%2C%20HEIGHT%2C%20CellT%3E%3E-for-Grid%3CWIDTH%2C%20HEIGHT%2C%20CellT%3E) will try and place one grid atop another, failing if any cell is occupied in both grids.
//! - [Grid::drop] will place a grid on another, applying gravity until the other hits a block or the floor.
//! - [Grid::with_solid_rows_cleared] will clear filled rows, and scroll the rest of the board down to fill.

use array_macro::array;
use std::{
    fmt, mem,
    ops::{self, BitAnd},
};

/// A generic matrix of cells.
/// See [module documentation](index.html) for more.
// choice: static array, not hashmap of coords, probably better optimised
// choice: static array, could've used array2D etc
// choice: row-wise, because we'll be searching and clearing rows
// choice: generic CellT, not e.g bitvec because a likely product extension is
//         coloring individual blocks etc
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid<const WIDTH: usize, const HEIGHT: usize, CellT = CellState> {
    pub rows: [[CellT; WIDTH]; HEIGHT],
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

/// The first colliding indices when trying to combine [Grid]s with [std::ops::BitAnd](struct.Grid.html#impl-BitAnd<Grid<WIDTH%2C%20HEIGHT%2C%20CellT>>-for-Grid<WIDTH%2C%20HEIGHT%2C%20CellT>).
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone, Copy)]
#[error("would clobber non-default cell at row {row_ix}, column {col_ix} (this is the first clobber, there may be more)")]
pub struct WouldClobber {
    pub row_ix: usize,
    pub col_ix: usize,
}

pub fn is_empty<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

pub fn is_occupied<T: Default + PartialEq>(t: &T) -> bool {
    !is_empty(t)
}

/// Mask `self` with `rhs`, failing if the masks collide.
/// The returned [Err] shows the colliding indices of the first colliding blocks.
/// ```
/// use tetris::{grid, WouldClobber};
/// assert_eq!(grid![
///     [. . . .],
///     [. . . .],
///     [. . # #],
///     [. . # #],
/// ] & grid![
///     [# # . .],
///     [# # . .],
///     [. . . .],
///     [. . . .],
/// ],
/// Ok(grid![
///     [# # . .],
///     [# # . .],
///     [. . # #],
///     [. . # #],
/// ]));
/// assert_eq!(grid![
///       // ↓
///     [# # # #],
///     [# # # #], // ← !
///     [. . . .],
///     [. . . .],
/// ] & grid![
///       // ↓
///     [. . # #], // ← !
///     [. . # #],
///     [. . # #],
///     [. . # #],
/// ],
/// Err(WouldClobber {
///     row_ix: 0,
///     col_ix: 2,
/// }));
/// ```
// consume so we don't need CellT: Clone, and big stack use is caller's responsibility
impl<const WIDTH: usize, const HEIGHT: usize, CellT> ops::BitAnd<Self>
    for Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default + PartialEq,
{
    type Output = Result<Grid<WIDTH, HEIGHT, CellT>, WouldClobber>;

    fn bitand(mut self, mut rhs: Self) -> Self::Output {
        for ((row_ix, col_ix, lhs), rhs) in self
            .rows
            .iter_mut()
            .enumerate()
            .flat_map(|(row_ix, row)| {
                row.iter_mut()
                    .enumerate()
                    .map(move |(col_ix, cell)| (row_ix, col_ix, cell))
            })
            .zip(rhs.rows.iter_mut().flatten())
        {
            match (is_empty(lhs), is_empty(rhs)) {
                (false, false) => {
                    return Err(WouldClobber { row_ix, col_ix });
                }
                (false, true) => (), // lhs already contains the value
                (true, false) => mem::swap(lhs, rhs),
                (true, true) => (),
            }
        }
        Ok(self)
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> ops::Shr<usize> for Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default,
{
    type Output = Self;

    /// Push rightmost column off the edge, filling a new leftmost column with the default.
    /// ```
    /// use tetris::grid;
    /// let grid = grid![
    ///     [. . . . ],
    ///     [. # # . ],
    ///     [. # # . ],
    ///     [. . . . ],
    /// ];
    /// assert_eq!(
    /// grid.clone() >> 1,
    /// grid![
    ///     [. . . . ],
    ///     [. . # # ], // →
    ///     [. . # # ], // →
    ///     [. . . . ],
    /// ]);
    /// assert_eq!(
    /// grid.clone() >> 2,
    /// grid![
    ///     [. . . . ],
    ///     [. . . # ], // → →
    ///     [. . . # ], // → →
    ///     [. . . . ],
    /// ]);
    /// ```
    fn shr(mut self, rhs: usize) -> Self::Output {
        for _ in 0..rhs {
            for row in self.rows.iter_mut() {
                if let Some(rightmost_cell) = row.last_mut() {
                    *rightmost_cell = Default::default()
                }
                if WIDTH > 1 {
                    row.rotate_right(1)
                }
            }
        }
        self
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default + Clone + PartialEq,
{
    /// Try and move this grid down, fail if the last row is non-empty
    /// ```
    /// use tetris::grid;
    /// assert_eq!(grid![
    ///     [. . . . ],
    ///     [. # # . ],
    ///     [. # # . ],
    ///     [. . . . ],
    /// ].try_bump_down(),
    /// Some(grid![
    ///     [. . . . ], // ↳ empty row wraps around
    ///     [. . . . ], // ↓
    ///     [. # # . ], // ↓
    ///     [. # # . ], // ↓
    /// ]));
    /// assert_eq!(grid![
    ///     [. . . . ],
    ///     [. . . . ],
    ///     [. # # . ],
    ///     [. # # . ],
    /// ].try_bump_down(),
    ///     None, // final row would fall off
    /// );
    ///
    /// ```
    pub fn try_bump_down(mut self) -> Option<Self> {
        match self.rows.last() {
            Some(last_row) if last_row.iter().all(is_empty) => {
                self.rows.rotate_right(1);
                Some(self)
            }
            Some(_) => None,
            None => Some(self),
        }
    }

    /// Try and bump by `by` rows, returning None if any of those bumps would fail.
    pub fn try_shift_down(mut self, by: usize) -> Option<Self> {
        for _ in 0..by {
            self = self.try_bump_down()?
        }
        Some(self)
    }

    /// Place `rhs` on the grid, and move it down until:
    /// - it hits another block
    /// - it hits the bottom of the grid
    ///
    /// Returns [None] if `rhs` can't be placed on the grid.
    /// ```
    /// use tetris::grid;
    /// assert_eq!(grid![
    ///     [. . . . ],
    ///     [. . . . ],
    ///     [. . . . ],
    ///     [. . . . ],
    ///     [. # # . ],
    /// ].drop(grid![
    ///     [# # . . ],
    ///     [# # . . ],
    ///     [. . . . ],
    ///     [. . . . ],
    ///     [. . . . ],
    /// ]),
    /// Some(grid![
    ///     [. . . . ], // ↓
    ///     [. . . . ], // ↓
    ///     [# # . . ], // ↓
    ///     [# # . . ], // ↳ we've hit blocks underneath
    ///     [. # # . ],
    /// ]));
    /// ```
    pub fn drop(self, rhs: Self) -> Option<Self> {
        let mut furthest = self.clone().bitand(rhs.clone()).ok()?;

        // bound by HEIGHT to catch an empty rhs
        for shift in 0..HEIGHT {
            match rhs.clone().try_shift_down(shift) {
                Some(shifted) => match self.clone().bitand(shifted) {
                    Ok(new_furthest) => furthest = new_furthest,
                    Err(_) => break,
                },
                None => break, // rhs has hit the bottom of the grid
            }
        }
        Some(furthest)
    }

    /// Clear full rows by shifting taller rows down
    /// ```
    /// use tetris::grid;
    /// assert_eq!(grid![
    ///     [. . . . . . . . . .],
    ///     [. . # # . . . . . .],
    ///     [. . # # . . . . . .],
    ///     [# # # # # # # # # #], // ← will be removed
    ///     [# # . . # # # # # #],
    /// ].with_solid_rows_cleared(),
    /// grid![
    ///     [. . . . . . . . . .], // ↳ fresh new row
    ///     [. . . . . . . . . .],
    ///     [. . # # . . . . . .], // ↓
    ///     [. . # # . . . . . .], // ↓
    ///     [# # . . # # # # # #],
    /// ]
    /// )
    /// ```
    pub fn with_solid_rows_cleared(mut self) -> Self {
        // outer loop is necessary because inner won't check the shifted row
        // could also do a mark and sweep
        while self.rows.iter().any(|row| row.iter().all(is_occupied)) {
            for row_ix in (0..HEIGHT).rev() {
                if self.rows[row_ix].iter().all(is_occupied) {
                    self.rows[row_ix] = Self::empty_row();
                    self.rows[..=row_ix].rotate_right(1);
                }
            }
        }
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CellState {
    // The reason we do a song and dance with `Default` above is because
    // putting information in `Occupied` is now trivial - a likely extension for
    // the business (e.g adding colours)
    Occupied,
    #[default]
    Unoccupied,
}

impl fmt::Debug for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Occupied => write!(f, "#"),
            Self::Unoccupied => write!(f, "."),
        }
    }
}

/// Construct a [Grid<_, _, CellState>], where `.` is [CellState::Unoccupied] and `#` is [CellState::Occupied]
/// ```
/// use tetris::grid;
/// grid![
///     [. . . . ],
///     [. # # . ],
///     [. # # . ],
///     [. . . . ],
/// ];
/// ```
#[macro_export]
macro_rules! grid {
    ($([$($cell:tt)* $(,)?]),* $(,)?) => {
        $crate::Grid {
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
        $crate::CellState::Occupied
    };
    (@cell .) => {
        $crate::CellState::Unoccupied
    };
}

#[cfg(test)]
mod tests {
    use std::ops::{BitAnd, Shr};

    use super::*;

    #[test]
    fn shr_empty() {
        let _: Grid<0, 0, CellState> = grid![] >> 1;
        let _: Grid<0, 1, CellState> = grid![[]] >> 1;
        let _: Grid<1, 0, CellState> = grid![] >> 1;
    }

    #[test]
    fn bitand() {
        assert_eq!(grid![[#]].bitand(grid![[.]]), Ok(grid![[#]]));
    }

    #[test]
    fn cant_shift_off_edge() {
        assert_eq!(grid![[#]].try_shift_down(1), None)
    }

    #[test]
    fn clobbering() {
        assert_eq!(
            grid![
                [. . . .], // row 0
                [. # # .], // row 1
                [. . . .], // row 2
            ]
            .bitand(grid![
                [. . . .],
                [. . # .],
                [. . . .],
            ]),
            Err(WouldClobber {
                row_ix: 1,
                col_ix: 2
            })
        )
    }

    #[test]
    fn drop_single() {
        assert_eq!(grid!([.]).drop(grid!([#])), Some(grid!([#])))
    }

    #[test]
    fn drop_through_air() {
        assert_eq!(
            grid!([.], [.], [.]).drop(grid!([#], [.], [.])),
            Some(grid!([.], [.], [#]))
        )
    }

    #[test]
    fn drop_with_no_solution() {
        assert_eq!(grid!([#]).drop(grid!([#])), None)
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
                [#],
                [.],
                [.],
            ]),
            Some(grid![
                [.],
                [#],
                [#],
            ])
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
                [# #],
                [. .],
                [. .],
            ]),
            Some(grid![
                [. .],
                [# #],
                [. #],
            ])
        )
    }

    #[test]
    fn drop_does_not_warp_past_overhang() {
        assert_eq!(
            grid![
                [. .],
                [# #],
                [. .],
            ]
            .drop(grid![
                [# #],
                [. .],
                [. .],
            ]),
            Some(grid![
                [# #],
                [# #],
                [. .],
            ])
        )
    }

    #[test]
    fn shift_right_empty() {
        let _: Grid<0, 0, CellState> = grid![].shr(1);
    }

    #[test]
    fn shift_right_clears_single_column() {
        assert_eq!(grid![[#]].shr(1), grid![[.]]);
    }

    #[test]
    fn shift_right_pushes_shifted_column_off_edge() {
        assert_eq!(grid![[# #]].shr(1), grid![[. #]])
    }

    #[test]
    fn solid_row_is_cleared() {
        assert_eq!(
            grid![
                [# . .],
                [# # #],
                [. # .],
            ]
            .with_solid_rows_cleared(),
            grid![
                [. . .],
                [# . .],
                [. # .],
            ]
        )
    }

    #[test]
    fn multiple_solid_rows_cleared() {
        assert_eq!(
            grid![
                [# . .],
                [# # #],
                [# # #],
                [. # .],
                [# # #],
                [. . #],
            ]
            .with_solid_rows_cleared(),
            grid![
                [. . .],
                [. . .],
                [. . .],
                [# . .],
                [. # .],
                [. . #],
            ]
        )
    }

    #[test]
    fn final_addition_example1() {
        assert_eq!(
            grid![
                [. . . . . . . . . .],
                [. . . . . . . . . .],
                [# # # # # # # # . .]
            ]
            .bitand(grid![
                [. . . . . . . . # #],
                [. . . . . . . . # #],
                [. . . . . . . . . .],
            ])
            .unwrap(),
            grid![
                [. . . . . . . . # #],
                [. . . . . . . . # #],
                [# # # # # # # # . .]
            ],
        )
    }

    #[test]
    fn final_drop_example1() {
        assert_eq!(
            grid![
                [. . . . . . . . . .],
                [. . . . . . . . . .],
                [# # # # # # # # . .]
            ]
            .drop(grid![
                [. . . . . . . . # #],
                [. . . . . . . . # #],
                [. . . . . . . . . .],
            ])
            .unwrap(),
            grid![
                [. . . . . . . . . .],
                [. . . . . . . . # #],
                [# # # # # # # # # #]
            ],
        )
    }
}
