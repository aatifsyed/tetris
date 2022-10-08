#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// choice: static array, not hashmap of coords, probably better optimised
// choice: static array, could've used array2D etc
// choice: row-wise, because we'll be searching and clearing rows
// choice: generic CellT, not e.g bitvec because a likely product extension is
//         coloring individual blocks etc
pub struct Grid<const WIDTH: usize, const HEIGHT: usize, CellT> {
    rows: [[CellT; WIDTH]; HEIGHT],
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
    pub fn drop(&self, shape: &Self) -> Self {
        // todo: optimise so that we just get the collision point and bump that many rows
        todo!()
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Occupied,
    Unoccupied,
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
}
