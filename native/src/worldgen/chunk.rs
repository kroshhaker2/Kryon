pub const CHUNK_WIDTH: usize = 16;
pub const SUB_CHUNK_HEIGHT: usize = 16;
pub const BLOCKS_PER_SUB_CHUNK: usize = CHUNK_WIDTH * CHUNK_WIDTH * SUB_CHUNK_HEIGHT;
pub const BIOMES_PER_SUB_CHUNK: usize = 4 * 4 * 4;

pub type BlockStateId = u32;
pub type BiomeId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkKind {
    Proto,
    Level,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkStatus {
    Empty,
    StructureStarts,
    StructureReferences,
    Biomes,
    Terrain,
    Features,
    InitializeLight,
    Light,
    Spawn,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone)]
pub struct ChunkAccess {
    pub pos: ChunkPos,
    pub kind: ChunkKind,
    pub status: ChunkStatus,
    pub sections: Vec<SubChunk>,
}

pub type Chunk = ChunkAccess;

// TODO: use it
pub enum PaletteKind<T> {
    Local(Vec<T>),
    Global,
}

#[derive(Debug, Clone)]
pub struct PalettedContainer<T> {
    /// Один ID для однородного сабчанка, локальная палитра или пустой список для глобальных ID.
    pub palette: Vec<T>,
    /// Уже упакованные слова: локальные индексы или глобальные ID.
    pub data: Vec<u64>,
    pub bits_per_entry: u8,
    pub size: usize,
}

impl PalettedContainer<BlockStateId> {
    /// Записывает подготовленный контейнер блоков для PalettedContainer.read() в Paper.
    pub fn write_block_states(&self, out: &mut Vec<u8>) {
        out.push(self.bits_per_entry);
        match self.bits_per_entry {
            0 => {
                write_varint(out, self.palette[0]);
                return;
            }
            4..=8 => {
                write_varint(out, self.palette.len() as u32);
                for &id in &self.palette {
                    write_varint(out, id);
                }
            }
            1..=3 => panic!("block palettes require at least four bits per local index"),
            _ => {}
        }
        for word in &self.data {
            out.extend_from_slice(&word.to_be_bytes());
        }
    }
}

fn write_varint(out: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        out.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

#[derive(Debug, Clone)]
pub struct SubChunk {
    pub y: i32,
    pub block_states: PalettedContainer<BlockStateId>,
    pub biomes: PalettedContainer<BiomeId>,
    pub non_empty_block_count: u16,
    pub ticking_block_count: u16,
    pub ticking_fluid_count: u16,
}
