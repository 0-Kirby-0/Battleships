#![allow(dead_code)]

use std::fmt::Debug;

use crate::types::{Axis, Coordinate};
use anyhow::{Ok, Result};

#[derive(Clone)]
pub struct Field<T> {
    data: Vec<Vec<T>>,
}

impl<T> Field<T> {
    // Constructors //

    pub fn new(width: usize, height: usize, value: T) -> Self
    where
        T: Copy,
    {
        Self {
            data: vec![vec![value; width]; height],
        }
    }

    pub fn new_default(width: usize, height: usize) -> Self
    where
        T: Clone + Default,
    {
        Self {
            data: vec![vec![T::default(); width]; height],
        }
    }

    pub fn new_from_grid(data: Vec<Vec<T>>) -> Self {
        Self { data }
    }

    // Dimensions //

    pub fn width(&self) -> usize {
        self.data.first().unwrap().len()
    }

    pub fn height(&self) -> usize {
        self.data.len()
    }

    pub fn length_of_axis(&self, axis: Axis) -> usize {
        match axis {
            Axis::Row => self.width(),
            Axis::Column => self.height(),
        }
    }

    pub fn number_of_lines_in_axis(&self, axis: Axis) -> usize {
        match axis {
            Axis::Row => self.height(),
            Axis::Column => self.width(),
        }
    }

    // Direct Access //

    pub fn get_grid(&self) -> &Vec<Vec<T>> {
        &self.data
    }

    pub fn set_grid(&mut self, grid: Vec<Vec<T>>) {
        self.data = grid;
    }

    // Line Access //

    pub fn get_line(&self, axis: Axis, index: usize) -> Option<Vec<T>>
    where
        T: Clone,
    {
        if index >= self.number_of_lines_in_axis(axis) {
            return None;
        }
        match axis {
            Axis::Row => self.data.get(index).cloned(),
            Axis::Column => Some(
                self.data
                    .iter()
                    .map(|row| row.get(index).unwrap().clone())
                    .collect::<Vec<_>>(),
            ),
        }
    }

    pub fn get_lines_context(&self, coord: Coordinate) -> (Vec<T>, Vec<T>)
    where
        T: Clone,
    {
        (
            self.get_line(Axis::Row, coord.row).unwrap(),
            self.get_line(Axis::Column, coord.column).unwrap(),
        )
    }

    pub fn set_line(&mut self, axis: Axis, index: usize, line: Vec<T>) -> Result<()>
    where
        T: Clone,
    {
        match axis {
            Axis::Row => {
                *self.data.get_mut(index).unwrap() = line;
            }
            Axis::Column => self
                .data
                .iter_mut()
                .zip(line.iter())
                .for_each(|(row, val)| *row.get_mut(index).unwrap() = val.clone()),
        }
        Ok(())
    }

    pub fn line_iterator(&self, axis: Axis) -> FieldLineIterator<T>
    where
        T: Clone,
    {
        FieldLineIterator {
            field: self,
            axis,
            index: 0,
        }
    }

    // Value Access //
    pub fn get_value(&self, coord: Coordinate) -> Option<&T> {
        self.data.get(coord.row)?.get(coord.column)
    }

    pub fn set_value(&mut self, coord: Coordinate, value: &T) -> Result<()>
    where
        T: Clone,
    {
        *self
            .data
            .get_mut(coord.row)
            .ok_or_else(|| anyhow::anyhow!("Row index out of bounds."))?
            .get_mut(coord.column)
            .ok_or_else(|| anyhow::anyhow!("Column index out of bounds."))? = value.clone();

        Ok(())
    }
    pub fn value_iterator<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        //honestly not all that useful because flattening loses any 2d structure, but might as well
        self.get_grid().iter().flat_map(|line| line.iter())
    }

    // Transformations //

    pub fn transform_all<F, R>(&self, transform: F) -> Field<R>
    where
        F: Fn(&T) -> R,
    {
        //theoretically, one could check if R implements into::<T> and transform in-place, rather than allocating a new field
        //and discarding the old. However, under the current type system that would require duplicate functions, which isn't exactly DRY
        //So: This is good enough :)
        let transformed_grid = self
            .get_grid()
            .iter()
            .map(|line| line.iter().map(&transform).collect())
            .collect();

        Field::new_from_grid(transformed_grid)
    }

    pub fn transform_by_line<F, R>(&self, transform: F) -> (Field<R>, Field<R>)
    where
        T: Clone,
        R: Clone,
        F: Fn(Vec<T>) -> Vec<R>,
    {
        let rows = self
            .line_iterator(Axis::Row)
            .map(&transform)
            .collect::<Vec<_>>();

        let row_field = Field::new_from_grid(rows.clone());
        let mut column_field = Field::new_from_grid(rows);

        for (index, column) in self.line_iterator(Axis::Column).map(&transform).enumerate() {
            column_field.set_line(Axis::Column, index, column).unwrap();
        }

        (row_field, column_field)
    }

    // Merging //

    pub fn merge_fields<F, O, R>(&self, other: &Field<O>, merge: F) -> Field<R>
    where
        F: Fn(&T, &O) -> R,
    {
        let merged = self
            .get_grid()
            .iter()
            .zip(other.get_grid().iter())
            .map(|(self_line, other_line)| {
                self_line
                    .iter()
                    .zip(other_line.iter())
                    .map(|(self_val, other_val)| merge(self_val, other_val))
                    .collect()
            })
            .collect();
        Field::new_from_grid(merged)
    }

    pub fn merge_line<F>(&mut self, axis: Axis, index: usize, line: &[T], transform: F)
    where
        T: Clone,
        F: Fn(&T, &T) -> T,
    {
        let merged_line = self
            .get_line(axis, index)
            .unwrap()
            .iter()
            .zip(line.iter())
            .map(|(self_val, other_val)| transform(self_val, other_val))
            .collect();
        self.set_line(axis, index, merged_line).unwrap()
    }

    // Other //

    pub fn find_all<P>(&self, predicate: P) -> Vec<Coordinate>
    where
        P: Fn(&T) -> bool,
    {
        self.get_grid()
            .iter()
            .enumerate()
            .flat_map(|(row, line)| {
                line.iter()
                    .enumerate()
                    .filter(|(_, val)| predicate(val))
                    .map(move |(column, _)| Coordinate { row, column })
            })
            .collect()
    }

    pub fn debug_print(&self)
    where
        T: Debug,
    {
        println!("Debug Printing:");
        for line in self.get_grid() {
            for val in line {
                print!("[{:?}]", val)
            }
            println!();
        }
    }
}

pub struct FieldLineIterator<'a, T: Clone> {
    field: &'a Field<T>,
    axis: Axis,
    index: usize,
}

impl<'a, T> Iterator for FieldLineIterator<'a, T>
where
    T: Clone,
{
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.field.get_line(self.axis, self.index);
        self.index += 1;
        result
    }
}
