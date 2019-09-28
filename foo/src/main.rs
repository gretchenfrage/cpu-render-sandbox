#![allow(unused_imports)]

extern crate cpurender;

use std::{u8, u32};
use std::thread::sleep;
use std::time::Duration;

use cpurender::*;
use cpurender::re::vek::*;
use cpurender::fragment::fragment;

fn main() {
    let x_len = 500;
    let y_len = 500;

    fragment(x_len, y_len, |xy| {
        let Vec2 { x, y } = xy;

        Rgba {
            r: (x % 0xFF) as u8,
            g: (y % 0xFF) as u8,
            b: ((x ^ y) % 0xFF) as u8,
            a: 0xFF,
        }
    });
}
