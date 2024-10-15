use crate::field::Field;
use crate::heatmap;
use crate::types::{Axis, Action, Coordinate, ShotStatus};
use crate::types::Argument::{Known, Unknown};

use anyhow::{anyhow, Ok, Result};



pub struct State {
    shots: Field<ShotStatus>,
    ships: Vec<usize>,
    heat_field: Field<f32>,
    top_moves: Vec<Coordinate>,
    action_history: Vec<Action>,
}
impl State {
    pub fn new(width: usize, height: usize, ships: &[usize]) -> Self {
        let shots: Field<ShotStatus> = Field::new_default(width, height);
        let ships = ships.to_vec();
        let heat_field = heatmap::gen_heat_field(&shots, &ships);
        let top_moves= State::generate_top_moves(&heat_field);
        let action_history = vec![];

        State {
            shots,
            ships,
            heat_field,
            top_moves,
            action_history,
        }
    }

    fn update(&mut self) {
        self.heat_field = heatmap::gen_heat_field(&self.shots, &self.ships);
        self.top_moves = State::generate_top_moves(&self.heat_field);
    }

    pub fn take_action(&mut self, mut action: Action) -> Result<()> {

        //if the action is an "undo", convert it to the opposite of the last action
        let mut action_was_undo= false;
        if std::mem::discriminant(&action) == std::mem::discriminant(&Action::Undo){
            action= self.get_last_action()?.opposite();
            self.action_history.pop();
            action_was_undo= true;
        }
        
        //execute the action
        match action {
            Action::Fire(Known(coord)) => self.shots.set_value(coord, &ShotStatus::Miss)?,
            Action::Unfire(Known(coord)) => self.shots.set_value(coord, &ShotStatus::Untested)?,
            Action::Hit(Known(coord)) => self.shots.set_value(coord, &ShotStatus::Hit)?,
            Action::Sink(Known(ship_length)) => self.sink_ship(ship_length)?,
            Action::Unsink(Known(ship_length)) => self.ships.push(ship_length),

            Action::Fire(Unknown)
            | Action::Unfire(Unknown) | Action::Hit(Unknown)
            | Action::Sink(Unknown)
            | Action::Unsink(Unknown) => {
                unreachable!("Actions with unknown arguments cannot be taken.")
            }

            Action::Undo => unreachable!("Undos were already converted to the appropriate action."),
        };

        if !action_was_undo {
            self.action_history.push(action);
        }
 
        self.update();


        Ok(())
    }

    fn sink_ship(&mut self, ship_length: usize) -> Result<()> {
        let position = self
            .ships
            .iter()
            .position(|&ship| ship == ship_length)
            .ok_or_else(|| anyhow::anyhow!("Ship not found."))?;

        let ship_locations= self.generate_possible_ship_locations(ship_length);
        if ship_locations.is_empty() {return Err(anyhow::anyhow!("Ship doesn't fit existing hits."));}

        self.ships.remove(position);

        if self.ships.is_empty() {
            return Err(anyhow::anyhow!("Game is over, go home :)"));
        }

        


        let chosen_location= if ship_locations.len() == 1 {ship_locations.first().unwrap().clone()} else {
            Self::ask_user_for_ship_location(ship_locations)
        };

        for coord in chosen_location {
            self.shots.set_value(coord, &ShotStatus::Sunk).unwrap();
        }

      



        Ok(())
    }

    fn generate_possible_ship_locations(&self, ship_length: usize) -> Vec<Vec<Coordinate>>{
        [Axis::Row, Axis::Column].iter().flat_map(|&axis|{
            (0..self.shots.number_of_lines_in_axis(axis)).flat_map(|index|{
                self.shots.get_line(axis, index).unwrap().windows(ship_length).enumerate().filter(|(_,window)|{
                    window.iter().all(|status| std::mem::discriminant(status) == std::mem::discriminant(&ShotStatus::Hit))
                }).map(|(off_axis_idx, _)|{
                    (0..ship_length).map(move|offset|{
                        let mut coord= Coordinate::default();
                        coord.set_axis_index(axis, index);
                        coord.set_axis_index(axis.opposite(), off_axis_idx+offset);
                        coord
                    }).collect::<Vec<_>>()
                }).collect::<Vec<_>>()
            }).collect::<Vec<_>>()
        }).collect()
    }

    fn ask_user_for_ship_location(ship_locations: Vec<Vec<Coordinate>>) -> Vec<Coordinate>{
        use std::io::Write;

        println!("The ship to sink could be in multiple places. Please select one:");
        for (idx, location) in ship_locations.iter().enumerate(){
            print!("{}: ",idx+1);
            for coord in location {
                print!("{}", coord.printable())
            }
            println!()
        }

        loop{
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if let std::result::Result::Ok(idx) = input.trim().parse::<usize>() {
                if let Some(location) = ship_locations.get(idx.saturating_sub(1)){
                    return location.clone();
                }
            }
            println!("Invalid, please try again.");

        }
        
      
    }

    pub fn get_top_moves(&self) -> Vec<Coordinate>{
        self.top_moves.clone()
    }

    fn generate_top_moves(heat_field: &Field<f32>) -> Vec<Coordinate> {
        let mut max_val= f32::NEG_INFINITY;
        let mut top_moves= vec![];
        heat_field.get_grid().iter().enumerate().flat_map(|(row_idx, row)|{
            row.iter().enumerate().map(move |(column_idx, val)| (Coordinate{row: row_idx, column: column_idx}, val))
        }).for_each(|(coord, &val)|{
            if val > max_val {
                max_val= val;
                top_moves.clear();
                top_moves.push(coord);
            } else if (val - max_val).abs() <= f32::EPSILON {
                top_moves.push(coord);
            }
        }); 
        
        top_moves

    }

    pub fn get_last_action(&mut self) -> Result<Action> {
        self.action_history.last().ok_or_else(|| anyhow!("No more actions to undo.")).copied()

    }

    pub fn get_last_matching_action(&self, action: Action) -> Result<Action> {
        match action {
            Action::Undo => unreachable!("Undo-s or actions without associated data may never be appended to the action history."),
            Action::Fire(_) | Action::Unfire(_) | Action::Hit(_) | Action::Sink(_) | Action::Unsink(_) => {
                self.action_history.iter().rev().find(|&&act| std::mem::discriminant(&act)==std::mem::discriminant(&action)).ok_or_else(|| anyhow::anyhow!("Could not find last instance of action in history.")).copied()
            }
            
        }
    }

    pub fn debug_print_state(&self) {
        use colored::Colorize;
        println!("Board State:");
        for (row_idx, (heat_line, status_line)) in self.heat_field.get_grid().iter().zip(self.shots.get_grid().iter()).enumerate() {
            for (column_idx,(heat, status)) in heat_line.iter().zip(status_line.iter()).enumerate() {
                match status {
                    ShotStatus::Untested => {
                        let coord= Coordinate{row: row_idx, column: column_idx};
                        if let Some((idx,_))= self.top_moves.iter().enumerate().find(|(idx,&top_move_coord)|coord == top_move_coord){
                            if idx == 0{
                                print!("{}",format!("[{:.2}]", heat).red())
                            } else {
                                print!("{}",format!("[{:.2}]", heat).green())
                            }
                        } else {
                            print!("[{:.2}]", heat)
                        }
                    },
                    ShotStatus::Hit => print!("[####]"),
                    ShotStatus::Miss => print!("[----]"),
                    ShotStatus::Sunk => print!("[||||]"),
                }
            }
            println!()
        }
       
    }
}
