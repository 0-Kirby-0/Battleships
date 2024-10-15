use crate::field::Field;
use crate::types::{Axis, Coordinate, ShotStatus};

pub fn gen_heat_field(shots: &Field<ShotStatus>, ships: &[usize]) -> Field<f32> {
    let basic_heat_grid = gen_basic_heat_grid(shots, ships);
    let hits_heat_grid = gen_hits_heat_grid(shots, ships);

    let combined_heat_grid = reduce_heat_grids([basic_heat_grid, hits_heat_grid].into_iter());

    let masked_heat_grid = mask_heat_grid(shots, combined_heat_grid);

    Field::new_from_grid(masked_heat_grid)
}

/*
** The following two functions have a failure mode which means one of two things, and we can't know which:
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
fn gen_basic_heat_grid(shots: &Field<ShotStatus>, ships: &[usize]) -> Vec<Vec<f32>> {
    let heat_grids = ships.iter().map(|&ship_length| {
        let (ship_counts, total_ship_count) = gen_basic_ship_counts(shots, ship_length);

        if total_ship_count == 0 {
            println!("!!WARNING!!\nShip of length {ship_length} couldn't be placed a single time.\nSomething is wrong. Continuing regardless.\n################");
            //Avoid the div/0
            vec![vec![0.; shots.width()]; shots.height()]
        } else {
            ship_count_field_to_heat_grid(&ship_counts, total_ship_count)
        }
    });

    reduce_heat_grids(heat_grids)
}

fn gen_hits_heat_grid(shots: &Field<ShotStatus>, ships: &[usize]) -> Vec<Vec<f32>> {
    if !shots
        .get_grid()
        .iter()
        .flatten()
        .any(|status| std::mem::discriminant(status) == std::mem::discriminant(&ShotStatus::Hit))
    {
        //there's no hits to work on, don't bother
        return vec![vec![0.; shots.width()]; shots.height()];
    }

    let mut overall_total_ship_count = 0;
    let heat_grids = ships.iter().map(|&ship_length| {
        let (ship_counts, total_ship_count) = gen_hits_ship_counts(shots, ship_length);
        overall_total_ship_count += total_ship_count;
        if total_ship_count == 0 {
            //This just means the hit(s) werent on this ship, but some other ship.
            //Avoid the div/0
            vec![vec![0.; shots.width()]; shots.height()]
        } else {
            ship_count_field_to_heat_grid(&ship_counts, total_ship_count)
        }
    });

    let heat_grid = reduce_heat_grids(heat_grids);

    if overall_total_ship_count == 0 {
        println!("!!WARNING!!\nNo ship fits the given hit(s).\nSomething is wrong. Continuing regardless.\n################")
    }

    heat_grid
}

fn reduce_heat_grids(mut grids: impl Iterator<Item = Vec<Vec<f32>>>) -> Vec<Vec<f32>> {
    // basically a manually implemented .reduce(), but with correctly handling heat value
    // combinatory multiplication, and without allocating too much

    //Allocating one grid to multiply values into.
    //Inverting heat so combinatory multiplication works correctly
    let acc: Vec<Vec<f32>> = grids
        .next()
        .unwrap()
        .iter()
        .map(|line| line.iter().map(|&val| 1. - val).collect())
        .collect();

    //folding heat grids into the accumulator, after inverting them too
    grids
        .fold(acc, |mut acc_grid, grid| {
            acc_grid
                .iter_mut()
                .zip(grid.iter())
                .for_each(|(acc_line, line)| {
                    acc_line
                        .iter_mut()
                        .zip(line.iter())
                        .for_each(|(acc_val, val)| *acc_val *= 1. - val);
                });
            acc_grid
        })
        //now that heat values are combined, invert them back
        .iter()
        .map(|line| line.iter().map(|val| 1. - val).collect())
        .collect()
}

fn mask_heat_grid(shots: &Field<ShotStatus>, mut grid: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    grid.iter_mut()
        .zip(shots.get_grid().iter())
        .for_each(|(heat_line, status_line)| {
            heat_line
                .iter_mut()
                .zip(status_line.iter())
                .for_each(|(heat, status)| match status {
                    ShotStatus::Hit | ShotStatus::Miss | ShotStatus::Sunk => *heat = 0.,
                    ShotStatus::Untested => (),
                });
        });

    grid
}

fn ship_count_field_to_heat_grid(
    ship_counts: &Field<usize>,
    total_ship_count: usize,
) -> Vec<Vec<f32>> {
    let total_ship_count = total_ship_count as f32;

    ship_counts
        .get_grid()
        .iter()
        .map(|line| {
            line.iter()
                .map(|&val| val as f32 / total_ship_count)
                .collect()
        })
        .collect()
}

fn gen_basic_ship_counts(shots: &Field<ShotStatus>, ship_length: usize) -> (Field<usize>, usize) {
    let mut ship_counts = Field::<usize>::new_default(shots.width(), shots.height());
    let mut total_ship_count = 0;

    for axis in [Axis::Row, Axis::Column] {
        for index in 0..shots.number_of_lines_in_axis(axis) {
            let (counts_line, ship_count) =
                gen_line(&get_bool_shots_line(shots, axis, index), ship_length);

            ship_counts
                .add_line(axis, index, &counts_line)
                .expect("Unable to add ship count line to ship count field.");

            total_ship_count += ship_count;
        }
    }

    (ship_counts, total_ship_count)
}

fn gen_hits_ship_counts(shots: &Field<ShotStatus>, ship_length: usize) -> (Field<usize>, usize) {
    let hit_list = get_hit_coordinates(shots);
    let mut ship_counts = Field::<usize>::new_default(shots.width(), shots.height());
    let mut total_ship_count = 0;
    for hit in hit_list.iter() {
        for axis in [Axis::Row, Axis::Column] {
            let masked_bool_shots_line = mask_around_hit(
                &get_bool_shots_line(shots, axis, hit.get_axis_index(axis)),
                hit.get_axis_index(axis.opposite()),
                ship_length,
            );

            let (counts_line, ship_count) = gen_line(&masked_bool_shots_line, ship_length);
            ship_counts
                .add_line(axis, hit.get_axis_index(axis), &counts_line)
                .expect("Unable to add ship count line to ship count field.");

            total_ship_count += ship_count;
        }
    }
    (ship_counts, total_ship_count)
}

fn get_bool_shots_line(shots: &Field<ShotStatus>, axis: Axis, index: usize) -> Vec<bool> {
    shots
        .get_line(axis, index)
        .expect("Couldn't retreieve line from shots field.")
        .iter()
        .map(|s| s.can_contain_ship())
        .collect()
}

fn gen_line(shots: &[bool], ship_length: usize) -> (Vec<usize>, usize) {
    let mut result = Vec::with_capacity(shots.len());
    let mut ship_count = 0;

    for (streak_length, streak_type) in get_streaks(shots) {
        match streak_type {
            false => result.extend(vec![0; streak_length]),
            true => {
                let (section, count) = gen_free_space(streak_length, ship_length);
                result.extend(section);
                ship_count += count;
            }
        }
    }

    (result, ship_count)
}

fn get_streaks<T: Clone + Eq>(line: &[T]) -> Vec<(usize, T)> {
    let mut out = vec![(1, line.first().unwrap().clone())];

    for pair in line.windows(2) {
        if pair[0] == pair[1] {
            out.last_mut().unwrap().0 += 1;
        } else {
            out.push((1, pair[1].clone()));
        }
    }

    out
}

fn gen_free_space(space: usize, ship_length: usize) -> (Vec<usize>, usize) {
    if ship_length > space {
        return (vec![0; space], 0);
    }
    let ship_count = space - ship_length + 1;

    (
        (0..space)
            .map(|i| {
                let left_dist = i + 1;
                let right_dist = space - i;

                left_dist.min(right_dist).min(ship_length).min(ship_count)
            })
            .collect(),
        ship_count,
    )
}

fn get_hit_coordinates(shots: &Field<ShotStatus>) -> Vec<Coordinate> {
    shots
        .get_grid()
        .iter()
        .enumerate()
        .flat_map(|(row_idx, line)| {
            line.iter()
                .enumerate()
                .filter(|(_, &status)| {
                    std::mem::discriminant(&status) == std::mem::discriminant(&ShotStatus::Hit)
                })
                .map(move |(column_idx, _)| Coordinate {
                    row: row_idx,
                    column: column_idx,
                })
        })
        .collect()
}

fn mask_around_hit(shots: &[bool], hit_location: usize, ship_length: usize) -> Vec<bool> {
    shots
        .iter()
        .enumerate()
        .map(|(index, value)| {
            if (index + ship_length <= hit_location) | (hit_location + ship_length <= index) {
                false
            } else {
                *value
            }
        })
        .collect()
}
