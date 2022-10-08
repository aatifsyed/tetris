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
#[error("would clobber non-default cell at row {row_ix}, column {col_ix} (this is the first clobber, there may be more)")]
pub struct WouldClobber {
    row_ix: usize,
    col_ix: usize,
}

fn is_empty<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

fn is_occupied<T: Default + PartialEq>(t: &T) -> bool {
    !is_empty(t)
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> ops::Add<Self> for &Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default + PartialEq + Clone,
{
    type Output = Result<Grid<WIDTH, HEIGHT, CellT>, WouldClobber>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Grid::<WIDTH, HEIGHT, CellT>::default();
        for (((row_ix, col_ix, lhs), rhs), dest) in self
            .rows
            .iter()
            .enumerate()
            .flat_map(|(row_ix, row)| {
                row.iter()
                    .enumerate()
                    .map(move |(col_ix, cell)| (row_ix, col_ix, cell))
            })
            .zip(rhs.rows.iter().flatten())
            .zip(result.rows.iter_mut().flatten())
        {
            match (is_empty(lhs), is_empty(rhs)) {
                (false, false) => {
                    return Err(WouldClobber { row_ix, col_ix });
                }
                (false, true) => *dest = lhs.clone(),
                (true, false) => *dest = rhs.clone(),
                (true, true) => (),
            }
        }
        Ok(result)
    }
}

mod impl_add {
    use super::{Grid, WouldClobber};
    use std::ops;

    macro_rules! impl_add {
        (lhs = $lhs:ty, rhs = $rhs:ty) => {
            impl_add!(
                lhs = $lhs,
                rhs = $rhs,
                fragment = (|l: $lhs, r: $rhs| (&l).add(&r))
            );
        };
        (lhs = $lhs:ty, rhs = $rhs:ty, fragment = $frag:tt) => {
            impl<const WIDTH: usize, const HEIGHT: usize, CellT> ops::Add<$rhs> for $lhs
            where
                CellT: Default + PartialEq + Clone,
            {
                type Output = Result<Grid<WIDTH, HEIGHT, CellT>, WouldClobber>;

                fn add(self, rhs: $rhs) -> Self::Output {
                    ($frag)(self, rhs)
                }
            }
        };
    }

    impl_add!(lhs = &Grid<WIDTH, HEIGHT, CellT>, rhs = Grid<WIDTH, HEIGHT, CellT>);
    impl_add!(lhs = Grid<WIDTH, HEIGHT, CellT>, rhs = &Grid<WIDTH, HEIGHT, CellT>, fragment = (|l: Grid<WIDTH, HEIGHT, _>, r|(l.add(r))) );
    impl_add!(lhs = Grid<WIDTH, HEIGHT, CellT>, rhs = Grid<WIDTH, HEIGHT, CellT>);
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
    pub fn shift_down(mut self, rhs: usize) -> Self {
        for _ in 0..rhs {
            self = self.bump_down();
        }
        self
    }

    pub fn bump_down(mut self) -> Self {
        if let Some(last_row) = self.rows.last_mut() {
            *last_row = Self::empty_row();
        }
        if self.rows.len() != 0 {
            self.rows.rotate_right(1);
        }
        self
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> Grid<WIDTH, HEIGHT, CellT>
where
    CellT: Default + Clone + PartialEq,
{
    pub fn drop(self, rhs: Self) -> Option<Self> {
        (0..HEIGHT)
            .map(|shift| self.clone() + rhs.clone().shift_down(shift))
            .take_while(Result::is_ok)
            .map(Result::unwrap)
            .last()
    }

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
    fn shift_down_emtpy() {
        let _: Grid<0, 0, CellState> = grid![].shift_down(1);
    }

    #[test]
    fn shift_down_clears_single_row() {
        assert_eq!(grid![[#]].shift_down(1), grid![[.]]);
    }

    #[test]
    fn shift_down_pushes_shifted_row_off_edge() {
        assert_eq!(grid![[#],[#]].shift_down(1), grid![[.],[#]])
    }

    #[test]
    fn clobbering() {
        assert_eq!(
            grid![
                [. . . .], // row 0
                [. # # .], // row 1
                [. . . .], // row 2
            ]
            .add(grid![
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
}
