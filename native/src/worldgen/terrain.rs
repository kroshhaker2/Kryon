use std::{array};

use crate::worldgen::chunk::{
    BLOCKS_PER_SUB_CHUNK, BlockStateId, PalettedContainer, SUB_CHUNK_HEIGHT,
};

pub struct TerrainChunk {
    pub min_y: i32,
    pub sections: Vec<TerrainSubChunk>,
}

impl TerrainChunk {
    pub fn write_to(&self, out: &mut Vec<u8>) {
        write_u32(out, self.sections.len() as u32);

        for section in &self.sections {
            let block_states = section.to_block_states();
            let mut section_bytes = Vec::new();
            block_states.write_block_states(&mut section_bytes);

            write_i32(out, section.y);
            write_u32(out, section_bytes.len() as u32);
            out.extend_from_slice(&section_bytes);
        }
    }

    pub fn set_block(&mut self, local_x: u8, world_y: i32, local_z: u8, state: BlockStateId) {
        let relative_y = world_y - self.min_y;

        assert!(relative_y >= 0);
        let section_index = (relative_y / SUB_CHUNK_HEIGHT as i32) as usize;
        let local_y = (relative_y % 16) as u8;

        let section = &mut self.sections[section_index];
        let index = TerrainSubChunk::index(local_x, local_y, local_z);
        section.blocks[index] = state;
    }
}

pub struct TerrainSubChunk {
    pub y: i32,
    pub blocks: Vec<BlockStateId>,
}

impl TerrainSubChunk {
    pub fn index(x: u8, y: u8, z: u8) -> usize {
        ((y as usize) << 8) | ((z as usize) << 4) | x as usize
    }

    pub fn to_block_states(&self) -> PalettedContainer<BlockStateId> {
        assert_eq!(
            self.blocks.len(),
            BLOCKS_PER_SUB_CHUNK,
            "a terrain sub-chunk must contain {BLOCKS_PER_SUB_CHUNK} blocks"
        );

        let mut palette = Vec::new();
        let mut palette_indices = std::collections::HashMap::new();
        let mut values = Vec::with_capacity(BLOCKS_PER_SUB_CHUNK);

        for &state in &self.blocks {
            let next_index = palette.len() as u32;
            let index = *palette_indices.entry(state).or_insert_with(|| {
                palette.push(state);
                next_index
            });
            values.push(index);
        }

        if palette.len() == 1 {
            return PalettedContainer {
                palette,
                data: Vec::new(),
                bits_per_entry: 0,
                size: BLOCKS_PER_SUB_CHUNK,
            };
        }

        if palette.len() <= 256 {
            let bits_per_entry = bits_needed((palette.len() - 1) as u32).max(4);
            return PalettedContainer {
                palette,
                data: pack_values(&values, bits_per_entry),
                bits_per_entry,
                size: BLOCKS_PER_SUB_CHUNK,
            };
        }

        let bits_per_entry = self
            .blocks
            .iter()
            .copied()
            .map(bits_needed)
            .max()
            .unwrap()
            .max(9);

        PalettedContainer {
            palette: Vec::new(),
            data: pack_values(&self.blocks, bits_per_entry),
            bits_per_entry,
            size: BLOCKS_PER_SUB_CHUNK,
        }
    }
}

fn bits_needed(value: u32) -> u8 {
    (u32::BITS - value.leading_zeros()) as u8
}

fn pack_values(values: &[u32], bits_per_entry: u8) -> Vec<u64> {
    let values_per_word = 64 / bits_per_entry as usize;
    let mut data = vec![0; values.len().div_ceil(values_per_word)];

    for (index, &value) in values.iter().enumerate() {
        let word_index = index / values_per_word;
        let bit_offset = (index % values_per_word) * bits_per_entry as usize;
        data[word_index] |= (value as u64) << bit_offset;
    }

    data
}

fn write_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}
