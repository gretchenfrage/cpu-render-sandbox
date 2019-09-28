
use crate::{open_window, Paint};

use rayon::prelude::*;
use vek::*;

/// Launch a window with the given function for computing a fragment color.
///
/// This uses rayon for parallelism.
pub fn fragment<F: Fn(Vec2<i32>) -> Rgba<u8> + Send + Sync + 'static>(
    x_size: usize,
    y_size: usize,
    fragment: F,
) {
    // open window, drawing thread
    open_window(
        x_size,
        y_size,
        move |queue| {

            // parallel iter over fragments
            (0..x_size).into_par_iter()
                .flat_map(|x| (0..y_size).into_par_iter()
                    .map(move |y| (x, y)))
                .for_each(move |(x, y)| {

                    // paint
                    let color = fragment(Vec2::new(x as i32, y as i32));
                    queue.push(Paint {
                        x,
                        y,
                        r: color.r,
                        g: color.g,
                        b: color.b,
                        a: color.a,
                    });

                });
        }
    );
}