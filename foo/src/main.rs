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

fn approx_eq(a: float, b: float) -> bool {
    (a - b).abs() < 0.00001
}

fn main() {
    let x_len = 500;
    let y_len = 500;

    struct State {
        cam_pos: Vec3<float>,
        cam_dir: Vec3<float>,
        cam_fov: float,
    }

    let state = State {
        cam_pos: Vec3::new(-5.0, 5.0, -5.0),
        cam_dir: Vec3::new(0.5, -0.5, 1.0).normalized(),
        //cam_dir: Vec3::forward_lh(),
        //cam_fov: 90.0_float.to_radians(),
        cam_fov: (90.0 as float).to_radians(),
    };

    fragment_stateful(
        x_len,
        y_len,
        state,
        move |xy, state| {
            // convert xy from [0, (x|y)_len] to [-1, 1]
            let xy_balanced: Vec2<float> = (
                (
                    xy.numcast::<float>().unwrap()
                        / Vec2::new(x_len, y_len).numcast::<float>().unwrap()
                        * 2.0
                ) - Vec2::one()
            );
            debug_assert!(xy_balanced
                .map(|c| c >= -1.0 && c <= 1.0)
                .reduce_and());

            // calculate ray direction for this fragment
            let mut direction: Vec3<float> = {
                // calculate view-space angle of the fragment
                let view_space_angle: Vec2<float> = xy_balanced
                    .map(|n| n.sin() * state.cam_fov / 2.0);

                // apply that rotation to the cam dir
                Quaternion::rotation_y(view_space_angle.x)
                    * Quaternion::rotation_x(-view_space_angle.y)
                    * state.cam_dir
            };
            debug_assert!(approx_eq(direction.magnitude(), 1.0));

            // calculate voxel coordinate and ingress
            let (mut voxel, mut ingress) = {
                let voxel_float: Vec3<float> = state.cam_pos.floor();
                let ingress: Vec3<float> = (state.cam_pos
                    - voxel_float
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
                        |p, d| (p % 1.0 == 0.0 && d < 0.0) as i32 as float
                    )
                );
                (voxel_float.numcast::<i32>().unwrap(), ingress)
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

                for &a in &[0, 1, 2] {
                    debug_assert!(planes[a] != ingress[a]);

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
                }

                let mut seq_distance: [float; 3] = [0.0; 3];
                let mut seq_voxel_delta: [Vec3<i32>; 3] = [Vec3::zero(); 3];

                for &(a, b, c) in &[(0, 1, 2), (1, 2, 0), (2, 0, 1)] {

                    // this index calculation works as a fix
                    // by merging steps in the sequence if they're close
                    let index: usize = {
                        // one step may be greater than another, but if its advantage is
                        // no greater than epsilon, it will be merged with the lesser
                        let epsilon: f32 = 0.000004;
                        (
                            distances[a] - distances[b] >= epsilon
                        ) as usize + (
                            distances[a] - distances[c] >= epsilon
                        ) as usize - (
                            // however, there is an edge case, in which the points
                            // form the ordered sequence [a, b, c],
                            // where:
                            //     (b - a) <= epsilon
                            // and:
                            //     (c - b) <= epsilon
                            // however:
                            //     (c - a) > epsilon
                            //
                            // in this case, we subtract 1, bringing the index
                            // from 1 to 0
                            (
                                distances[a] > distances[b]
                                    && distances[b] > distances[c]
                                    && distances[a] - distances[c] <= epsilon * 2.0
                            ) || (
                                distances[a] > distances[c]
                                    && distances[c] > distances[b]
                                    && distances[a] - distances[b] <= epsilon * 2.0
                            )
                        ) as usize
                    };

                    seq_distance[index] = distances[a];

                    seq_voxel_delta[index][a] += zero_respecting_signum(direction[a]) as i32;

                    debug_assert_eq!(
                        direction[a] == 0.0,
                        distances[a] == 0.0,
                    );
                }

                // i have just discovered an oversight of the original algorithm, in which the
                // sequence of voxels that a ray collides with may have the same voxel delta
                // more than once in a row (this is actually very common).
                //
                // we are modifying the algorithm such that we only hit the first element in the
                // sequence before resuming the loop. the other option would be to consider
                // elements in the sequence non-valid if their `seq_ingress` value, as computed
                // by the former version of the algorithm, had any component outside of the
                // valid 0 <= n <= 1 range. however, that would drastically decrease the
                // branch-predictability of the step's is-present check. conversely, by intersecting
                // with the first valid element, we eliminate the branch altogether,
                // which is excellent.

                // only the 0th element of the sequence can be absent, since absence is
                // represented with a distance of 0, and the sequence is sorted/deduped by
                // distance. therefore, if the 0th element of the sequence is absent, the
                // first present index is 1, otherwise, that index is 0.
                let hit_index: usize = (distances[0] == 0.0) as usize;

                debug_assert!(distances[hit_index] != 0.0);

                // the following code becomes much simpler, now that we have eliminated the
                // inner loop.
                ingress = (
                    (
                        ingress + (direction * seq_distance[hit_index])
                    ) - (
                        seq_voxel_delta[hit_index].numcast::<float>().unwrap()
                    )
                );

                voxel += seq_voxel_delta[hit_index];

                if !ingress
                    .map(|c| approx_eq(c, 0.0) || approx_eq(c, 1.0))
                    .reduce_or() || !ingress
                    .map(|c| c >= -0.00001 && c <= 1.00001)
                    .reduce_and() {
                    dbg!(ingress);
                }
                debug_assert!(ingress
                    .map(|c| approx_eq(c, 0.0) || approx_eq(c, 1.0))
                    .reduce_or());
                debug_assert!(ingress
                    .map(|c| c >= -0.00001 && c <= 1.00001)
                    .reduce_and());

                // collision
                if voxel.partial_cmpge(&Vec3::new(0, 0, 0)).reduce_and() &&
                    voxel.partial_cmplt(&Vec3::new(5, 5, 5)).reduce_and() {

                    hits += 1;

                }

                // println!("shazam! {:#?}", (ingress, voxel));
            }

            // compute color
            //let rgb: Rgb<float> = Rgb::one() - (Rgb::one() / 15.0 * hits as float);
            let rgb: Rgb<float> = Rgb {
                r: hits as float / 15.0,
                g: hits as float / 10.0,
                b: hits as float / 5.0,
            };

            /*
            // debug color
            let debug_val: Vec3<float> = direction;
            let rgb = Rgb::<float> {
                r: debug_val.x,
                g: debug_val.y,
                b: debug_val.z,
            };
            */

            Rgba::<float>::from_opaque(rgb)
                .map(|c| c.clamp(0.0, 1.0))
                .map(|c| (c * 0xFF as float) as u8)
        },
    );
}
