
pub use packed_simd::{f32x4, i32x4, m32x4};
use std::{
    ops::*,
    fmt::{self, Display, Formatter},
};


/// Declare a wec.
macro_rules! wec {
    (
        $(#[$attr:meta])*
        [bitops=$bitops:ident]
        [bitshift=$bitshift:ident]
        [arithm=$arithm:ident]
        [mask($($mask:tt)*)]
        [ternary($($ternary:tt)*)]
        [cast($($cast:tt)*)]
        [transmute($($transmute:tt)*)]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
        axis $axis_struct:ident { $($axis:ident),* $(,)? }
    )=>{        
        $(#[$attr])*
        #[derive(Copy, Clone, Debug, Default)]
        pub struct $wec_struct {
            $( pub $field: $comp, )*
        }
        
        impl $wec_struct {
            pub fn splat(n: $comp) -> $wec_struct {
                $wec_struct {
                    $( $field: n, )*
                }
            }
            
            /// VectorCast delegate function to enable turbofish syntax.
            pub fn cast<Scalar>(self) -> <Self as VectorCast<Scalar>>::Output
            where
                Self: VectorCast<Scalar>
            {
                <Self as VectorCast<Scalar>>::v_cast(self)
            }
            
            /// VectorTransmute delegate function to enable turbofish syntax.
            pub fn transmute<Scalar>(self) -> <Self as VectorTransmute<Scalar>>::Output
            where
                Self: VectorTransmute<Scalar>
            {
                <Self as VectorTransmute<Scalar>>::v_transmute(self)
            }
        }
        
        impl Index<$axis_struct> for $wec_struct {
            type Output = $comp;
            
            fn index(&self, i: $axis_struct) -> &$comp {
                match i {
                    $( $axis_struct::$axis => &self.$field, )*
                }
            }
        }
        
        impl IndexMut<$axis_struct> for $wec_struct {
            fn index_mut(&mut self, i: $axis_struct) -> &mut $comp {
                match i {
                    $( $axis_struct::$axis => &mut self.$field, )*
                }
            }
        }
        
        wec! {
            [shield=$bitops]
            @bitops
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            [shield=$bitshift]
            @bitshift
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            [shield=$arithm]
            @arithm
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @mask
            [mask($($mask)*)]
            wec $wec_struct($comp) { $($field),* } 
        }
        wec! {
            @ternary
            [ternary($($ternary)*)]
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @cast
            [cast($($cast)*)]
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @transmute
            [transmute($($transmute)*)]
            wec $wec_struct($comp) { $($field),* }
        }
    };
    
    ([shield=false $($t2:tt)*] $($t:tt)*)=>{};
    ([shield=true]
        @arithm
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        wec! {
            @op(Add, add)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op(Sub, sub)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op(Mul, mul)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op(Div, div)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op(Rem, rem)
            wec $wec_struct($comp) { $($field),* }
        }
        
        wec! {
            @op_unary(Neg, neg)
            wec $wec_struct { $($field),* }
        }
        
        wec! {
            @op_assn(AddAssign, add_assign)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op_assn(RemAssign, rem_assign)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op_assn(SubAssign, sub_assign)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op_assn(MulAssign, mul_assign)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op_assn(DivAssign, div_assign)
            wec $wec_struct($comp) { $($field),* }
        }
    };
    ([shield=true]
        @bitshift
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        wec! {
            @op(Shl, shl)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op(Shr, shr)
            wec $wec_struct($comp) { $($field),* }
        }
        
        wec! {
            @op_assn(ShlAssign, shl_assign)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op_assn(ShrAssign, shr_assign)
            wec $wec_struct($comp) { $($field),* }
        }
    };
    ([shield=true]
        @bitops
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        wec! {
            @op(BitAnd, bitand)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op(BitOr, bitor)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op(BitXor, bitxor)
            wec $wec_struct($comp) { $($field),* }
        }
        
        
        wec! {
            @op_unary(Not, not)
            wec $wec_struct { $($field),* }
        }
        
        wec! {
            @op_assn(BitAndAssign, bitand_assign)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op_assn(BitOrAssign, bitor_assign)
            wec $wec_struct($comp) { $($field),* }
        }
        wec! {
            @op_assn(BitXorAssign, bitxor_assign)
            wec $wec_struct($comp) { $($field),* }
        }
    };
    (   
        @ternary
        [ternary(false)]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (   
        @ternary
        [ternary(simd {})]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (   
        @ternary
        [ternary(bool {})]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (   
        @ternary
        [ternary(simd {
            $rhs:ident;
            $($rhs_tail:tt)*
        })]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl Ternary<$rhs> for $wec_struct {
            fn ternary(self, if_true: $rhs, if_false: $rhs) -> $rhs {
                $rhs {
                    $( $field: self.$field.select(if_true.$field, if_false.$field), )*
                }
            }
        }
        wec! {
            @ternary
            [ternary(simd {
                $($rhs_tail)*
            })]
            wec $wec_struct($comp) { $($field),* }
        }
    };
    (
        @ternary
        [ternary(bool {
            $rhs:ident;
            $($rhs_tail:tt)*
        })]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl Ternary<$rhs> for $wec_struct {
            fn ternary(self, if_true: $rhs, if_false: $rhs) -> $rhs {
                $rhs {
                    $( $field: if self.$field { if_true.$field } else { if_false.$field }, )*
                }
            }
        }
        wec! {
            @ternary
            [ternary(bool {
                $($rhs_tail)*
            })]
            wec $wec_struct($comp) { $($field),* }
        }
    };
    (   
        @cast
        [cast(false)]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (   
        @cast
        [cast(simd {})]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (
        @cast
        [cast(simd {
            $scalar:ident -> $vector:ident;
            $($tail:tt)*
        })]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl VectorCast<$scalar> for $wec_struct {
            type Output = $vector;
            
            fn v_cast(self) -> $vector {
                $vector {
                    $( $field: packed_simd::FromCast::from_cast(self.$field), )*
                }
            }
        }
        wec! {
            @cast
            [cast(simd {
                $($tail)*
            })]
            wec $wec_struct($comp) { $($field),* }
        }
    };
    (   
        @transmute
        [transmute(false)]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (   
        @transmute
        [transmute(simd {})]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (
        @transmute
        [transmute(simd {
            $scalar:ident -> $vector:ident;
            $($tail:tt)*
        })]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl VectorTransmute<$scalar> for $wec_struct {
            type Output = $vector;
            
            fn v_transmute(self) -> $vector {
                $vector {
                    $( $field: packed_simd::FromBits::from_bits(self.$field), )*
                }
            }
        }
        wec! {
            @transmute
            [transmute(simd {
                $($tail)*
            })]
            wec $wec_struct($comp) { $($field),* }
        }
    };
    (   
        @cast
        [cast(scalar {})]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (
        @cast
        [cast(scalar {
            // special case for bools
            bool -> $vector:ident;
            $($tail:tt)*
        })]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl VectorCast<bool> for $wec_struct {
            type Output = $vector;
            
            fn v_cast(self) -> $vector {
                $vector {
                    $( $field: self.$field != (0 as _), )*
                }
            }
        }
        wec! {
            @cast
            [cast(scalar {
                $($tail)*
            })]
            wec $wec_struct($comp) { $($field),* }
        }
    };
    (   
        @cast
        [cast(bool {})]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (
        @cast
        [cast(bool {
            $scalar:ident -> $vector:ident;
            $($tail:tt)*
        })]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl VectorCast<$scalar> for $wec_struct {
            type Output = $vector;
            
            fn v_cast(self) -> $vector {
                $vector {
                    $( $field: if self.$field { 1 as _ } else { 0 as _ }, )*
                }
            }
        }
        wec! {
            @cast
            [cast(bool {
                $($tail)*
            })]
            wec $wec_struct($comp) { $($field),* }
        }
    };
    (   
        @transmute
        [transmute(bool {})]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (
        @cast
        [cast(scalar {
            $scalar:ident -> $vector:ident;
            $($tail:tt)*
        })]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl VectorCast<$scalar> for $wec_struct {
            type Output = $vector;
            
            fn v_cast(self) -> $vector {
                $vector {
                    $( $field: self.$field as $scalar, )*
                }
            }
        }
        wec! {
            @cast
            [cast(scalar {
                $($tail)*
            })]
            wec $wec_struct($comp) { $($field),* }
        }
    };
    (   
        @transmute
        [transmute(scalar {})]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (
        @transmute
        [transmute(scalar {
            $scalar:ident -> $vector:ident;
            $($tail:tt)*
        })]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl VectorTransmute<$scalar> for $wec_struct {
            type Output = $vector;
            
            fn v_transmute(self) -> $vector {
                $vector {
                    $( $field: unsafe {
                        std::mem::transmute(self.$field)
                    }, )*
                }
            }
        }
        wec! {
            @transmute
            [transmute(scalar {
                $($tail)*
            })]
            wec $wec_struct($comp) { $($field),* }
        }
    };
    (
        @mask
        [mask(false)]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{};
    (
        @mask
        [mask(simd $mask:ident)]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl $wec_struct {
            /// `==` bool mask.
            pub fn m_eq(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field.eq(rhs.$field), )*
                }
            }
            
            /// `!=` mask.
            pub fn m_ne(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field.ne(rhs.$field), )*
                }
            }
            
            /// `<` mask.
            pub fn m_lt(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field.lt(rhs.$field), )*
                }
            }
            
            /// `<=` mask.
            pub fn m_le(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field.le(rhs.$field), )*
                }
            }
            
            /// `>` mask.
            pub fn m_gt(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field.gt(rhs.$field), )*
                }
            }
            
            /// `>=` mask.
            pub fn m_ge(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field.ge(rhs.$field), )*
                }
            }
        }
    };
    (
        @mask
        [mask(bool $mask:ident)]
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl $wec_struct {
            /// `==` bool mask.
            pub fn m_eq(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field == rhs.$field, )*
                }
            }
            
            /// `!=` mask.
            pub fn m_ne(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field != rhs.$field, )*
                }
            }
            
            /// `<` mask.
            pub fn m_lt(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field < rhs.$field, )*
                }
            }
            
            /// `<=` mask.
            pub fn m_le(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field <= rhs.$field, )*
                }
            }
            
            /// `>` mask.
            pub fn m_gt(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field > rhs.$field, )*
                }
            }
            
            /// `>=` mask.
            pub fn m_ge(self, rhs: $wec_struct) -> $mask {
                $mask {
                    $( $field: self.$field >= rhs.$field, )*
                }
            }
        }
    };
    
    
    (
        @op_unary($trait:ident, $method:ident)
        wec $wec_struct:ident { $($field:ident),* $(,)? }
    )=>{
        impl $trait for $wec_struct {
            type Output = $wec_struct;
            
            fn $method(self) -> $wec_struct {
                $wec_struct {
                    $( $field: $trait::$method(self.$field) ,)*
                }
            }
        }
    };
    (
        @op($trait:ident, $method:ident)
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl $trait<$wec_struct> for $wec_struct {
            type Output = $wec_struct;
            
            fn $method(self, rhs: $wec_struct) -> $wec_struct {
                $wec_struct {
                    $( $field: $trait::$method(self.$field, rhs.$field), )*
                }
            }
        }
        
        impl $trait<$comp> for $wec_struct {
            type Output = $wec_struct;
            
            fn $method(self, rhs: $comp) -> $wec_struct {
                $wec_struct {
                    $($ field: $trait::$method(self.$field, rhs), )*
                }
            }
        }
    };
    (
        @op_assn($trait:ident, $method:ident)
        wec $wec_struct:ident($comp:ty) { $($field:ident),* $(,)? }
    )=>{
        impl $trait<$wec_struct> for $wec_struct {
            fn $method(&mut self, rhs: $wec_struct) {
                $( $trait::$method(&mut self.$field, rhs.$field); )*
            }
        }
        
        impl $trait<$comp> for $wec_struct {
            fn $method(&mut self, rhs: $comp) {
                $( $trait::$method(&mut self.$field, rhs); )*
            }
        }
    };
}

/// Axis in 2D space.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Axis2 {
    X, Y
}

/// Axis in 3D space.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Axis3 {
    X, Y, Z
}

/// Ternary component-wise selection operation.
pub trait Ternary<T> {
    fn ternary(self, if_true: T, if_false: T) -> T;
}

/// Vector component-wise casting operation.
pub trait VectorCast<T> {
    type Output;
    
    fn v_cast(self) -> Self::Output;
}

/// Vector component-wise safe transmution.
pub trait VectorTransmute<T> {
    type Output;
    
    fn v_transmute(self) -> Self::Output;
}

wec! {
    /// 2D SIMD float vector.
    [bitops=false]
    [bitshift=false]
    [arithm=true]
    [mask(simd Mec2)]
    [ternary(false)]
    [cast(simd { i32 -> Wic2; bool -> Mec2; })]
    [transmute(simd { i32 -> Wic2; })]
    wec Wec2(f32x4) { x, y }
    axis Axis2 { X, Y }
}

wec! {
    /// 3D SIMD float vector.
    [bitops=false]
    [bitshift=false]
    [arithm=true]
    [mask(simd Mec3)]
    [ternary(false)]
    [cast(simd { i32 -> Wic3; bool -> Mec3; })]
    [transmute(simd { i32 -> Wic3; })]
    wec Wec3(f32x4) { x, y, z }
    axis Axis3 { X, Y, Z }
}

wec! {
    /// 2D SIMD int vector.
    [bitops=true]
    [bitshift=true]
    [arithm=true]
    [mask(simd Mec2)]
    [ternary(false)]
    [cast(simd { f32 -> Wec2; bool -> Mec2; })]
    [transmute(simd { f32 -> Wec2; })]
    wec Wic2(i32x4) { x, y }
    axis Axis2 { X, Y }
}

wec! {
    /// 3D SIMD int vector.
    [bitops=true]
    [bitshift=true]
    [arithm=true]
    [mask(simd Mec3)]
    [ternary(false)]
    [cast(simd { f32 -> Wec3; bool -> Mec3; })]
    [transmute(simd { f32 -> Wec3; })]
    wec Wic3(i32x4) { x, y, z }
    axis Axis3 { X, Y, Z }
}

wec! {
    /// 2D SIMD mask vector.
    ///
    /// It's like per-component booleans.
    [bitops=true]
    [bitshift=false]
    [arithm=false]
    [mask(simd Mec2)]
    [ternary(simd { Wec2; Wic2; Mec2; })]
    [cast(simd { f32 -> Wec2; i32 -> Wic2; })]
    [transmute(false)]
    wec Mec2(m32x4) { x, y }
    axis Axis2 { X, Y }
}

wec! {
    /// 3D SIMD mask vector.
    ///
    /// It's like per-component booleans.
    [bitops=true]
    [bitshift=false]
    [arithm=false]
    [mask(simd Mec3)]
    [ternary(simd { Wec3; Wic3; Mec3; })]
    [cast(simd { f32 -> Wec3; i32 -> Wic3; })]
    [transmute(false)]
    wec Mec3(m32x4) { x, y, z }
    axis Axis3 { X, Y, Z }
}

wec! {
    /// 2D non-SIMD float vector.
    [bitops=false]
    [bitshift=false]
    [arithm=true]
    [mask(bool Bec2)]
    [ternary(false)]
    [cast(scalar { i32 -> Vic2; bool -> Bec2; })]
    [transmute(scalar { i32 -> Vic2; })]
    wec Vec2(f32) { x, y }
    axis Axis2 { X, Y }
}

wec! {
    /// 3D non-SIMD float vector.
    [bitops=false]
    [bitshift=false]
    [arithm=true]
    [mask(bool Bec3)]
    [ternary(false)]
    [cast(scalar { i32 -> Vic3; bool -> Bec3; })]
    [transmute(scalar { i32 -> Vic3; })]
    wec Vec3(f32) { x, y, z }
    axis Axis3 { X, Y, Z }
}

wec! {
    /// 2D non-SIMD int vector.
    [bitops=true]
    [bitshift=true]
    [arithm=true]
    [mask(bool Bec2)]
    [ternary(false)]
    [cast(scalar { f32 -> Vec2; bool -> Bec2; })]
    [transmute(scalar { f32 -> Vec2; })]
    wec Vic2(i32) { x, y }
    axis Axis2 { X, Y }
}

wec! {
    /// 3D non-SIMD int vector.
    [bitops=true]
    [bitshift=true]
    [arithm=true]
    [mask(bool Bec3)]
    [ternary(false)]
    [cast(scalar { f32 -> Vec3; bool -> Bec3; })]
    [transmute(scalar { f32 -> Vec3; })]
    wec Vic3(i32) { x, y, z }
    axis Axis3 { X, Y, Z }
}

wec! {
    /// 2D non-SIMD bool vector.
    ///
    /// It's like per-component booleans.
    [bitops=true]
    [bitshift=false]
    [arithm=false]
    [mask(bool Bec2)]
    [ternary(bool { Vec2; Vic2; Bec2; })]
    [cast(bool { f32 -> Vec2; i32 -> Vic2; })]
    [transmute(false)]
    wec Bec2(bool) { x, y }
    axis Axis2 { X, Y }
}

wec! {
    /// 3D non-SIMD bool vector.
    ///
    /// It's like per-component booleans.
    [bitops=true]
    [bitshift=false]
    [arithm=false]
    [mask(bool Bec3)]
    [ternary(bool { Vec3; Vic3; Bec3; })]
    [cast(bool { f32 -> Vec3; i32 -> Vic3; })]
    [transmute(false)]
    wec Bec3(bool) { x, y, z }
    axis Axis3 { X, Y, Z }
}

