
use crate::{open_window, Paint};

use rayon::prelude::*;
use vek::*;

use std::time::{Instant, Duration};

/// Launch a window with the given function for computing a fragment color.
///
/// This uses rayon for parallelism.
pub fn fragment<F: Fn(Vec2<i32>) -> Rgba<u8> + Send + Sync + 'static>(
    x_size: usize,
    y_size: usize,
    fragment: F,
) {
    // delegate
    fragment_stateful(
        x_size,
        y_size,
        (),
        move |xy, ()| fragment(xy),
    )
}

/// Launch a window with the given function for computing a fragment color. The fragment
/// function will have read-access to some shared state.
///
/// This uses rayon for parallelism.
pub fn fragment_stateful<S, F>(
    x_size: usize,
    y_size: usize,
    state: S,
    fragment: F,
)
    where
        S: Send + Sync + 'static,
        F: Send + Sync + 'static,
        F: Fn(Vec2<i32>, &S) -> Rgba<u8> {

    // open window, drawing thread
    open_window(
        x_size,
        y_size,
        move |queue| {

            let runs = 100;
            let start = Instant::now();

            for i in 0..runs {

                // parallel iter over fragments
                (0..x_size).into_par_iter()
                    .flat_map(|x| (0..y_size).into_par_iter()
                        .map(move |y| (x, y)))
                    //.collect::<Vec<_>>().into_iter() // sequential for debug
                    .for_each(|(x, y)| {

                        // paint
                        let mut color = fragment(
                            Vec2::new(x as i32, y as i32),
                            &state,
                        );

                        if i % 2 == 1 {
                            color.r = 0xFF - color.r;
                            color.g = 0xFF - color.g;
                            color.b = 0xFF - color.b;
                        }

                        queue.push(Paint {
                            x,
                            y,
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: color.a,
                        });
                    });

                dbg!(i);
            }

            let end = Instant::now();

            let avg_time = (end - start) / runs;
            dbg!(avg_time);
        }
    );
}