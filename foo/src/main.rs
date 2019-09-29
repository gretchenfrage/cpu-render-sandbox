#![allow(unused_imports)]
#![allow(unused_parens)]
#![feature(clamp)]

extern crate cpurender;

use std::{f32, u32, u8};
use std::mem;
use std::thread::sleep;
use std::time::Duration;

use cpurender::*;
use cpurender::frag::*;
use cpurender::re::vek::*;

/// Rust's f32::sign implementation simply extracts the sign bit, which will
/// consider signum(+0.0)=1.0 and signum(-0.0)=-1.0
fn zero_respecting_signum(n: f32) -> f32 {
    n.signum() * (n != 0.0) as i32 as f32
}

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
                let ingress: Vec3<f32> = (state.cam_pos
                    - voxel_f32
                    // consider the following on all axis:
                    //
                    // if direction is negative, then the collision plane will be at
                    // 0. if the camera position is a multiple or 1, then
                    // `cam_pos - floor(cam_pos)` will also equal zero. that would cause
                    // the ingress point to lie in the plane it is casting to intersect
                    // with, which would ruin the math. so, in that situation, we
                    // need to set the ingress value to 1.
                    + state.cam_pos.map2(
                        direction,
                        |p, d| (p % 1.0 == 0.0 && d < 0.0) as i32 as f32
                    )
                );
                (voxel_f32.numcast::<i32>().unwrap(), ingress)
            };
            debug_assert!(ingress
                .map(|c| c >= 0.0 && c <= 1.0)
                .reduce_and());

            // raytrace loop
            'outer_loop: for _ in 0..10 {
                let planes: Vec3<f32> = direction.ceil();
                debug_assert!(planes
                    .map(|c| c == 0.0 || c == 1.0)
                    .reduce_and());

                let mut distances: [f32; 3] = [f32::NAN; 3];

                for &a in &[0, 1, 2] {
                    distances[a] = (
                        (
                            (
                                planes[a] - ingress[a]
                            ) / (
                                direction[a] + (direction[a] == 0.0) as i32 as f32
                            )
                        ) * (
                            (direction[a] != 0.0) as i32 as f32
                        )
                    );

                    debug_assert!(distances[a] >= 0.0);
                }

                let mut seq_distance: [f32; 3] = [0.0; 3];
                let mut seq_voxel_delta: [Vec3<i32>; 3] = [Vec3::zero(); 3];

                for &(a, b, c) in &[(0, 1, 2), (1, 2, 0), (2, 0, 1)] {
                    let index: usize = (
                        (distances[a] > distances[b]) as usize
                            + (distances[a] > distances[c]) as usize
                    );

                    seq_distance[index] = distances[a];

                    seq_voxel_delta[index][a] += zero_respecting_signum(direction[a]) as i32;

                    debug_assert_eq!(
                        direction[a] == 0.0,
                        distances[a] == 0.0,
                    );
                }
            }

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
        },
    );
}
