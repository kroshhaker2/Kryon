use std::collections::HashMap;

use crate::worldgen::chunk::BiomeId;

#[derive(Debug, Clone)]
pub struct RegisteredBiome {
    pub id: BiomeId,
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct BiomeRegistry {
    pub biomes: Vec<RegisteredBiome>,
    pub ids_by_key: HashMap<String, BiomeId>,
}

impl BiomeRegistry {
    pub fn read(bytes: &[u8]) -> Option<Self> {
        let mut offset = 0;
        let count = read_u32(bytes, &mut offset)? as usize;
        let mut biomes = Vec::with_capacity(count);
        let mut ids_by_key = HashMap::with_capacity(count);

        for _ in 0..count {
            let id = read_u32(bytes, &mut offset)?;
            let key_len = read_u32(bytes, &mut offset)? as usize;
            let end = offset.checked_add(key_len)?;
            let key = std::str::from_utf8(bytes.get(offset..end)?)
                .ok()?
                .to_owned();
            offset = end;

            if ids_by_key.insert(key.clone(), id).is_some() {
                return None;
            }

            biomes.push(RegisteredBiome { id, key });
        }

        (offset == bytes.len()).then_some(Self { biomes, ids_by_key })
    }
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
    let end = offset.checked_add(size_of::<u32>())?;
    let value = u32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
    *offset = end;
    Some(value)
}
