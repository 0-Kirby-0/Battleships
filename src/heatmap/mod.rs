mod base;
mod hit;

use crate::types::field::Field;
use crate::types::ShotStatus;

pub fn gen_heat_field(shots: &Field<ShotStatus>, ship_lengths: &[usize]) -> Field<f32> {
    let bool_shots = shots.transform_all(|&status| status.can_contain_ship());
    let hits = shots.find_all(|status| {
        std::mem::discriminant(status) == std::mem::discriminant(&ShotStatus::Hit)
    });

    let base_heat = base::gen_heat(&bool_shots, ship_lengths);
    let hit_heat = hit::gen_heat(&bool_shots, &hits, ship_lengths);

    let combined_heat = reduce_heat_fields([base_heat, hit_heat].into_iter());

    mask_heat_field(&combined_heat, shots)
}

fn reduce_heat_fields(fields: impl Iterator<Item = Field<f32>>) -> Field<f32> {
    fields
        .map(|field| field.transform_all(|val| 1. - val))
        .reduce(|acc, e| acc.merge_fields(&e, |acc_val, e_val| acc_val * e_val))
        .unwrap()
        .transform_all(|val| 1. - val)
}

fn mask_heat_field(heat: &Field<f32>, shots: &Field<ShotStatus>) -> Field<f32> {
    heat.merge_fields(shots, |&heat_val, status| match status {
        ShotStatus::Hit | ShotStatus::Miss | ShotStatus::Sunk => 0.,
        ShotStatus::Untested => heat_val,
    })
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
