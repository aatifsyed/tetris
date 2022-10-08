use anyhow::Context;
use clap::Parser;
use derive_more::From;
use indoc::indoc;
use recap::Recap;
use serde::Deserialize;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    ops::Shr,
    path::{Path, PathBuf},
    str::FromStr,
};
use strum::EnumString;
use tetris::{is_occupied, CellState, Grid};

/// From brief
const WIDTH: usize = 10;
/// From brief, with allowance for the tallest block
const HEIGHT: usize = 100 + 3;

// todo: add tracing etc
#[derive(Debug, Parser)]
#[command(about = indoc!("
    DRW Tetris
    ==========
    
    For each line in the input, interpret that line as a comma-separated sequence of INPUT_BLOCK, where
    INPUT_BLOCK : { 'Q', 'Z', 'S', 'T', 'I', 'L', 'J' } + DIGIT
    
    Each INPUT_BLOCK is placed on a 10 * 103 GRID at INPUT_BLOCK.DIGIT position, and dropped.
    Rows clear in typical tetris style.
    
    After the sequence has been processed, print the height of the tallest occupied row.
    "))]
struct Args {
    /// The input file (defaults to stdin)
    #[arg(short, long)]
    infile: Option<PathBuf>,
    /// The output file (defaults to stdout)
    #[arg(short, long)]
    outfile: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let infile = or_stdin(args.infile)?;
    let mut outfile = or_stdout(args.outfile)?;
    for line in infile.lines() {
        let input_blocks =
            parse_line(&line.context("couldn't read input")?).context("couldn't parse line")?;
        writeln!(
            outfile,
            "{}",
            highest_block_after_processing(Grid::<WIDTH, HEIGHT>::default(), input_blocks)
                .context("couldn't place input block on congested grid")?
        )
        .context("couldn't write output")?;
    }
    outfile.flush().context("couldn't write output")?;
    Ok(())
}

fn or_stdin(path: Option<impl AsRef<Path>>) -> anyhow::Result<Box<dyn BufRead>> {
    if let Some(path) = path {
        match File::open(path) {
            Ok(read) => Ok(Box::new(BufReader::new(read))),
            Err(e) => Err(e).context("couldn't open input"),
        }
    } else {
        Ok(Box::new(io::stdin().lock()))
    }
}
fn or_stdout(path: Option<impl AsRef<Path>>) -> anyhow::Result<Box<dyn Write>> {
    if let Some(path) = path {
        match File::create(path) {
            Ok(write) => Ok(Box::new(write)),
            Err(e) => Err(e).context("couldn't open output"),
        }
    } else {
        Ok(Box::new(io::stdout()))
    }
}
/// drop each [InputBlock] onto a [Grid], and clear rows, returning the final state of the grid
fn process_blocks<const WIDTH: usize, const HEIGHT: usize>(
    mut grid: Grid<WIDTH, HEIGHT>,
    blocks: impl IntoIterator<Item = impl Into<InputBlock>>,
) -> anyhow::Result<Grid<WIDTH, HEIGHT>> {
    for block in blocks {
        let InputBlock {
            shape,
            starting_column,
        } = block.into();
        let new_shape = grid_for(shape).shr(starting_column);
        grid = grid
            .drop(new_shape)
            .context("grid's top row are already occupied")?
            .with_solid_rows_cleared();
    }
    Ok(grid)
}

fn first_occupied_row_ix<const WIDTH: usize, const HEIGHT: usize>(
    grid: &Grid<WIDTH, HEIGHT>,
) -> Option<usize> {
    (0..HEIGHT).find(|&row_ix| grid.rows[row_ix].iter().any(is_occupied))
}
fn highest_block<const WIDTH: usize, const HEIGHT: usize>(grid: &Grid<WIDTH, HEIGHT>) -> usize {
    first_occupied_row_ix(grid)
        .map(|row_ix| HEIGHT - row_ix)
        .unwrap_or(0)
}

fn highest_block_after_processing<const WIDTH: usize, const HEIGHT: usize>(
    grid: Grid<WIDTH, HEIGHT>,
    blocks: impl IntoIterator<Item = impl Into<InputBlock>>,
) -> anyhow::Result<usize> {
    let final_grid = process_blocks(grid, blocks)?;
    Ok(highest_block(&final_grid))
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

#[derive(Debug, Deserialize, Recap, PartialEq, Eq, Clone, Copy, From)]
#[recap(regex = r#"(?P<shape>\w)(?P<starting_column>\d+)"#)]
struct InputBlock {
    pub shape: BlockShape,
    pub starting_column: usize,
}

// todo: make a grammar and use a parser
fn parse_line(s: &str) -> anyhow::Result<Vec<InputBlock>> {
    Ok(s.trim()
        .split(',')
        .map(InputBlock::from_str)
        .collect::<Result<Vec<_>, _>>()?)
}

/// Place `with` in each of the `coords`
/// # Panics
/// - If any of the coords are out of bounds
fn fill<const WIDTH: usize, const HEIGHT: usize, CellT: Clone>(
    grid: &mut Grid<WIDTH, HEIGHT, CellT>,
    with: CellT,
    coords: impl IntoIterator<Item = (usize, usize)>,
) {
    for (row_ix, col_ix) in coords {
        grid.rows[row_ix][col_ix] = with.clone();
    }
}

/// Place a [BlockShape] in a new [Grid]
/// # Panics
/// - If the grid is too small to fit the shape
fn grid_for<const WIDTH: usize, const HEIGHT: usize>(shape: BlockShape) -> Grid<WIDTH, HEIGHT> {
    use CellState::Occupied as X;
    // once const rust is more mature, we can static assert that WIDTH fits I and HEIGHT fits J/L
    // (the code will currently panic)
    let mut grid = Grid::default();
    match shape {
        BlockShape::Q => fill(&mut grid, X, [(0, 0), (0, 1), (1, 0), (1, 1)]),
        BlockShape::Z => fill(&mut grid, X, [(0, 0), (0, 1), (1, 1), (1, 2)]),
        BlockShape::S => fill(&mut grid, X, [(0, 1), (0, 2), (1, 0), (1, 1)]),
        BlockShape::T => fill(&mut grid, X, [(0, 0), (0, 1), (0, 2), (1, 1)]),
        BlockShape::I => fill(&mut grid, X, [(0, 0), (0, 1), (0, 2), (0, 3)]),
        BlockShape::L => fill(&mut grid, X, [(0, 0), (1, 0), (2, 0), (2, 1)]),
        BlockShape::J => fill(&mut grid, X, [(0, 1), (1, 1), (2, 1), (2, 0)]),
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use tetris::grid;
    use BlockShape::{I, J, L, Q, S, T, Z};

    #[test]
    fn parse1() -> anyhow::Result<()> {
        use BlockShape::{I, Q};
        assert_eq!(
            parse_line("I0,I4,Q8")?,
            vec![
                InputBlock::from((I, 0)),
                InputBlock::from((I, 4)),
                InputBlock::from((Q, 8))
            ]
        );
        Ok(())
    }
    #[test]
    fn shapes() -> anyhow::Result<()> {
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

    const EXAMPLE1: [(BlockShape, usize); 3] = [(I, 0), (I, 4), (Q, 8)];
    const EXAMPLE2: [(BlockShape, usize); 3] = [(T, 1), (Z, 3), (I, 4)];
    const EXAMPLE3: [(BlockShape, usize); 8] = [
        (Q, 0),
        (I, 2),
        (I, 6),
        (I, 0),
        (I, 6),
        (I, 6),
        (Q, 2),
        (Q, 4),
    ];

    #[test]
    fn process_example1() -> anyhow::Result<()> {
        assert_eq!(
            process_blocks(Grid::default(), EXAMPLE1)?,
            grid![
                [. . . . . . . . . . ],
                [. . . . . . . . . . ],
                [. . . . . . . . # # ],
            ]
        );
        Ok(())
    }

    #[test]
    fn process_example2() -> anyhow::Result<()> {
        assert_eq!(
            process_blocks(Grid::default(), EXAMPLE2)?,
            grid![
                [. . . . # # # # . . ],
                [. . . # # . . . . . ],
                [. # # # # # . . . . ],
                [. . # . . . . . . . ],
            ]
        );
        Ok(())
    }

    #[test]
    fn process_example3() -> anyhow::Result<()> {
        assert_eq!(
            process_blocks(Grid::default(), EXAMPLE3)?,
            grid![
                [. . . . . . . . . .],
                [. . . . . . . . . .],
                [. . # # . . . . . .],
                [. . # # . . . . . .],
                [# # . . # # # # # #],
            ]
        );
        Ok(())
    }
    #[test]
    fn highest_block_example1() -> anyhow::Result<()> {
        assert_eq!(
            highest_block_after_processing(Grid::<WIDTH, HEIGHT>::default(), EXAMPLE1)?,
            1
        );
        Ok(())
    }

    #[test]
    fn highest_block_example2() -> anyhow::Result<()> {
        assert_eq!(
            highest_block_after_processing(Grid::<WIDTH, HEIGHT>::default(), EXAMPLE2)?,
            4
        );
        Ok(())
    }

    #[test]
    fn highest_block_example3() -> anyhow::Result<()> {
        assert_eq!(
            highest_block_after_processing(Grid::<WIDTH, HEIGHT>::default(), EXAMPLE3)?,
            3
        );
        Ok(())
    }
}
