#![allow(dead_code)]

use crate::types::{Axis, Coordinate};
use anyhow::{Ok, Result};

#[derive(Clone)]
pub struct Field<T> {
    data: Vec<Vec<T>>,
}

impl<T: Copy> Field<T> {
    pub fn new(width: usize, height: usize, value: T) -> Self {
        Self {
            data: vec![vec![value; width]; height],
        }
    }
}
impl<T: Clone + Default> Field<T> {
    pub fn new_default(width: usize, height: usize) -> Self {
        Self {
            data: vec![vec![T::default(); width]; height],
        }
    }
}
impl<T> Field<T> {
    pub fn new_from_grid(data: Vec<Vec<T>>) -> Self {
        Self { data }
    }

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
}

impl<T: Clone> Field<T> {
    pub fn set_value(&mut self, coord: Coordinate, value: &T) -> Result<()> {
        *self
            .data
            .get_mut(coord.row)
            .ok_or_else(|| anyhow::anyhow!("Row index out of bounds."))?
            .get_mut(coord.column)
            .ok_or_else(|| anyhow::anyhow!("Column index out of bounds."))? = value.clone();

        Ok(())
    }

    pub fn set_line(&mut self, axis: Axis, index: usize, line: Vec<T>) -> Result<()> {
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

    pub fn get_line(&self, axis: Axis, index: usize) -> Option<Vec<T>> {
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
}
impl<T> Field<T> {
    pub fn set_grid(&mut self, grid: Vec<Vec<T>>) {
        self.data = grid;
    }
    pub fn get_value(&self, coord: Coordinate) -> Option<&T> {
        self.data.get(coord.row)?.get(coord.column)
    }

    pub fn get_grid(&self) -> &Vec<Vec<T>> {
        &self.data
    }
}

impl<T: Clone + std::ops::Add> Field<T>
where
    <T as std::ops::Add>::Output: Into<T>,
{
    pub fn add_line(&mut self, axis: Axis, index: usize, line: &[T]) -> Result<()> {
        self.set_line(
            axis,
            index,
            self.get_line(axis, index)
                .unwrap()
                .iter()
                .zip(line)
                .map(|(val1, val2)| (val1.clone() + val2.clone()).into())
                .collect(),
        )
    }
}

pub fn print_grid<T: std::fmt::Display>(grid: &[Vec<T>]) {
    for row in grid {
        for val in row {
            print!("[{val:1.3}]");
        }
        println!();
    }
}
