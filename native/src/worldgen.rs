use crate::worldgen::{
    chunk::SUB_CHUNK_HEIGHT,
    dimension::DimensionType,
    registry::BlockStateRegistry,
    terrain::{TerrainChunk, TerrainSubChunk},
};

pub mod biome;
pub mod chunk;
pub mod dimension;
pub mod registry;
pub mod rng;
pub mod terrain;

pub struct WorldGenerator {
    pub seed: i64,
    pub dimension: DimensionType,
    pub block_states: BlockStateRegistry,
}

impl WorldGenerator {
    pub fn generate_chunk(&self, x: i32, z: i32, bytes: &mut Vec<u8>) {
        let terrain = self.generate_terrain(x, z);
        terrain.write_to(bytes);
    }

    fn generate_terrain(&self, chunk_x: i32, chunk_z: i32) -> TerrainChunk {
        let height = self.dimension.height;
        let min_y = self.dimension.min_y;

        let sections_count = (height / SUB_CHUNK_HEIGHT as i32) as usize;
        let first_section_y = min_y / SUB_CHUNK_HEIGHT as i32;

        let mut sections = Vec::with_capacity(sections_count);

        for index in 0..sections_count {
            let section_y = first_section_y + index as i32;

            sections.push(TerrainSubChunk {
                y: section_y,
                blocks: vec![self.block_states.ids_by_name["minecraft:air"]; 4096],
            });
        }

        let mut chunk = TerrainChunk {
            min_y: self.dimension.min_y,
            sections,
        };

        for local_z in 0..16 {
            for local_x in 0..16 {
                let world_x = chunk_x * 16 + local_x;
                let world_z = chunk_z * 16 + local_z;

                let surface_y = 64;

                for y in min_y..(min_y + height) {
                    if y > surface_y {
                        continue;
                    }

                    chunk.set_block(
                        local_x as u8,
                        y,
                        local_z as u8,
                        self.block_states.ids_by_name["minecraft:stone"],
                    );
                }
            }
        }

        chunk
    }
}
