use crate::worldgen::{
    chunk::SUB_CHUNK_HEIGHT,
    dimension::DimensionType,
    perlin_noise::PerlinNoise,
    registry::BlockStateRegistry,
    rng::RandomSource,
    terrain::{TerrainChunk, TerrainSubChunk},
};

pub mod biome;
pub mod chunk;
pub mod dimension;
pub mod perlin_noise;
pub mod registry;
pub mod rng;
pub mod terrain;

pub struct WorldGenerator {
    pub seed: i64,
    pub dimension: DimensionType,
    pub block_states: BlockStateRegistry,
    random: RandomSource,
    perlin_noise: PerlinNoise,
}

impl WorldGenerator {
    pub fn new(seed: i64, dimension: DimensionType, block_states: BlockStateRegistry) -> Self {
        let mut random = RandomSource::new(0, seed);
        let perlin_noise = PerlinNoise::new(&mut random);

        Self {
            seed,
            dimension,
            block_states,
            random,
            perlin_noise,
        }
    }

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

        let frequency: f64 = 0.01;

        for z in 0..16 {
            for x in 0..16 {
                let world_x = chunk_x * 16 + x;
                let world_z = chunk_z * 16 + z;

                for y in min_y..(min_y + height) {
                    let noise = self.perlin_noise.noise(
                        world_x as f64 * frequency,
                        y as f64 * frequency,
                        world_z as f64 * frequency,
                        &mut [0.0; 3],
                    );

                    let density = noise + (64.0 - y as f32) / 32.0;

                    if density <= 0.0 {
                        continue;
                    }

                    chunk.set_block(
                        x as u8,
                        y,
                        z as u8,
                        self.block_states.ids_by_name["minecraft:stone"],
                    );
                }
            }
        }

        chunk
    }
}
