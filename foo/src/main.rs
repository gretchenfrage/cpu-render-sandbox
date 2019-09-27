
extern crate cpurender;

use std::{u8, u32};
use std::thread::sleep;
use std::time::Duration;

use cpurender::*;

fn main() {
    let x_len = 500;
    let y_len = 500;

    open_window(x_len, y_len, move |paint| {

        let mut shift: u32 = 0;
        loop {
            for x in 0..x_len {
                for y in 0..y_len {
                    paint.push(Paint {
                        x,
                        y,
                        r: (x.rotate_right(shift) % 0xFF) as u8,
                        g: (y.rotate_right(shift) % 0xFF) as u8,
                        b: ((x.rotate_right(shift) ^ y.rotate_right(shift)) % 0xFF) as u8,
                        a: 0xFF,
                    })
                }
            }

            shift = shift.wrapping_add(1);
            sleep(Duration::from_millis(100));
        }

    });
}
