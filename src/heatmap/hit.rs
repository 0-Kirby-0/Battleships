use super::*;
use crate::types::field::Field;
use crate::types::{Axis, Coordinate};

pub(super) fn gen_heat(
    bool_shots: &Field<bool>,
    hits: &[Coordinate],
    ship_lengths: &[usize],
) -> Field<f32> {
    if hits.is_empty() {
        //there are no hits to work on, just return all 0.s
        return Field::new_default(bool_shots.width(), bool_shots.height());
    }

    let heat_fields = ship_lengths
        .iter()
        .map(|&ship_length| gen_ship_heat(bool_shots, hits, ship_length));

    reduce_heat_fields(heat_fields)
}

fn gen_ship_heat(bool_shots: &Field<bool>, hits: &[Coordinate], ship_length: usize) -> Field<f32> {
    let mut heat = Field::new_default(bool_shots.width(), bool_shots.height());
    for &hit in hits {
        let (row, column) = bool_shots.get_lines_context(hit);

        let (row_ship_count_line, row_ship_count) =
            gen_line(&mask_around_hit(&row, hit.column, ship_length), ship_length);
        let (column_ship_count_line, column_ship_count) =
            gen_line(&mask_around_hit(&column, hit.row, ship_length), ship_length);

        let total_ship_count = row_ship_count + column_ship_count;

        if total_ship_count != 0 {
            let row_heat = ship_counts_to_heat(&row_ship_count_line, total_ship_count);
            let column_heat = ship_counts_to_heat(&column_ship_count_line, total_ship_count);

            heat.merge_line(Axis::Row, hit.row, &row_heat, |acc, e| acc + e);
            heat.merge_line(Axis::Column, hit.column, &column_heat, |acc, e| acc + e);
        }
        //should there be no ships, avoid the div/0 but otherwise don't worry about it
    }

    heat.transform_all(|val| val / hits.len() as f32)
}

fn ship_counts_to_heat(ship_counts: &[usize], total_ship_count: usize) -> Vec<f32> {
    let total_ship_count = total_ship_count as f32;
    ship_counts
        .iter()
        .map(|&ship_count| ship_count as f32 / total_ship_count)
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
