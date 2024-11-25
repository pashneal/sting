use std::collections::HashMap;


// DSL for the board
// TODO

#[derive(Copy, Clone, Debug)]
pub enum PieceType {
    Queen,
    Grasshopper,
    Spider,
    Beetle,
    Ant,
    Pillbug,
    Ladybug,
    Mosquito,
}

impl PieceType {
    pub fn to_str(&self) -> &str {
        use PieceType::*;
        match self {
            Queen => "Q",
            Grasshopper => "G",
            Spider => "S",
            Beetle => "B",
            Ant => "A",
            Pillbug => "P",
            Ladybug => "L",
            Mosquito => "M",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PieceColor {
    Black,
    White,
}

#[derive(Copy, Clone, Debug)]
pub struct Piece {
    pub piece : PieceType,
    pub color : PieceColor,
}

impl Piece {
    pub fn new(piece: PieceType, color: PieceColor) -> Piece {
        Piece { piece, color }
    }

    /// Uppercase letter for white, lowercase for black
    pub fn to_str(&self) -> String {
        let piece_str = match self.color {
            PieceColor::White => self.piece.to_str().to_uppercase(),
            PieceColor::Black => self.piece.to_str().to_lowercase(),
        };

        piece_str
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    NW,
    NE,
    E,
    SE,
    SW,
    W,
}

#[derive(Copy, Clone, Debug)]
pub struct HexLocation {
    pub x: i8,
    pub y: i8,
}

impl HexLocation {
    pub fn new(x: i8, y: i8) -> HexLocation {
        HexLocation { x, y }
    }

    pub fn apply(&self, direction : Direction) -> Self {
        use Direction::*;
        let (mut x, mut y) = (self.x, self.y);
        match direction {
            NW => { y -= 1 }
            E => { x += 1 }
            W => { x -= 1 }
            SE => { y += 1 }
            NE => { x += 1; y -= 1 }
            SW => { x -= 1; y += 1 }
        }
        HexLocation::new(x, y)
    }
}


pub const HEX_GRID_SIZE: usize = 60;
pub const HEX_GRID_CENTER: (usize, usize) = (HEX_GRID_SIZE / 2, HEX_GRID_SIZE / 2);
pub const MAX_HEIGHT: usize = 7;

/// Represents a hexagonal grid
///
/// The coordinate system is axial as found here:
/// https://www.redblobgames.com/grids/hexagons/
///
/// As pieces can potentially stack, they are filled from the 
/// first element of the array to the last
///
/// HexLocation 0,0 is in the center of the grid to make 
/// the grid easier to reason about as Hive is a boardless "floating" game
pub struct HexGrid {
    grid: [[[Option<Piece>; MAX_HEIGHT]; HEX_GRID_SIZE]; HEX_GRID_SIZE],
}

impl HexGrid {
    pub fn new() -> HexGrid {
        HexGrid {
            grid: [[[None; MAX_HEIGHT]; HEX_GRID_SIZE]; HEX_GRID_SIZE],
        }
    }

    fn centralize(location: HexLocation) -> (usize, usize){
        let (x, y) = (location.x, location.y);
        let (x, y) = (x + HEX_GRID_CENTER.0 as i8, y + HEX_GRID_CENTER.1 as i8);
        (x as usize, y as usize)
    }

    pub fn add(&mut self, piece: Piece, location: HexLocation) {
        let (x, y) = HexGrid::centralize(location);
        for i in 0..MAX_HEIGHT {
            if self.grid[y][x][i].is_none() {
                self.grid[y][x][i] = Some(piece);
                break;
            }
        }
    }

    pub fn remove(&mut self, location: HexLocation) -> Option<Piece> {
        let (x, y) = HexGrid::centralize(location);
        for i in 0..MAX_HEIGHT {
            if self.grid[y][x][i].is_some() {

                let piece = self.grid[y][x][i];
                self.grid[y][x][i] = None;
                return piece;
            }
        }

        None
    }



    pub fn peek(&self, location: HexLocation) -> Vec<Option<Piece>> {
        let (x,y) = HexGrid::centralize(location);
        self.axial(x,y)
    }

    /// Access the grid using the axial coordinate system
    /// https://www.redblobgames.com/grids/hexagons/#coordinates-cube
    fn axial(&self, x : usize, y : usize) -> Vec<Option<Piece>> {
        let mut pieces = vec![];
        for piece in self.grid[y][x] {
            if piece.is_some() {
                pieces.push(piece);
            }
        }
        return pieces
    }

    fn oddr_to_axial(&self, row: usize, col: usize) -> (i8, i8) {
        let q = col as i8 - (row as i8 - ((row as i8) & 1)) / 2;
        let r = row as i8;
        (q, r)
    }

    /// Access the grid using the odd-r coordinate system
    /// https://www.redblobgames.com/grids/hexagons/#coordinates-offset
    fn oddr(&self, row: usize, col: usize) -> Vec<Option<Piece>> {
        let (q, r) = self.oddr_to_axial(row, col);
        if q < 0 { return vec![]; } // out of bounds
        self.axial(q as usize, r as usize)
    }

    pub fn move_piece(&mut self, from: HexLocation, to: HexLocation) {
        let piece = self.remove(from);
        self.add(piece.unwrap(), to);
    }

    /// Outputs the current state of the grid according to the DSL
    /// specified above
    ///
    /// example:
    ///
    ///  . . . . .
    ///   . Q 3 g .
    ///  . . A b . 
    ///   . 2 . m .
    ///  . . . . .
    ///
    ///  start - [ 3, -2 ]
    ///
    ///  3 - [G b B] 
    ///  2 - [a M]
    pub fn to_dsl(&self) -> String {
        self.board_string() + "\n" + &self.start_string() + "\n" + &self.stacks_string()
    }

    /// Returns the coordinate of the top-most and left-most corner of the
    /// grid when returned by board_string() in a form compatible
    /// with the DSL
    ///
    /// Will have the format
    /// start - [ <x>, <y> ]
    pub fn start_string(&self) ->  String {
        let ((top, left), _) = self.bounds();

        let top_row = top - 1;
        let left_col = left - 1;

        let (left_q, top_r) = self.oddr_to_axial(top_row, left_col);
        let (left, top) = (left_q - HEX_GRID_CENTER.0 as i8, top_r - HEX_GRID_CENTER.1 as i8);

        let mut start = "start - [".to_owned();
        start.push_str(&format!(" {}, {} ", left, top));
        start.push_str("]");
        start
    }

    /// Outputs the stack part of this current grid according to the DSL
    /// specified above. 
    ///
    /// Will have the format:
    /// <number> - [ <piece> <piece> ... ]
    /// <number> - [ <piece> <piece> ... ]
    /// ...
    pub fn stacks_string(&self) -> String {
        let (min, max) = self.bounds();
        let (top, left) = min;
        let (bottom, right) = max;

        let mut stack_string = String::new();


        for row in top..=bottom {
            for col in left..=right {
                print!("({}, {}) ", row, col);
                let pieces = self.oddr(row, col);
                if pieces.len() > 1 {
                    stack_string.push_str(&format!("{} - [ ", pieces.len()));
                    for piece in pieces {
                        if piece.is_none() {
                            break;
                        }
                        stack_string.push_str(&piece.as_ref().unwrap().to_str());
                        stack_string.push_str(" ");
                    }
                    stack_string.push_str("]\n");
                }
            }
        }

        stack_string

    }

    /// Outputs the board part of this current grid according to the DSL
    /// specified above
    ///
    /// example:
    ///
    ///  . . . . .
    ///   . Q 3 g .
    ///  . . A b . 
    ///   . 2 . m .
    ///  . . . . .
    ///
    pub fn board_string(&self) -> String {
        if self.is_empty() {
            return ".".to_owned();
        }

        let (min, max) = self.bounds();
        let (min_row, min_col) = min;
        let (max_row, max_col) = max;
        
        let left = min_col-1;
        let right = max_col+1;
        let bottom = max_row+1;
        let top = min_row-1;

        let mut board = String::new();

        for row in top..=bottom {
            if row % 2 == 1 {
                board.push_str(" ");
            }
            for col in left..=right {
                let pieces = self.oddr(row, col);
                
                match pieces.len() {
                    0 => board.push_str("."),
                    1 => board.push_str(&pieces[0].as_ref().unwrap().to_str()),
                    _ => board.push_str(&format!("{}", pieces.len())),
                
                }
                // Do not add a space after the last column
                if col != right {   
                    board.push_str(" ");
                }
            }
            board.push_str("\n");
        }

        return board;
    }

    /// Returns a bounding box around all present pieces
    /// in the grid according the odd_r format as described here: 
    /// https://www.redblobgames.com/grids/hexagons/#coordinates-offset
    fn bounds(&self)  -> ((usize, usize), (usize, usize)) {
        let mut min_row = HEX_GRID_SIZE;
        let mut min_col = HEX_GRID_SIZE;
        let mut max_row = 0;
        let mut max_col = 0;

        for row in 0..HEX_GRID_SIZE {
            for col in 0..HEX_GRID_SIZE {
                if self.oddr(row, col).len() > 0{
                    min_row = min_row.min(row);
                    min_col = min_col.min(col);
                    max_row = max_row.max(row);
                    max_col = max_col.max(col);
                }
            }
        }

        ((min_row, min_col), (max_row, max_col))

    }


    /// Checks to see if the board contains no pieces 
    fn is_empty(&self) -> bool {
        for y in 0..HEX_GRID_SIZE {
            for x in 0..HEX_GRID_SIZE {
                if self.grid[y][x][0].is_some() {
                        return false;
                }
            }
        }

        true
    }
}


#[test]
fn test_board_string_empty(){
    let grid = HexGrid::new();
    let board = grid.board_string();
    let expected = ".";
    assert_eq!(board, expected, "Empty board should be a single dot");
}


#[test]
fn test_board_string_single(){
    let mut grid = HexGrid::new();
    let piece = Piece::new(PieceType::Queen, PieceColor::White);
    grid.add(piece, HexLocation::new(0,1));
    let board = grid.board_string();
    let expected = concat!(
        ". . .\n",
        " . Q .\n",
        ". . .\n",
    );

    assert_eq!(board, expected, "Should represent a single piece");
}

#[test]
fn test_board_string_single_stack(){
    let mut grid = HexGrid::new();
    let piece = Piece::new(PieceType::Queen, PieceColor::White);
    let dummy = Piece::new(PieceType::Pillbug, PieceColor::Black);
    grid.add(piece, HexLocation::new(0,1));
    grid.add(dummy, HexLocation::new(0,1));
    let board = grid.board_string();
    let expected = concat!(
        ". . .\n",
        " . 2 .\n",
        ". . .\n",
    );

    assert_eq!(board, expected, "Should represent a single stack");
}

#[test]
fn test_board_string_multiple(){
    let mut grid = HexGrid::new();
    let white_queen = Piece::new(PieceType::Queen, PieceColor::White);
    let white_ant = Piece::new(PieceType::Ant, PieceColor::White);
    let black_beetle = Piece::new(PieceType::Beetle, PieceColor::Black);
    let black_mosquito = Piece::new(PieceType::Mosquito, PieceColor::Black);
    let black_grasshopper = Piece::new(PieceType::Grasshopper, PieceColor::Black);
    let black_spider = Piece::new(PieceType::Spider, PieceColor::Black);
    let white_ladybug = Piece::new(PieceType::Ladybug, PieceColor::White);
    let white_beetle = Piece::new(PieceType::Beetle, PieceColor::White);
    let dummy = Piece::new(PieceType::Pillbug, PieceColor::Black);

    let start = HexLocation::new(10,-3);
    let white_ant_loc = start;
    let white_queen_loc = white_ant_loc.apply(Direction::NW);
    let black_beetle_loc = white_ant_loc.apply(Direction::E);
    let black_mosquito_loc = black_beetle_loc.apply(Direction::SE);
    let black_grasshopper_loc = black_beetle_loc.apply(Direction::NE);
    let stack2_loc = white_ant_loc.apply(Direction::SW);
    let stack3_loc = white_ant_loc.apply(Direction::NE);
    let black_spider_loc = stack2_loc.apply(Direction::SW);
    let white_ladybug_loc = black_spider_loc.apply(Direction::SE);
    let white_beetle_loc = white_ladybug_loc.apply(Direction::SW);


    grid.add(white_queen, white_queen_loc);
    grid.add(white_ant, white_ant_loc);
    grid.add(black_beetle, black_beetle_loc);
    grid.add(black_mosquito, black_mosquito_loc);
    grid.add(black_grasshopper, black_grasshopper_loc);
    grid.add(black_spider, black_spider_loc);
    grid.add(white_ladybug, white_ladybug_loc);
    grid.add(white_beetle, white_beetle_loc);

    grid.add(dummy, stack2_loc);
    grid.add(dummy, stack2_loc);

    grid.add(dummy, stack3_loc);
    grid.add(dummy, stack3_loc);
    grid.add(dummy, stack3_loc);


    let expected = concat!(
        " . . . . . .\n",
        ". . Q 3 g .\n",
        " . . A b . .\n",
        ". . 2 . m .\n",
        " . s . . . .\n",
        ". . L . . .\n",
        " . B . . . .\n",
        ". . . . . .\n",
    );

    let board = grid.board_string();
    println!("{}", board);

    assert_eq!(board, expected);
}

#[test]
fn test_board_multiple_stacks(){
    let mut grid = HexGrid::new();
    let white_queen = Piece::new(PieceType::Queen, PieceColor::White);
    let white_ant = Piece::new(PieceType::Ant, PieceColor::White);
    let black_beetle = Piece::new(PieceType::Beetle, PieceColor::Black);
    let black_mosquito = Piece::new(PieceType::Mosquito, PieceColor::Black);
    let black_grasshopper = Piece::new(PieceType::Grasshopper, PieceColor::Black);
    let black_spider = Piece::new(PieceType::Spider, PieceColor::Black);
    let white_ladybug = Piece::new(PieceType::Ladybug, PieceColor::White);
    let white_beetle = Piece::new(PieceType::Beetle, PieceColor::White);

    let start = HexLocation::new(2,0);
    let white_ant_loc = start;
    let white_queen_loc = white_ant_loc.apply(Direction::NW);
    let black_beetle_loc = white_ant_loc.apply(Direction::E);
    let black_mosquito_loc = black_beetle_loc.apply(Direction::SE);
    let black_grasshopper_loc = black_beetle_loc.apply(Direction::NE);
    let stack2_loc = white_ant_loc.apply(Direction::SW);
    let stack3_loc = white_ant_loc.apply(Direction::NE);
    let black_spider_loc = stack2_loc.apply(Direction::SW);
    let white_ladybug_loc = black_spider_loc.apply(Direction::SE);
    let white_beetle_loc = white_ladybug_loc.apply(Direction::SW);


    grid.add(white_queen, white_queen_loc);
    grid.add(white_queen, white_queen_loc);
    grid.add(white_queen, white_queen_loc);

    grid.add(white_ant, white_ant_loc);
    grid.add(black_beetle, black_beetle_loc);

    grid.add(black_mosquito, black_mosquito_loc);
    grid.add(black_mosquito, black_mosquito_loc);

    grid.add(black_grasshopper, black_grasshopper_loc);
    grid.add(black_grasshopper, black_grasshopper_loc);

    grid.add(black_spider, black_spider_loc);
    grid.add(white_ladybug, white_ladybug_loc);
    grid.add(white_beetle, white_beetle_loc);

    grid.add(black_beetle, stack2_loc);
    grid.add(black_mosquito, stack2_loc);

    grid.add(white_queen, stack3_loc);
    grid.add(black_grasshopper, stack3_loc);
    grid.add(white_ant, stack3_loc);


    // Modeled after the board string:
    //let expected = concat!(
        //" . . . . . .\n",
        //". . 3 3 2 .\n",
        //" . . A b . .\n",
        //". . 2 . 2 .\n",
        //" . s . . . .\n",
        //". . L . . .\n",
        //" . B . . . .\n",
        //". . . . . .\n",
    //);

    let expected = concat!(
        "3 - [ Q Q Q ]\n",
        "3 - [ Q g A ]\n",
        "2 - [ g g ]\n",
        "2 - [ b m ]\n",
        "2 - [ m m ]\n",
    );
    let board = grid.stacks_string();
    println!("{}", board);

    assert_eq!(board, expected);
}


#[test]
fn test_board_string_padding(){
    let start = HexLocation::new(0,0);
    let ne = start.apply(Direction::NE);
    let se = start.apply(Direction::SE);

    let expected = concat!(
        ". . .\n",
        " . Q .\n",
        ". Q .\n",
        " . Q .\n",
        ". . .\n",
    );

    let mut grid = HexGrid::new();
    let white_queen = Piece::new(PieceType::Queen, PieceColor::White);
    grid.add(white_queen, start);
    grid.add(white_queen, ne);
    grid.add(white_queen, se);

    let board = grid.board_string();
    println!("{}", board);
    assert_eq!(board, expected);

    let start = HexLocation::new(0,1);
    let nw = start.apply(Direction::NW);
    let sw = start.apply(Direction::SW);

    let expected = concat!(
        " . . .\n",
        ". Q .\n",
        " . Q .\n",
        ". Q .\n",
        " . . .\n",
    );

    let mut grid = HexGrid::new();
    let white_queen = Piece::new(PieceType::Queen, PieceColor::White);
    grid.add(white_queen, start);
    grid.add(white_queen, nw);
    grid.add(white_queen, sw);

    let board = grid.board_string();
    println!("{}", board);
    assert_eq!(board, expected);
}


#[test]
fn test_start_string1(){
    let start = HexLocation::new(0,0);
    let ne = start.apply(Direction::NE);
    let se = start.apply(Direction::SE);

    // The board looks like this:
    //let expected = concat!(
        //". . .\n",
        //" . Q .\n",
        //". Q .\n",
        //" . Q .\n",
        //". . .\n",
    //);

    let mut grid = HexGrid::new();
    let white_queen = Piece::new(PieceType::Queen, PieceColor::White);
    grid.add(white_queen, start);
    grid.add(white_queen, ne);
    grid.add(white_queen, se);

    let start_string = grid.start_string();
    let expected = "start - [ 0, -2 ]";
    assert_eq!(start_string, expected);
}


#[test]
fn test_start_string2(){
    let start = HexLocation::new(5,-6);
    let nw = start.apply(Direction::NW);
    let sw = start.apply(Direction::SW);

    // board  expected to look like:
    //. . . .
    // . Q . .
    //. . Q .
    // . Q . .
    //. . . .

    let mut grid = HexGrid::new();
    let white_queen = Piece::new(PieceType::Queen, PieceColor::White);
    grid.add(white_queen, start);
    grid.add(white_queen, nw);
    grid.add(white_queen, sw);

    let board = grid.board_string();
    println!("{}", board);

    let start_string = grid.start_string();
    let expected = "start - [ 4, -8 ]";
    assert_eq!(start_string, expected);
}
