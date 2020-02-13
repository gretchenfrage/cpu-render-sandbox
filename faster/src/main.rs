
extern crate rayon;
extern crate packed_simd;

pub mod wec;

use crate::wec::*;
use rayon::prelude::*;
use packed_simd::{f32x4, i32x4};

/// Fragment width of tareget.
const XFRAGS: u32 = 1024;

/// Fragment height of target.
const YFRAGS: u32 = 1024;

/// SIMD units per concurrency block.
const BLOCK: u32 = 256 / SIMD;

/// Texels per SIMD unit.
const SIMD: u32 = 4;

/// Component type.
type Comp = f32;
/// Fragment (pixel) type.
type Frag = Wec2;

/// Blocks per row.
const ROW_BLOCKS: u32 = XFRAGS / BLOCK / SIMD;

/// Image canvas.
/// 
/// Row-major layout.
type Target = 
    [[[Frag
    ; (BLOCK / SIMD) as usize]
    ; (ROW_BLOCKS) as usize]
    ; (YFRAGS) as usize];
    
type Block = 
    [Frag
    ; (BLOCK / SIMD) as usize];
    
type Row = 
    [[Frag
    ; (BLOCK / SIMD) as usize]
    ; (ROW_BLOCKS) as usize];

/// Texel color calculation, per-SIMD-unit.
trait Program: Send + Sync {
    fn run(&self, texel: Wic2) -> Frag;
}

struct Foo;

impl Program for Foo {
    fn run(&self, texel: Wic2) -> Frag {
        let mut dir: Wec3 = Wec3 {
            x: f32x4::splat(0.),
            y: f32x4::splat(0.),
            z: f32x4::splat(1.),
        };
        
        let mut voxel: Wic3 = Wic3 {
            x: i32x4::splat(0),
            y: i32x4::splat(0),
            z: i32x4::splat(0),
        };
        
        let mut ingress: Wec3 = Wec3 {
            x: f32x4::splat(0.5),
            y: f32x4::splat(0.5),
            z: f32x4::splat(0.),
        };
        /*
        for _ in 0..50 {
            fn ceil(v: Wec3) -> Wic3 {
                (v % Wec3::splat(1.)).m_eq(Wec3::splat(0.))
                    .ternary()
            }
            
            let planes = dir % Wec3::splat(1.)
        }
        */
        unimplemented!()
    }
}

fn main() {
    
    fn fj_blocks<P: Program>(
        slice: &mut [Block], 
        program: &P,
        row: u32,
        col_start: u32,
    ) {
        if let &mut [ref mut block] = slice {
            for (i, unit) in block.iter_mut().enumerate() {
                let texel = Wic2 {
                    x: i32x4::new(
                        ((col_start + i as u32) * SIMD + 0) as _,
                        ((col_start + i as u32) * SIMD + 1) as _,
                        ((col_start + i as u32) * SIMD + 2) as _,
                        ((col_start + i as u32) * SIMD + 3) as _,
                    ),
                    y: i32x4::splat(row as _),
                };
                let color = program.run(texel);
                *unit = color;
            }
        } else {
            let mid = slice.len() / 2;
            let (a, b) = slice.split_at_mut(mid);
            rayon::join(|| fj_blocks(a, program, row, col_start),
                        || fj_blocks(b, program, row, col_start + mid as u32));
        }
    }
    
    fn fj_rows<P: Program>(
        slice: &mut [Row],
        program: &P,
        row_start: u32,
    ) {
        if let &mut [ref mut row] = slice {
            fj_blocks(row, program, row_start, 0);
        } else {
            let mid = slice.len() / 2;
            let (a, b) = slice.split_at_mut(mid);
            rayon::join(|| fj_rows(a, program, row_start),
                        || fj_rows(b, program, row_start + mid as u32));
        }
    }
    
    let mut target: Box<Target> = Box::new(
        [[[Frag::default()
        ; (BLOCK / SIMD) as usize]
        ; (ROW_BLOCKS) as usize]
        ; (YFRAGS) as usize]);
        
        
    fn render<P: Program>(program: &P, target: &mut Target) {
        fj_rows(target, program, 0);
    }
    
    /*
    fn split<P: Program>(
        target: &mut Target, 
        program: &P,
        row: u32,
        start: u32,
        stop: u32,
    ) {
        if start == stop {
            
        } else {
            
        }
    }
    */
}
