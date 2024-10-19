use super::*;
use field::helpers::Axis;
use field::Field;

/*
** The following function has a failure mode which means one of two things, and we can't know which:
**
** 1: The user made a small mistake, mistyped or misread, and entered bad data.
** In this case, we want to let them know, but keep going, so they can fix their mistake instead of having to start over.
**
** 2: If we're in automated play, the system gave us bad data, and things are desperately wrong.
** If that's the case, we should panic, crash and burn, but since we can't be sure if we're in automated mode, we have to be conservative.
**
** We're also pretty far down, so modifying things to instead pass an Err() up the chain and handle it *somewhere else* would require
** extensive changes and an incredible amount of defensive coding.
** What's worse: We have no way of knowing when the error actually occurred and what data is bad,
** meaning there is no actually good way to handle that Err() (such as an undo) even all the way at the top,
** all we could do is panic in automated play.
**
** So, my solution: Throw up a warning from right here, and just keep truckin' :)
*/

pub(super) fn gen_heat(bool_shots: &Field<bool>, ship_lengths: &[usize]) -> Field<f32> {
    let heat_fields = ship_lengths.iter().map(|&ship_length| {
        let (ship_counts, total_ship_count) = gen_ship_counts(bool_shots, ship_length);

        if total_ship_count == 0 {
            println!("!!WARNING!!\nShip of length {ship_length} couldn't be placed a single time.");
            println!("Something is wrong. Continuing regardless.\n###############################");
            //Avoid the div/0
            Field::new_default(bool_shots.width(), bool_shots.height())
        } else {
            ship_counts_to_heat(&ship_counts, total_ship_count)
        }
    });

    reduce_heat_fields(heat_fields)
}

fn ship_counts_to_heat(ship_counts: &Field<usize>, total_ship_count: usize) -> Field<f32> {
    let total_ship_count = total_ship_count as f32;
    ship_counts.transform_all(|&ship_count| ship_count as f32 / total_ship_count)
}

fn gen_ship_counts(bool_shots: &Field<bool>, ship_length: usize) -> (Field<usize>, usize) {
    //because the borrow-checker doesn't like variables being captured in closures,
    //this workaround is necessary to calm it down.
    let total_ship_count = std::cell::RefCell::new(0);

    let ship_counter = |line: Vec<bool>| {
        let (ship_count_line, ship_count) = gen_line(&line, ship_length);
        *total_ship_count.borrow_mut() += ship_count;
        ship_count_line
    };

    let rows = bool_shots
        .transform_by_line(Axis::Row, ship_counter)
        .unwrap();
    let columns = bool_shots
        .transform_by_line(Axis::Column, ship_counter)
        .unwrap();

    let ship_counts = rows.merge_field(&columns, |row_val, column_val| row_val + column_val);

    //dropping the refcell and returning its content
    let total_ship_count = *total_ship_count.borrow();

    (ship_counts, total_ship_count)
}
