package io.papermc.paper.kryon;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.io.ByteArrayOutputStream;
import java.nio.file.Path;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.HolderLookup;
import net.minecraft.resources.RegistryOps;
import net.minecraft.core.registries.Registries;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.Property;
import net.minecraft.world.level.dimension.DimensionType;

public final class KryonNative {
    private static final Arena ARENA = Arena.ofShared();

    private static final SymbolLookup LIBRARY =
        SymbolLookup.libraryLookup(
            Path.of("libraries/kryon/native/linux-x86_64/libkryon_native.so")
                .toAbsolutePath(),
            ARENA
        );

    private static final MethodHandle VERSION =
        Linker.nativeLinker().downcallHandle(
            LIBRARY.find("kryon_native_version").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT)
        );

    public static int version() {
        try {
            return (int) VERSION.invokeExact();
        } catch (Throwable throwable) {
            throw new RuntimeException("Failed to call kryon_native_version", throwable);
        }
    }

    private static final MethodHandle CREATE_GENERATOR =
        Linker.nativeLinker().downcallHandle(
            LIBRARY.find("kryon_generator_create").orElseThrow(),
            FunctionDescriptor.of(
                ValueLayout.ADDRESS, // generator
                ValueLayout.JAVA_LONG, // seed
                ValueLayout.ADDRESS, // dimension
                ValueLayout.JAVA_LONG, // dimension length
                ValueLayout.ADDRESS, // block states
                ValueLayout.JAVA_LONG // block states length
            )
        );

    private static final MethodHandle GENERATOR_GENERATE =
        Linker.nativeLinker().downcallHandle(
            LIBRARY.find("kryon_generator_generate").orElseThrow(),
            FunctionDescriptor.of(
                ValueLayout.JAVA_INT, // result
                ValueLayout.ADDRESS, // generator
                ValueLayout.JAVA_INT, ValueLayout.JAVA_INT, // chunkX, chunkZ
                ValueLayout.ADDRESS, ValueLayout.ADDRESS // ptr, len
            )
        );

    private static final MethodHandle DESTROY_GENERATOR =
        Linker.nativeLinker().downcallHandle(
            LIBRARY.find("kryon_generator_destroy").orElseThrow(),
            FunctionDescriptor.ofVoid(
                ValueLayout.ADDRESS // generator
            )
        );

    private static final MethodHandle FREE_BUFFER =
        Linker.nativeLinker().downcallHandle(
            LIBRARY.find("kryon_buffer_free").orElseThrow(),
            FunctionDescriptor.ofVoid(
                ValueLayout.ADDRESS,
                ValueLayout.JAVA_LONG
            )
        );

    public static MemorySegment createGenerator(long seed, DimensionType dimensionType, HolderLookup.Provider registries) {
        byte[] blockStates = writeBlockStateRegistry();
        byte[] dimensionBytes = writeDimensionType(dimensionType, registries);

        try (Arena arena = Arena.ofConfined()) {
            MemorySegment nativeBlockStates = arena.allocate(blockStates.length);
            nativeBlockStates.asByteBuffer().put(blockStates);

            MemorySegment nativeDimensionType = arena.allocate(dimensionBytes.length);
            nativeDimensionType.asByteBuffer().put(dimensionBytes);

            MemorySegment generator = (MemorySegment) CREATE_GENERATOR.invokeExact(
                seed,
                nativeDimensionType,
                (long) dimensionBytes.length,
                nativeBlockStates,
                (long) blockStates.length
            );
            if (generator.address() == 0) {
                throw new IllegalStateException("Native generator rejected the block state registry");
            }
            return generator;
        } catch (Throwable throwable) {
            throw new RuntimeException("Failed to create native generator", throwable);
        }
    }

    public static void destroyGenerator(MemorySegment generator) {
        try {
            DESTROY_GENERATOR.invokeExact(generator);
        } catch (Throwable throwable) {
            throw new RuntimeException("Failed to destroy native generator", throwable);
        }
    }

    public static GeneratedChunk generateChunk(MemorySegment generator, int chunkX, int chunkZ) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment outPtr = arena.allocate(ValueLayout.ADDRESS);
            MemorySegment outLen = arena.allocate(ValueLayout.JAVA_LONG);
            int status = (int) GENERATOR_GENERATE.invokeExact(generator, chunkX, chunkZ, outPtr, outLen);

            if (status != 0) {
                throw new IllegalStateException("kryon_generator_generate returned " + status);
            }

            MemorySegment data = outPtr.get(ValueLayout.ADDRESS, 0);
            long length = outLen.get(ValueLayout.JAVA_LONG, 0);
            return new GeneratedChunk(data, length);
        } catch (Throwable throwable) {
            throw new RuntimeException("Failed to call kryon_generator_generate", throwable);
        }
    }

    public static final class GeneratedChunk implements AutoCloseable {
        private final MemorySegment data;
        private final long length;
        private boolean freed;

        private GeneratedChunk(MemorySegment data, long length) {
            this.data = data;
            this.length = length;
        }

        public MemorySegment data() {
            if (this.freed) {
                throw new IllegalStateException("Generated chunk buffer has been freed");
            }
            return this.data.reinterpret(this.length);
        }

        public long length() {
            return this.length;
        }

        @Override
        public void close() {
            if (this.freed) {
                return;
            }

            try {
                FREE_BUFFER.invokeExact(this.data, this.length);
            } catch (Throwable throwable) {
                throw new RuntimeException("Failed to free generated chunk buffer", throwable);
            }
            this.freed = true;
        }
    }

    private static byte[] writeBlockStateRegistry() {
        List<RegisteredBlockState> states = new ArrayList<>();

        for (Block block : BuiltInRegistries.BLOCK) {
            for (BlockState state : block.getStateDefinition().getPossibleStates()) {
                states.add(new RegisteredBlockState(Block.getId(state), stateName(block, state)));
            }
        }
        states.sort(Comparator.comparingInt(RegisteredBlockState::id));

        ByteArrayOutputStream output = new ByteArrayOutputStream();
        writeU32(output, states.size());
        for (RegisteredBlockState state : states) {
            byte[] name = state.name().getBytes(StandardCharsets.UTF_8);
            writeU32(output, state.id());
            writeU32(output, name.length);
            output.writeBytes(name);
        }
        return output.toByteArray();
    }

    private static byte[] writeDimensionType(DimensionType dimensionType, HolderLookup.Provider registries) {
        return DimensionType.DIRECT_CODEC
            .encodeStart(RegistryOps.create(JsonOps.INSTANCE, registries), dimensionType)
            .getOrThrow(IllegalStateException::new)
            .toString()
            .getBytes(StandardCharsets.UTF_8);
    }

    private static byte[] writeBiomeRegistry(final RegistryAccess registryAccess) {
        final Registry<Biome> biomeRegistry = registryAccess.lookupOrThrow(Registries.BIOME);
        final List<RegisteredBiome> biomes = new ArrayList<>();

        for (final Biome biome : biomeRegistry) {
            final int id = biomeRegistry.getId(biome);
            final String key = biomeRegistry.getKey(biome).toString();

            if (id < 0) {
                throw new IllegalStateException("Biome is absent from its registry: " + key);
            }

            biomes.add(new RegisteredBiome(id, key));
        }

        biomes.sort(Comparator.comparingInt(RegisteredBiome::id));

        final ByteArrayOutputStream output = new ByteArrayOutputStream();
        writeU32(output, biomes.size());

        for (final RegisteredBiome biome : biomes) {
            final byte[] key = biome.key().getBytes(StandardCharsets.UTF_8);
            writeU32(output, biome.id());
            writeU32(output, key.length);
            output.writeBytes(key);
        }

        return output.toByteArray();
    }

    private record RegisteredBiome(int id, String key) { }

    private static String stateName(Block block, BlockState state) {
        StringBuilder name = new StringBuilder(BuiltInRegistries.BLOCK.getKey(block).toString());
        List<Property<?>> properties = new ArrayList<>(state.getProperties());
        properties.sort(Comparator.comparing(Property::getName));

        if (!properties.isEmpty()) {
            name.append('[');
            for (int index = 0; index < properties.size(); index++) {
                if (index != 0) {
                    name.append(',');
                }
                appendProperty(name, state, properties.get(index));
            }
            name.append(']');
        }

        return name.toString();
    }

    private static <T extends Comparable<T>> void appendProperty(StringBuilder output, BlockState state, Property<T> property) {
        output.append(property.getName())
            .append('=')
            .append(property.getName(state.getValue(property)));
    }

    private static void writeU32(ByteArrayOutputStream output, int value) {
        output.write(value);
        output.write(value >>> 8);
        output.write(value >>> 16);
        output.write(value >>> 24);
    }

    private record RegisteredBlockState(int id, String name) {
    }

    private KryonNative() {
    }
}
