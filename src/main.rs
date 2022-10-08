use std::{ops::Shr, str::FromStr};

use anyhow::Context;
use derive_more::From;
use generic_new::GenericNew;
use recap::Recap;
use serde::Deserialize;
use strum::EnumString;
use tetris::{is_empty, CellState, Grid};

fn print<const WIDTH: usize, const HEIGHT: usize>(grid: &Grid<WIDTH, HEIGHT>, comment: &str) {
    println!("{comment}");
    for row in grid.rows {
        println!("\t{row:?}");
    }
    println!("{comment}");
}

fn process_blocks<const WIDTH: usize, const HEIGHT: usize>(
    blocks: impl IntoIterator<Item = impl Into<InputBlock>>,
) -> anyhow::Result<Grid<WIDTH, HEIGHT>> {
    let mut grid = Grid::default();
    for block in blocks {
        let InputBlock {
            shape,
            starting_column,
        } = block.into();
        let new_shape = grid_for(shape);
        print(&new_shape, "new shape");
        let new_shape = new_shape.shr(starting_column);
        print(&new_shape, "offset");
        grid = grid
            .drop(new_shape)
            .context("grid's top layer are already occupied")?;
        print(&grid, "dropped");
        grid = grid.with_solid_rows_cleared();
        print(&grid, "solid rows cleared")
    }
    Ok(grid)
}

fn main() -> anyhow::Result<()> {
    todo!()
}

#[derive(Debug, EnumString, Deserialize, PartialEq, Eq, Clone, Copy)]
enum BlockShape {
    Q,
    Z,
    S,
    T,
    I,
    L,
    J,
}

#[derive(Debug, Deserialize, Recap, PartialEq, Eq, Clone, Copy, GenericNew, From)]
#[recap(regex = r#"(?P<shape>\w)(?P<starting_column>\d+)"#)]
struct InputBlock {
    shape: BlockShape,
    starting_column: usize,
}

fn parse_line(s: &str) -> anyhow::Result<Vec<InputBlock>> {
    Ok(s.split(",")
        .map(InputBlock::from_str)
        .collect::<Result<Vec<_>, _>>()?)
}

fn fill<const WIDTH: usize, const HEIGHT: usize>(
    grid: &mut Grid<WIDTH, HEIGHT>,
    coords: impl IntoIterator<Item = (usize, usize)>,
) {
    for (row_ix, col_ix) in coords {
        grid.rows[row_ix][col_ix] = CellState::Occupied;
    }
}
fn grid_for<const WIDTH: usize, const HEIGHT: usize>(shape: BlockShape) -> Grid<WIDTH, HEIGHT> {
    // once const rust is more mature, we can static assert that WIDTH fits I and HEIGHT fits J/L
    // (the code will currently panic)
    let mut grid = Grid::default();
    match shape {
        BlockShape::Q => fill(&mut grid, [(0, 0), (0, 1), (1, 0), (1, 1)]),
        BlockShape::Z => fill(&mut grid, [(0, 0), (0, 1), (1, 1), (1, 2)]),
        BlockShape::S => fill(&mut grid, [(0, 1), (0, 2), (1, 0), (1, 1)]),
        BlockShape::T => fill(&mut grid, [(0, 0), (0, 1), (0, 2), (1, 1)]),
        BlockShape::I => fill(&mut grid, [(0, 0), (0, 1), (0, 2), (0, 3)]),
        BlockShape::L => fill(&mut grid, [(0, 0), (1, 0), (2, 0), (2, 1)]),
        BlockShape::J => fill(&mut grid, [(0, 1), (1, 1), (2, 1), (2, 0)]),
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use tetris::grid;

    #[test]
    fn parse1() -> anyhow::Result<()> {
        use BlockShape::{I, Q};
        assert_eq!(
            parse_line("I0,I4,Q8")?,
            vec![
                InputBlock::new(I, 0),
                InputBlock::new(I, 4),
                InputBlock::new(Q, 8)
            ]
        );
        Ok(())
    }
    #[test]
    fn shapes() -> anyhow::Result<()> {
        use BlockShape::{I, J, L, Q, S, T, Z};
        assert_eq!(
            grid_for(I),
            grid![
                [# # # # .],
                [. . . . .]
            ]
        );
        assert_eq!(
            grid_for(J),
            grid![
                [. # .],
                [. # .],
                [# # .],
                [. . .],
            ]
        );
        assert_eq!(
            grid_for(L),
            grid![
                [# . .],
                [# . .],
                [# # .],
                [. . .],
            ]
        );
        assert_eq!(
            grid_for(Q),
            grid![
                [# # .],
                [# # .],
                [. . .],
            ]
        );
        assert_eq!(
            grid_for(S),
            grid![
                [. # # .],
                [# # . .],
                [. . . .],
            ]
        );
        assert_eq!(
            grid_for(T),
            grid![
                [# # # .],
                [. # . .],
                [. . . .],
            ]
        );
        assert_eq!(
            grid_for(Z),
            grid![
                [# # . .],
                [. # # .],
                [. . . .],
            ]
        );
        Ok(())
    }

    #[test]
    fn example1() -> anyhow::Result<()> {
        use BlockShape::{I, J, L, Q, S, T, Z};
        assert_eq!(
            process_blocks([(I, 0), (I, 4), (Q, 8)])?,
            grid![
                [. . . . . . . . . . ],
                [. . . . . . . . . . ],
                [. . . . . . . . # # ],
            ]
        );
        Ok(())
    }
}
