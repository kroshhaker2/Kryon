#![deny(unsafe_op_in_unsafe_fn)]

use crate::worldgen::registry::BlockStateRegistry;
use crate::worldgen::{WorldGenerator, dimension::DimensionType};

mod worldgen;

#[unsafe(no_mangle)]
pub extern "C" fn kryon_native_version() -> u32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn kryon_generator_create(
    seed: i64,
    dimension: *const u8,
    dimension_len: usize,
    block_states: *const u8,
    block_states_len: usize,
) -> *mut WorldGenerator {
    if dimension.is_null() || block_states.is_null() {
        return std::ptr::null_mut();
    }

    let dimension = unsafe { std::slice::from_raw_parts(dimension, dimension_len) };
    let Some(dimension) = DimensionType::from_json(dimension) else {
        return std::ptr::null_mut();
    };

    let block_states = unsafe { std::slice::from_raw_parts(block_states, block_states_len) };
    let Some(block_states) = BlockStateRegistry::read(block_states) else {
        return std::ptr::null_mut();
    };

    Box::into_raw(Box::new(WorldGenerator {
        seed,
        dimension,
        block_states,
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn kryon_generator_generate(
    generator: *mut WorldGenerator,
    chunk_x: i32,
    chunk_z: i32,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    if generator.is_null() || out_ptr.is_null() || out_len.is_null() {
        return 1;
    }

    let generator = unsafe { &*generator };
    let mut bytes = Vec::new();

    generator.generate_chunk(chunk_x, chunk_z, &mut bytes);

    let bytes = bytes.into_boxed_slice();
    let len = bytes.len();
    let ptr = Box::into_raw(bytes) as *mut u8;

    unsafe {
        *out_ptr = ptr;
        *out_len = len;
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kryon_generator_destroy(generator: *mut WorldGenerator) {
    if !generator.is_null() {
        unsafe {
            drop(Box::from_raw(generator));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kryon_buffer_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
        }
    }
}
