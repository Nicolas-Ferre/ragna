use crate::{Cpu, Gpu, GpuTypeDetails, GpuValue, F32, I32, U32};
use itertools::Itertools;
use std::any::TypeId;

macro_rules! simd_type {
    (
        $gpu_name:ident, $gpu_item_name:ident,
        $cpu_name:ident, $cpu_item_name:ident,
        $wgsl_name:literal, 2
    ) => {
        simd_type!(
            $gpu_name, $gpu_item_name, $cpu_name, $cpu_item_name, $wgsl_name,
            2, [(x, 0, 0..4), (y, 1, 4..8)]
        );
    };
    (
        $gpu_name:ident, $gpu_item_name:ident,
        $cpu_name:ident, $cpu_item_name:ident,
        $wgsl_name:literal, 3
    ) => {
        simd_type!(
            $gpu_name, $gpu_item_name, $cpu_name, $cpu_item_name, $wgsl_name,
            3, [(x, 0, 0..4), (y, 1, 4..8), (z, 2, 8..12)]
        );
    };
    (
        $gpu_name:ident, $gpu_item_name:ident,
        $cpu_name:ident, $cpu_item_name:ident,
        $wgsl_name:literal, 4
    ) => {
        simd_type!(
            $gpu_name, $gpu_item_name, $cpu_name, $cpu_item_name, $wgsl_name,
            4, [(x, 0, 0..4), (y, 1, 4..8), (z, 2, 8..12), (w, 3, 12..16)]
        );
    };
    (
        $gpu_name:ident, $gpu_item_name:ident,
        $cpu_name:ident, $cpu_item_name:ident,
        $wgsl_name:literal, $field_count:literal,
        [$(($field_ident:ident, $field_index:literal, $field_byte_range:expr)),+]
    ) => {
        #[doc = concat!(
            "A GPU type allowing SIMD operations on ",
            stringify!($field_count),
            " [`",
            stringify!($gpu_item_name),
            "`](crate::",
            stringify!($gpu_item_name),
            ") values.",
        )]
        #[derive(Clone, Copy)]
        pub struct $gpu_name {
            $(
                #[doc = concat!(
                    "The `",
                    stringify!($field_ident),
                    "` value.",
                )]
                pub $field_ident: $gpu_item_name,
            )+
            value: GpuValue<Self>,
        }

        impl $gpu_name {
            /// Creates a new SIMD value.
            pub fn new($($field_ident: $gpu_item_name),+) -> Self {
                let var = crate::create_uninit_var::<Self>();
                $(crate::assign(var.$field_ident, $field_ident);)+
                var
            }
        }

        impl Gpu for $gpu_name {
            type Cpu = $cpu_name;

            fn details() -> GpuTypeDetails {
                GpuTypeDetails {
                    type_id: TypeId::of::<Self>(),
                    name: Some($wgsl_name),
                    array_generics: None,
                    size: Some(4 * $field_count),
                    alignment: Some(if $field_count == 2 {8} else {16}),
                    field_types: vec![$gpu_item_name::details()],
                }
            }

            fn value(self) -> GpuValue<Self> {
                self.value
            }

            fn from_value(value: GpuValue<Self>) -> Self {
                Self {
                    $($field_ident: $gpu_item_name::from_value(value.vec_field($field_index)),)+
                    value,
                }
            }
        }


        #[doc = concat!(
            "The CPU type corresponding to `",
            stringify!($gpu_name),
            "` GPU type",
        )]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        #[allow(non_camel_case_types, clippy::derive_partial_eq_without_eq)]
        pub struct $cpu_name {
            $(
                #[doc = concat!(
                    "The `",
                    stringify!($field_ident),
                    "` value.",
                )]
                pub $field_ident: $cpu_item_name,
            )+
        }

        impl Cpu for $cpu_name {
            type Gpu = $gpu_name;

            #[allow(clippy::cast_possible_truncation)]
            fn from_gpu(bytes: &[u8]) -> Self {
                Self {
                    $($field_ident: $cpu_item_name::from_gpu(&bytes[$field_byte_range]),)+
                }
            }

            fn to_wgsl(&self) -> String {
                let params = [$(self.$field_ident.to_wgsl()),+];
                format!("<name>({})", params.into_iter().join(","))
            }
        }
    };
}

simd_type!(U32x2, U32, u32x2, u32, "vec2u", 2);
simd_type!(I32x2, I32, i32x2, i32, "vec2i", 2);
simd_type!(F32x2, F32, f32x2, f32, "vec2f", 2);
simd_type!(U32x3, U32, u32x3, u32, "vec3u", 3);
simd_type!(I32x3, I32, i32x3, i32, "vec3i", 3);
simd_type!(F32x3, F32, f32x3, f32, "vec3f", 3);
simd_type!(U32x4, U32, u32x4, u32, "vec4u", 4);
simd_type!(I32x4, I32, i32x4, i32, "vec4i", 4);
simd_type!(F32x4, F32, f32x4, f32, "vec4f", 4);
