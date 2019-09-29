#![allow(unused_imports)]
#![allow(unused_parens)]
#![feature(clamp)]

extern crate cpurender;

use std::{u8, u32, f32};
use std::thread::sleep;
use std::time::Duration;

use cpurender::*;
use cpurender::re::vek::*;
use cpurender::frag::fragment;

fn main() {
    let x_len = 500;
    let y_len = 500;

    struct State {
        cam_pos: Vec3<f32>,
    }

    let state = State {
        cam_pos: Vec3::new(0., 0., 0.),
    };

    fragment(x_len, y_len, move |xy| {
        // convert xy from [0, (x|y)_len] to [-1, 1]
        let xy_balanced: Vec2<f32> = (
            (
                xy.numcast::<f32>().unwrap()
                    / Vec2::new(x_len, y_len).numcast::<f32>().unwrap()
                    * 2.0
            ) - Vec2::one()
        );

        debug_assert!(xy_balanced
            .map(|c| c >= -1.0 && c <= 1.0)
            .reduce_and());

        let rgb = Rgb::<f32> {
            r: xy_balanced.x,
            g: xy_balanced.y,
            b: xy_balanced.x - xy_balanced.y,
        };

        Rgba::<f32>::from_opaque(rgb)
            .map(|c| c.clamp(0.0, 1.0))
            .map(|c| (c * 0xFF as f32) as u8)
    });
}
