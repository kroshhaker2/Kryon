use std::collections::BTreeMap;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct DimensionType {
    pub json: Vec<u8>,
    pub has_fixed_time: bool,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub has_ender_dragon_fight: bool,
    pub coordinate_scale: f64,
    pub min_y: i32,
    pub height: i32,
    pub logical_height: i32,
    pub infiniburn: HolderSet,
    pub ambient_light: f32,
    pub monster_settings: MonsterSettings,
    pub skybox: Skybox,
    pub cardinal_light_type: CardinalLightType,
    pub attributes: BTreeMap<String, String>,
    pub timelines: HolderSet,
    pub default_clock: Option<RegistryHolder>,
}

impl DimensionType {
    pub fn from_json(json: &[u8]) -> Option<Self> {
        let value: Value = serde_json::from_slice(json).ok()?;
        let object = value.as_object()?;

        Some(Self {
            json: json.to_vec(),
            has_fixed_time: read_bool(object, "has_fixed_time").unwrap_or(false),
            has_skylight: read_bool(object, "has_skylight")?,
            has_ceiling: read_bool(object, "has_ceiling")?,
            has_ender_dragon_fight: read_bool(object, "has_ender_dragon_fight")?,
            coordinate_scale: read_f64(object, "coordinate_scale")?,
            min_y: read_i32(object, "min_y")?,
            height: read_i32(object, "height")?,
            logical_height: read_i32(object, "logical_height")?,
            infiniburn: HolderSet::Direct(Vec::new()),
            ambient_light: read_f32(object, "ambient_light")?,
            monster_settings: MonsterSettings {
                monster_spawn_light_test: IntProvider::Constant(0),
                monster_spawn_block_light_limit: 0,
            },
            skybox: match read_string(object, "skybox").unwrap_or("overworld") {
                "none" => Skybox::None,
                "overworld" => Skybox::Overworld,
                "end" => Skybox::End,
                _ => return None,
            },
            cardinal_light_type: match read_string(object, "cardinal_light").unwrap_or("default") {
                "default" => CardinalLightType::Default,
                "nether" => CardinalLightType::Nether,
                _ => return None,
            },
            attributes: BTreeMap::new(),
            timelines: HolderSet::Direct(Vec::new()),
            default_clock: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HolderSet {
    Direct(Vec<RegistryHolder>),
    Named {
        tag: String,
        values: Vec<RegistryHolder>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegistryHolder {
    Reference(String),
    Direct(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skybox {
    None,
    Overworld,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalLightType {
    Default,
    Nether,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonsterSettings {
    pub monster_spawn_light_test: IntProvider,
    pub monster_spawn_block_light_limit: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntProvider {
    Constant(i32),
    Uniform {
        min_inclusive: i32,
        max_inclusive: i32,
    },
    BiasedToBottom {
        min_inclusive: i32,
        max_inclusive: i32,
    },
    VeryBiasedToBottom {
        min_inclusive: i32,
        max_inclusive: i32,
    },
    Clamped {
        source: Box<IntProvider>,
        min_inclusive: i32,
        max_inclusive: i32,
    },
    ClampedNormal {
        mean: f32,
        deviation: f32,
        min_inclusive: i32,
        max_inclusive: i32,
    },
    WeightedList(Vec<WeightedIntProvider>),
    Trapezoid {
        min_inclusive: i32,
        max_inclusive: i32,
        plateau: i32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIntProvider {
    pub provider: IntProvider,
    pub weight: i32,
}

fn read_bool(object: &serde_json::Map<String, Value>, field: &str) -> Option<bool> {
    object.get(field)?.as_bool()
}

fn read_i32(object: &serde_json::Map<String, Value>, field: &str) -> Option<i32> {
    i32::try_from(object.get(field)?.as_i64()?).ok()
}

fn read_f32(object: &serde_json::Map<String, Value>, field: &str) -> Option<f32> {
    let value = object.get(field)?.as_f64()?;
    value.is_finite().then_some(value as f32)
}

fn read_f64(object: &serde_json::Map<String, Value>, field: &str) -> Option<f64> {
    let value = object.get(field)?.as_f64()?;
    value.is_finite().then_some(value)
}

fn read_string<'a>(object: &'a serde_json::Map<String, Value>, field: &str) -> Option<&'a str> {
    object.get(field)?.as_str()
}
