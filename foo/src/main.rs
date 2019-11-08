#![allow(unused_imports)]
#![allow(unused_parens)]
#![feature(clamp)]

extern crate cpurender;

use std::{f32, f64, u32, u8};
use std::mem;
use std::thread::sleep;
use std::time::Duration;

use cpurender::*;
use cpurender::frag::*;
use cpurender::re::vek::*;

// trick to allow us to easily toggle fp precision
#[allow(non_camel_case_types)]
type float = f32;
const NAN: float = f32::NAN;

/// Rust's float::sign implementation simply extracts the sign bit, which will
/// consider signum(+0.0)=1.0 and signum(-0.0)=-1.0
fn zero_respecting_signum(n: float) -> float {
    n.signum() * (n != 0.0) as i32 as float
}

/// Float equality within epsilon.
fn approx_eq(a: float, b: float) -> bool {
    (a - b).abs() < 0.00001
}

/// Power of 2.
fn square(x: float) -> float {
    x * x
}

/// UB if None.
#[cfg(debug_assertions)]
unsafe fn unwrap_unchecked<T>(opt: Option<T>) -> T {
    opt.unwrap()
}

/// UB if None.
#[cfg(not(debug_assertions))]
unsafe fn unwrap_unchecked<T>(opt: Option<T>) -> T {
    match opt {
        Some(t) => t,
        None => std::hint::unreachable_unchecked(),
    }
}

macro_rules! unroll {
    {
        for $var:pat in [
            $(
                $elem:expr
            ),*
            $(,)?
        ] $body:block
    } => {{
        $(
        {
            let $var = $elem;
            $body;
        }
        )*
    }};
}

fn main() {
    let x_len = 1000;
    let y_len = 1000;

    struct State {
        cam_pos: Vec3<float>,
        cam_dir: Vec3<float>,
        cam_fov: float,
    }

    let state = State {
        cam_dir: Vec3::new(1.0, -1.0, 2.0).normalized(),
        cam_pos: Vec3::new(-5.0, 5.0, -5.0),
        cam_fov: (100.0 as float).to_radians(),
    };

    fragment_stateful(
        x_len,
        y_len,
        state,
        move |xy, state| {
            // convert xy from [0, (x|y)_len] to [-1, 1]
            let xy_balanced: Vec2<float> = (
                (
                    unsafe { unwrap_unchecked(xy.numcast::<float>()) }
                        / unsafe { unwrap_unchecked(Vec2::new(x_len, y_len).numcast::<float>()) }
                        * 2.0
                ) - Vec2::one()
            );
            debug_assert!(xy_balanced
                .map(|c| c >= -1.0 && c <= 1.0)
                .reduce_and());

            // calculate ray direction for this fragment
            let mut direction: Vec3<float> = {
                let fov_coef = (state.cam_fov / 2.0).tan() / (45.0 as float).to_radians().tan();
                let perspective: Quaternion<float> = Quaternion::rotation_from_to_3d(
                    Vec3::new(0.0, 0.0, 1.0),
                    Vec3::new(
                        xy_balanced.x * fov_coef,
                        xy_balanced.y * fov_coef,
                        1.0,
                    ).normalized(),
                );

                let view: Quaternion<float> = {
                    // see: https://math.stackexchange.com/questions/470112/calculate-camera-pitch-yaw-to-face-point
                    let yaw: float = state.cam_dir.z.atan2(state.cam_dir.x);
                    let pitch: float = state.cam_dir.y.atan2(
                        (square(state.cam_dir.x) + square(state.cam_dir.z)).sqrt()
                    );
                    Quaternion::rotation_y(yaw) * Quaternion::rotation_x(-pitch)
                };

                view * perspective * Vec3::forward_lh()
            };
            debug_assert!(approx_eq(direction.magnitude(), 1.0));

            // calculate voxel coordinate and ingress
            let (mut voxel, mut ingress) = {
                let voxel_float: Vec3<float> = state.cam_pos.floor();
                let ingress: Vec3<float> = (
                    (
                        state.cam_pos
                    ) - (
                        voxel_float
                    ) + (
                        // consider the following on all axis:
                        //
                        // if direction is negative, then the collision plane will be at
                        // 0. if the camera position is a multiple or 1, then
                        // `cam_pos - floor(cam_pos)` will also equal zero. that would cause
                        // the ingress point to lie in the plane it is casting to intersect
                        // with, which would ruin the math. so, in that situation, we
                        // need to set the ingress value to 1.
                        state.cam_pos.map2(
                            direction,
                            |p, d| (p % 1.0 == 0.0 && d < 0.0) as i32 as float
                        )
                    )
                );
                (unsafe { unwrap_unchecked(voxel_float.numcast::<i32>()) }, ingress)
            };
            debug_assert!(ingress
                .map(|c| c >= 0.0 && c <= 1.0)
                .reduce_and());

            let mut hits = 0;

            // raytrace loop
            'outer_loop: for iteration in 0..50 {

                let planes: Vec3<float> = direction.ceil();
                debug_assert!(planes
                    .map(|c| c == 0.0 || c == 1.0)
                    .reduce_and());

                let mut distances: [float; 3] = [NAN; 3];

                unroll! { for a in [0, 1, 2] {
                    debug_assert!(planes[a] != ingress[a] || direction[a] == 0.0);

                    distances[a] = (
                        (
                            (
                                planes[a] - ingress[a]
                            ) / (
                                direction[a] + (direction[a] == 0.0) as i32 as float
                            )
                        ) * (
                            (direction[a] != 0.0) as i32 as float
                        )
                    );

                    debug_assert!(distances[a] >= 0.0);
                }};

                let mut seq_distance: [float; 3] = [0.0; 3];
                let mut seq_voxel_delta: [Vec3<i32>; 3] = [Vec3::zero(); 3];

                // sort seq, with equal-distance merging
                unroll!{ for (a, b, c) in [(0, 1, 2), (1, 2, 0), (2, 0, 1)] {
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
                }};

                let mut hit_index: usize = 2;

                unroll!{ for a in [1, 0] {
                    let b: usize = a + 1;

                    let dedup_epsilon: float = 0.00001;

                    let a_present: bool = seq_distance[a] != 0.0;
                    let should_merge: bool = (
                        a_present
                            & (
                            (seq_distance[a] - seq_distance[b]).abs() < dedup_epsilon
                        )
                    );

                    hit_index -= a_present as usize;
                    seq_voxel_delta[a] += (
                        seq_voxel_delta[b] * (
                            should_merge as usize as i32
                        )
                    );
                    seq_distance[a] = seq_distance[(
                        a + should_merge as usize
                    )];
                }};

                debug_assert!(seq_distance[hit_index] != 0.0);

                // the following code becomes much simpler, now that we have eliminated the
                // inner loop.
                ingress = (
                    (
                        ingress + (direction * seq_distance[hit_index])
                    ) - (
                        unsafe { unwrap_unchecked(seq_voxel_delta[hit_index].numcast::<float>()) }
                    )
                );

                voxel += seq_voxel_delta[hit_index];

                debug_assert!(ingress
                    .map(|c| approx_eq(c, 0.0) || approx_eq(c, 1.0))
                    .reduce_or());
                debug_assert!(ingress
                    .map(|c| c >= -0.00001 && c <= 1.00001)
                    .reduce_and());

                // collision
                fn in_grid(v: Vec3<i32>) -> bool {
                    v.partial_cmpge(&Vec3::new(0, 0, 0)).reduce_and()
                        && v.partial_cmplt(&Vec3::new(5, 5, 5)).reduce_and()
                }

                if in_grid(voxel) {

                    // this could be better optimized, but it's to only count external
                    // edges once
                    let incr = match in_grid(voxel - seq_voxel_delta[hit_index]) {
                        true => {
                            ingress
                                .map(|c| match approx_eq(c, 0.0) || approx_eq(c, 1.0) {
                                    true => 1,
                                    false => 0,
                                })
                                .reduce(|a, b| a + b)
                        },
                        false => 1,
                    };

                    hits += incr;

                    for _ in 0..incr {
                        //direction = Quaternion::rotation_x((-5.0 as float).to_radians()) * direction;
                    }

                }
            }

            // compute color
            let rgb: Rgb<float> = Rgb {
                r: hits as float / 15.0,
                g: hits as float / 10.0,
                b: hits as float / 5.0,
            };

            Rgba::<float>::from_opaque(rgb)
                .map(|c| c.clamp(0.0, 1.0))
                .map(|c| (c * 0xFF as float) as u8)
        },
    );
}
