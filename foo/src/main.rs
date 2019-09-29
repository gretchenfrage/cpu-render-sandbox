#![allow(unused_imports)]
#![allow(unused_parens)]
#![feature(clamp)]

extern crate cpurender;

use std::{u8, u32, f32};
use std::thread::sleep;
use std::time::Duration;
use std::mem;

use cpurender::*;
use cpurender::re::vek::*;
use cpurender::frag::*;

fn main() {
    let x_len = 500;
    let y_len = 500;

    struct State {
        cam_pos: Vec3<f32>,
        cam_dir: Vec3<f32>,
        cam_fov: f32,
    }

    let state = State {
        cam_pos: Vec3::new(-5.0, -5.0, -5.0),
        cam_dir: Vec3::new(0.5, -0.5, 1.0).normalized(),
        //cam_dir: Vec3::forward_lh(),
        cam_fov: 90.0_f32.to_radians(),
    };

    fragment_stateful(
        x_len,
        y_len,
        state,
        move |xy, state| {
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

            // calculate ray direction for this fragment
            let direction: Vec3<f32> = {
                // calculate view-space angle of the fragment
                let view_space_angle: Vec2<f32> = xy_balanced
                    .map(|n| n.sin() * state.cam_fov);

                // apply that rotation to the cam dir
                Quaternion::rotation_y(view_space_angle.x)
                    * Quaternion::rotation_x(-view_space_angle.y)
                    * state.cam_dir
            };
            debug_assert!((direction.magnitude() - 1.0).abs() < 0.00001);

            // calculate voxel coordinate and ingress
            let (voxel, ingress) = {
                let voxel_f32: Vec3<f32> = state.cam_pos.floor();
                let ingress: Vec3<f32> = state.cam_pos - voxel_f32;
                (voxel_f32.numcast::<i32>().unwrap(), ingress)
            };
            debug_assert!(ingress
                .map(|c| c >= 0.0 && c <= 1.0)
                .reduce_and());

            // debug color
            let debug_val: Vec3<f32> = direction;
            let rgb = Rgb::<f32> {
                r: debug_val.x,
                g: debug_val.y,
                b: debug_val.z,
            };

            Rgba::<f32>::from_opaque(rgb)
                .map(|c| c.clamp(0.0, 1.0))
                .map(|c| (c * 0xFF as f32) as u8)
        }
    );
}
