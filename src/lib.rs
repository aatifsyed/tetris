use std::ops::{self, Add};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// choice: static array, not hashmap of coords, probably better optimised
// choice: static array, could've used array2D etc
// choice: row-wise, because we'll be searching and clearing rows
// choice: generic CellT, not e.g bitvec because a likely product extension is
//         coloring individual blocks etc
pub struct Grid<const WIDTH: usize, const HEIGHT: usize, CellT> {
    rows: [[CellT; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize, CellT> ops::Add for Grid<WIDTH, HEIGHT, CellT>
where
    CellT: ops::Add<CellT, Output = CellT> + Clone,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        for (this, other) in self
            .rows
            .iter_mut()
            .flatten()
            .zip(rhs.rows.iter().flatten())
        {
            *this = this.clone().add(other.clone())
        }
        self
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Grid<WIDTH, HEIGHT, CellState> {
    const fn empty_row() -> [CellState; WIDTH] {
        // todo: not require Copy for CellState by going through an intermediary,
        //       array_macro, or MaybeUninit. Could keep this as const.
        [CellState::Unoccupied; WIDTH]
    }
    const fn empty() -> Self {
        Self {
            rows: [Self::empty_row(); HEIGHT],
        }
    }
    pub fn drop(&self, mut shape: Self) -> Self {
        // todo: optimise so that we just get the collision point and bump that many rows
        // choice: we could signal to the caller that something else happened
        //         (like the shape was slid all the way up). Depends on future usecases
        while self.collides(&shape) {
            shape = shape.bump_up()
        }
        // bumped up will eventually be blank, so this terminates
        self.clone().add(shape)
    }
    fn collides(&self, other: &Self) -> bool {
        for (self_cell, other_cell) in self.rows.iter().flatten().zip(other.rows.iter().flatten()) {
            if let (CellState::Occupied, CellState::Occupied) = (self_cell, other_cell) {
                return true;
            }
        }
        false
    }
    fn bump_up(&self) -> Self {
        let mut bumped = Self::empty();
        for (src, dst) in self.rows.iter().skip(1).zip(bumped.rows.iter_mut()) {
            *dst = *src
        }
        bumped
    }

    fn bump_right(&self) -> Self {
        let mut bumped = *self;
        for row in bumped.rows.iter_mut() {
            if let Some(rightmost_cell) = row.last_mut() {
                *rightmost_cell = CellState::Unoccupied
            }
            if WIDTH >= 1 {
                row.rotate_right(1)
            }
        }
        bumped
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CellState {
    Occupied,
    #[default]
    Unoccupied,
}

impl ops::Add for CellState {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (CellState::Unoccupied, CellState::Unoccupied) => CellState::Unoccupied,
            _ => CellState::Occupied,
        }
    }
}

#[cfg(test)]
mod tests {
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
        let _: Grid<0, 0, CellState> = grid![].bump_up();
    }

    #[test]
    fn bump_single_row_clears_it() {
        assert_eq!(grid![[#]].bump_up(), grid![[.]]);
    }

    #[test]
    fn bumped_row_falls_off_edge() {
        assert_eq!(grid![[#],[#]].bump_up(), grid![[#],[.]])
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
            .bump_right(),
            grid![[. #]]
        )
    }
}
