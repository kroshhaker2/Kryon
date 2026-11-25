# Native DimensionType protocol

This protocol transfers the complete Paper 26.3
`net.minecraft.world.level.dimension.DimensionType` record from Java to Rust.
Java uses `DimensionType.DIRECT_CODEC` with `JsonOps.INSTANCE`; the result is
encoded as UTF-8 bytes without a wrapper or length prefix. The FFI arguments
carry the address and byte length.

Rust parses the simple scalar fields into `DimensionType`, retains the original
JSON in `DimensionType.json`, and currently leaves complex fields as
placeholders.

The binary layout below is reserved for a later native codec and is not used by
the current Java bridge.

## Envelope

```text
bytes[4]  magic = "KDT1"
u16       version = 1
payload   DimensionType
```

## DimensionType payload

Fields are written in this exact order:

```text
u8             has_fixed_time
u8             has_skylight
u8             has_ceiling
u8             has_ender_dragon_fight
f64            coordinate_scale
i32            min_y
i32            height
i32            logical_height
HolderSet      infiniburn
f32            ambient_light
MonsterSettings monster_settings
u8             skybox
u8             cardinal_light_type
map<String, String> attributes
HolderSet      timelines
u8             has_default_clock
RegistryHolder default_clock  // only when has_default_clock is 1
```

Boolean values are `0` or `1`. `skybox` uses `0 = none`, `1 = overworld`,
`2 = end`. `cardinal_light_type` uses `0 = default`, `1 = nether`.

## Registry holders

```text
u8 holder_kind
```

- `0`: `Reference`, followed by its registry key string.
- `1`: `Direct`, followed by the JSON string emitted by the Paper codec.

`HolderSet` starts with `u8 set_kind`:

- `0`: direct; followed by `u32 value_count` and that many `RegistryHolder`s.
- `1`: named; followed by its tag key string, `u32 value_count`, and the
  resolved `RegistryHolder`s.

The resolved values are included for named sets so Rust does not need access
to Java registries after generator creation.

## Maps and lists

```text
u32 count
item[count]
```

Maps use sorted key order. Each map item is `String key`, then `String value`.

## MonsterSettings

```text
IntProvider monster_spawn_light_test
i32         monster_spawn_block_light_limit
```

## IntProvider

Each provider starts with `u8 provider_kind`:

```text
0 Constant:             i32 value
1 Uniform:              i32 min, i32 max
2 BiasedToBottom:       i32 min, i32 max
3 VeryBiasedToBottom:   i32 min, i32 max
4 Clamped:              IntProvider source, i32 min, i32 max
5 ClampedNormal:        f32 mean, f32 deviation, i32 min, i32 max
6 WeightedList:         u32 count, WeightedIntProvider[count]
7 Trapezoid:            i32 min, i32 max, i32 plateau
```

`WeightedIntProvider` is `IntProvider provider`, followed by `i32 weight`.
