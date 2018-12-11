extern crate itertools;
extern crate ndarray;

use ndarray::Array2;

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_sign_loss)]
fn safe_cast(n: usize) -> Option<i32> {
    if n < i32::max_value() as usize {
        Some(n as i32)
    } else {
        None
    }
}
fn compute_power_level(index: (usize, usize), serial_number: i32) -> i32 {
    let rack_id = safe_cast(index.0).unwrap() + 10;
    let power_level = (rack_id * safe_cast(index.1).unwrap() + serial_number) * rack_id;
    (power_level / 100) % 10 - 5
}

pub fn power_levels(grid_size: usize, serial_number: i32) -> Array2<i32> {
    let mut levels = Array2::<i32>::zeros((grid_size, grid_size));
    levels.indexed_iter_mut().for_each(|(index, level)| {
        *level = compute_power_level(index, serial_number)
    });
    levels
}

pub fn sum_window(grid: &Array2<i32>, window: usize) -> Array2<i32> {
    let new_size = grid.shape()[0] - window + 1;
    let mut sum_x = Array2::<i32>::zeros((new_size, grid.shape()[0]));
    for j in 0..grid.shape()[0] {
        let mut sum = 0;
        for i in 0..window - 1 {
            sum += grid[(i, j)];
        }
        for i in 0..new_size {
            sum += grid[(i + window - 1, j)];
            sum_x[(i, j)] = sum;
            sum -= grid[(i, j)];
        }
    }
    let mut result = Array2::<i32>::zeros((new_size, new_size));
    for i in 0..new_size {
        let mut sum = 0;
        for j in 0..window - 1 {
            sum += sum_x[(i, j)];
        }
        for j in 0..new_size {
            sum += sum_x[(i, j + window - 1)];
            result[(i, j)] = sum;
            sum -= sum_x[(i, j)];
        }
    }
    result
}

pub struct MaxResult {
    pub coords: (usize, usize),
    pub value: i32,
}

pub fn find_max(grid: &Array2<i32>) -> MaxResult {
    grid.indexed_iter()
        .max_by_key(|(_, v)| *v)
        .map(|(coords, &value)| MaxResult { coords, value })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_power_level() {
        assert_eq!(compute_power_level((3, 5), 8), 4);
        assert_eq!(compute_power_level((122, 79), 57), -5);
        assert_eq!(compute_power_level((217, 196), 39), 0);
        assert_eq!(compute_power_level((101, 153), 71), 4);
    }

    #[test]
    fn test_sum_window() {
        let array = Array2::from_shape_vec(
            (3, 3),
            itertools::iterate(0, &|i: &i32| *i + 1)
                .take(9)
                .collect::<Vec<_>>(),
        ).unwrap();
        assert_eq!(
            sum_window(array, 2),
            Array2::from_shape_vec((2, 2), vec![8, 12, 20, 24]).unwrap()
        );
    }
}
