use std::collections::HashMap;

use crate::worldgen::chunk::BlockStateId;

#[derive(Debug, Clone)]
pub struct RegisteredBlockState {
    pub id: BlockStateId,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct BlockStateRegistry {
    pub states: Vec<RegisteredBlockState>,
    pub ids_by_name: HashMap<String, BlockStateId>,
}

impl BlockStateRegistry {
    pub fn read(bytes: &[u8]) -> Option<Self> {
        let mut offset = 0;
        let state_count = read_u32(bytes, &mut offset)? as usize;
        let mut states = Vec::with_capacity(state_count);
        let mut ids_by_name = HashMap::with_capacity(state_count);

        for _ in 0..state_count {
            let id = read_u32(bytes, &mut offset)?;
            let name_length = read_u32(bytes, &mut offset)? as usize;
            let name_end = offset.checked_add(name_length)?;
            let name = std::str::from_utf8(bytes.get(offset..name_end)?)
                .ok()?
                .to_owned();
            offset = name_end;

            if ids_by_name.insert(name.clone(), id).is_some() {
                return None;
            }

            states.push(RegisteredBlockState { id, name });
        }

        (offset == bytes.len()).then_some(Self {
            states,
            ids_by_name,
        })
    }
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
    let end = offset.checked_add(size_of::<u32>())?;
    let value = u32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
    *offset = end;
    Some(value)
}
