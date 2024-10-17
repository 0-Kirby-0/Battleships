use Argument::{Known, Unknown};
use crate::types::Coordinate;
use strum_macros::EnumIter;

#[derive(Clone, Copy, Default)]
pub enum Argument<T: Clone + Copy + Default> {
    //Basically an Option<>, just with a more fitting name
    Known(T),
    #[default]
    Unknown,
}

#[derive(EnumIter, Clone, Copy)]
pub enum Action {
    Fire(Argument<Coordinate>),
    Hit(Argument<Coordinate>),
    Sink(Argument<usize>),
    Unfire(Argument<Coordinate>),
    Unsink(Argument<usize>),
    Undo,
}

impl Action {
    pub fn name(&self) -> &str {
        match *self {
            Action::Fire(_) => "fire",
            Action::Hit(_) => "hit",
            Action::Sink(_) => "sink",
            Action::Unfire(_) => "unfire",
            Action::Unsink(_) => "unsink",
            Action::Undo => "undo",
        }
    }

    pub fn opposite(&self) -> Action {
        match *self {
            Action::Fire(content) => Action::Unfire(content),
            Action::Hit(content) => Action::Unfire(content),
            Action::Sink(content) => Action::Unsink(content),
            Action::Unfire(content) => Action::Fire(content),
            Action::Unsink(content) => Action::Sink(content),
            Action::Undo => unreachable!("There exists no opposite of 'Undo'."),
        }
    }

    pub fn expected_arg_count(&self) -> usize {
        match *self {
            Action::Fire(_) | Action::Unfire(_) | Action::Hit(_) => 2,
            Action::Sink(_) | Action::Unsink(_) => 1,
            Action::Undo => 0,
        }
    }

    pub fn can_infer_args(&self) -> bool {
        match *self {
            Self::Fire(_)
            | Action::Unfire(_)
            | Action::Hit(_)
            | Action::Unsink(_)
            | Action::Undo => true,
            Action::Sink(_) => false,
        }
    }

    pub fn tx_syntax_help(&self) -> String {
        match self {
                    Action::Fire(_) => "'fire <column> <row>' [1-index] Fires at the specified coordinate.\n\tDefault: Executes most recent recommendation.".to_owned(),
                    Action::Hit(_) => "'hit <column> <row>' [1-index] Marks the specified coordinate as hit.\n\tDefault: Marks the most recently fired at coodinate as hit.".to_owned(),
                    Action::Sink(_) => "'sink <ship length>' Removes one ship of the specified length from the list.\n\tUnfortunately the length cannot logically be inferred.".to_owned(),
                    Action::Unfire(_) => "'unfire <column> <row>' [1-index] Removes specified fireing marker.\n\tDefault: Undoes most recent fire command.".to_owned(),
                    Action::Unsink(_) => "'unsink <ship length>' Adds one ship of the specified length to the list.\n\tDefault: Undoes the most recent sink command.".to_owned(),
                    Action::Undo => "'undo' Undoes the most recent action.".to_owned(),
                }
    }

    pub fn tx_success(&self) -> String {
        match self {
            Action::Fire(Known(coordinate)) => format!("Fired at {}.", coordinate.printable()),
            Action::Hit(Known(coordinate)) => {
                format!("Set hit marker at {}.", coordinate.printable())
            }
            Action::Sink(Known(ship_length)) => format!("Sunk a ship of length {ship_length}." ),
            Action::Unfire(Known(coordinate)) => {
                format!("Removed fire marker at {}.", coordinate.printable())
            }
            Action::Unsink(Known(ship_length)) => {
                format!("Added a ship of length {ship_length} to the roster." )
            }
            Action::Undo => unreachable!(
                "When undoing, the success message printed should be that of the action executed."
            ),

            Action::Fire(Unknown) 
            | Action::Unfire(Unknown) 
            | Action::Hit(Unknown) 
            | Action::Sink(Unknown) 
            | Action::Unsink(Unknown) => unreachable!("Since actions with unknown args cannot be executed, there should not be a success message.")
        }
    }
}