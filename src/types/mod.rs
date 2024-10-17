pub mod action;
pub mod field;

/*
* A collection of tiny helper enums used
* by multiple parts of the system.
* As they don't really make sense to associate
* with any other module, they're collected here.
*/
#[derive(Clone, Copy, Debug)]
pub enum Axis {
    Row,
    Column,
}

impl Axis {
    pub fn opposite(&self) -> Self {
        match self {
            Axis::Row => Axis::Column,
            Axis::Column => Axis::Row,
        }
    }
}

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

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Coordinate {
    pub row: usize,
    pub column: usize,
}

impl Coordinate {
    pub fn get_axis_index(self, axis: Axis) -> usize {
        match axis {
            Axis::Row => self.row,
            Axis::Column => self.column,
        }
    }
    pub fn set_axis_index(&mut self, axis: Axis, index: usize) {
        match axis {
            Axis::Row => self.row = index,
            Axis::Column => self.column = index,
        }
    }

    // This is very silly, but I simply do not think in row-major or zero-index
    // when playing grid-based games. Blame chess I guess.
    pub fn printable(&self) -> String {
        format!("[{}, {}]", self.column + 1, self.row + 1)
    }
    pub fn from_user(column: usize, row: usize) -> Coordinate {
        Coordinate {
            row: row - 1,
            column: column - 1,
        }
    }
}
