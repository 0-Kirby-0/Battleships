use crate::state;
use crate::types::action::{
    Action,
    Argument::{Known, Unknown},
};
use crate::types::Coordinate;
use anyhow::{Ok, Result};
use std::io::Write;
use strum::IntoEnumIterator;

pub fn main_loop(state: &mut state::State) {
    display_help();

    state.debug_print_state();
    display_recommended_moves(state);
    loop {
        println!("Please enter a command.");
        std::io::stdout().flush().unwrap();

        match play_round(state) {
            std::result::Result::Ok(success_report) => {
                println!("{success_report}");
                state.debug_print_state();
                display_recommended_moves(state);
            }
            Err(err) => println!("{err}"),
        }
    }
}

fn play_round(state: &mut state::State) -> Result<String> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let action = process_input(&input, state)?;

    let mut success_report;
    if std::mem::discriminant(&action) == std::mem::discriminant(&Action::Undo) {
        let last_action = state.get_last_action()?;
        success_report = format!("Successfully undid '{}'.\n", last_action.name());
        success_report += &last_action.opposite().tx_success();
    } else {
        success_report = action.tx_success();
    }

    state.take_action(action)?;

    Ok(success_report)
}

fn process_input(input: &str, state: &state::State) -> Result<Action> {
    let action = parse_input(input)?;

    match action {
        //the action already has its arguments, they needn't be inferred
        Action::Fire(Known(_))
        | Action::Unfire(Known(_))
        | Action::Hit(Known(_))
        | Action::Sink(Known(_))
        | Action::Unsink(Known(_))
        | Action::Undo => Ok(action),

        //"fire" infers you meant to fire at the recommended move
        Action::Fire(Unknown) => Ok(Action::Fire(Known(*state.get_top_moves().first().unwrap()))),
        //"hit" infers your last "fire" was a hit
        Action::Hit(Unknown) => {
            if let Action::Fire(coordinates) =
                state.get_last_matching_action(Action::Fire(Unknown))?
            {
                Ok(Action::Hit(coordinates))
            } else {
                unreachable!(
                    "Action history returned incorrect action when asked for the last 'Fire'"
                )
            }
        }
        //the "un-" actions infer you meant to undo their last opposite.
        Action::Unfire(Unknown) | Action::Unsink(Unknown) => Ok(state
            .get_last_matching_action(action.opposite())?
            .opposite()),

        Action::Sink(Unknown) => unreachable!("Cannot infer length of sunk ship."),
    }
}

fn parse_input(input: &str) -> Result<Action> {
    let words = input.split_whitespace().collect::<Vec<&str>>();
    let arg_count = words.len() - 1;

    let action = parse_action(
        words
            .first()
            .ok_or_else(|| anyhow::anyhow!("Unable to parse command."))?,
    )?;

    if arg_count == 0 {
        if action.can_infer_args() {
            return Ok(action);
        }
        return Err(anyhow::anyhow!(
            "No Arguments provided. Can't infer arguments for this action."
        ));
    } else if arg_count != action.expected_arg_count() {
        return Err(anyhow::anyhow!("Incorrect number of arguments."));
    }

    //This is awkward, but I can't blanket-assign parsed arguments to the variants, as they're of different types.
    //That could be solved with an "assign data" function, but then the compiler could not check for correct usage.
    match action {
        Action::Fire(_) | Action::Unfire(_) | Action::Hit(_) => {
            let coord = Known(Coordinate::from_user(
                parse_number(words[1])?,
                parse_number(words[2])?,
            ));

            match action {
                Action::Fire(_) => Ok(Action::Fire(coord)),
                Action::Unfire(_) => Ok(Action::Unfire(coord)),
                Action::Hit(_) => Ok(Action::Hit(coord)),
                _ => unreachable!(), //the outer match arm restricts this, no need to be careful :)
            }
        }

        Action::Sink(_) | Action::Unsink(_) => {
            let ship_length = Known(parse_number(words[1])?);

            match action {
                Action::Sink(_) => Ok(Action::Sink(ship_length)),
                Action::Unsink(_) => Ok(Action::Unsink(ship_length)),
                _ => unreachable!(), //same as above :)
            }
        }

        Action::Undo => unreachable!(),
    }
}

fn parse_action(maybe_action: &str) -> Result<Action> {
    let maybe_action_name = maybe_action.to_lowercase();

    for action in Action::iter() {
        if action.name() == maybe_action_name {
            return Ok(action);
        }
    }
    Err(anyhow::anyhow!("Invalid command."))
}

fn parse_number(maybe_number: &str) -> Result<usize> {
    maybe_number
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("Unable to read given numeric value."))
}

fn display_recommended_moves(state: &state::State) {
    println!(
        "Recommended move: {}",
        state.get_top_moves().first().unwrap().printable()
    );
    if state.get_top_moves().len() > 1 {
        print!("Alternate moves:");
        for &coord in state.get_top_moves().iter().skip(1) {
            print!("{}", coord.printable());
        }
        println!();
    }
}

fn display_help() {
    println!("Available commands:");
    for action in Action::iter() {
        println!("{}", action.tx_syntax_help());
    }
}
