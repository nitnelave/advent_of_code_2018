use std::ops::Index;
use std::ops::IndexMut;

/// 2-D vector, stored in a 1-D Vec.
#[derive(Debug)]
pub struct Vec2D<T> {
    height: usize,
    width: usize,
    vec: Vec<T>,
}

/// Index access with a tuple.
impl<T> Index<(usize, usize)> for Vec2D<T> {
    type Output = T;
    fn index(&self, i: (usize, usize)) -> &T {
        &self.vec[i.0 * self.width + i.1]
    }
}

/// Mutable index access with a tuple.
impl<T> IndexMut<(usize, usize)> for Vec2D<T> {
    //type Output = T;
    fn index_mut(&mut self, i: (usize, usize)) -> &mut T {
        &mut self.vec[i.0 * self.width + i.1]
    }
}

/// Build a 2-D vector by filling it with the values returned by the generator
/// (it gets called for each cell).
impl<T> Vec2D<T> {
    pub fn from_fn(height: usize, width: usize, generator: &Fn() -> T) -> Self {
        let vec = (0..(height * width))
            .map(|_| generator())
            .collect::<Vec<_>>();
        Self { height, width, vec }
    }
}
