pub mod action;
pub use field::helpers::Coordinate;
/*
* A collection of tiny helper enums used
* by multiple parts of the system.
* As they don't really make sense to associate
* with any other module, they're collected here.
*/
#[derive(Clone, Copy, Default, Debug)]
pub enum ShotStatus {
    #[default]
    Untested,
    Miss,
    Hit,
    Sunk,
}

impl ShotStatus {
    pub fn can_contain_ship(&self) -> bool {
        match self {
            ShotStatus::Untested | ShotStatus::Hit => true,
            ShotStatus::Miss | ShotStatus::Sunk => false,
        }
    }
}

pub trait Printable {
    fn printable(&self) -> String;
    fn from_user(column: usize, row: usize) -> Coordinate;
}

impl Printable for Coordinate {
    // This is very silly, but I simply do not think in row-major or zero-index
    // when playing grid-based games. Blame chess I guess.
    fn printable(&self) -> String {
        format!("[{}, {}]", self.column + 1, self.row + 1)
    }
    fn from_user(column: usize, row: usize) -> Coordinate {
        Coordinate {
            row: row - 1,
            column: column - 1,
        }
    }
}
