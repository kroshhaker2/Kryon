package io.papermc.paper.kryon;

import com.mojang.serialization.MapCodec;
import io.netty.buffer.Unpooled;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.server.level.WorldGenRegion;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.NoiseColumn;
import net.minecraft.world.level.StructureManager;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeManager;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.dimension.DimensionType;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.levelgen.RandomState;
import net.minecraft.world.level.levelgen.blending.Blender;
import net.minecraft.world.level.levelgen.densityfunction.SamplerContext;
import org.jspecify.annotations.Nullable;

import java.lang.foreign.MemorySegment;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.EnumSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.CompletableFuture;

public final class KryonChunkGenerator extends ChunkGenerator {
    private final MemorySegment generator;
    private final DimensionType dimensionType;

    public KryonChunkGenerator(BiomeSource biomeSource, MemorySegment kryonGenerator, DimensionType dimensionType) {
        super(biomeSource);
        this.generator = kryonGenerator;
        this.dimensionType = dimensionType;
    }

    @Override
    protected MapCodec<? extends ChunkGenerator> codec() {
        return null;
    }

    @Override
    public void spawnOriginalMobs(final WorldGenRegion worldGenRegion) {

    }

    @Override
    public int getGenDepth() {
        return this.dimensionType.height();
    }

    @Override
    public CompletableFuture<ChunkAccess> buildTerrain(final ChunkAccess chunk, final Blender blender, final RandomState randomState, final StructureManager structureManager, final BiomeManager biomeManager, @Nullable final WorldGenRegion carverBiomeRegion, final Set<Holder<Biome>> possibleBiomes) {
        try (KryonNative.GeneratedChunk generatedChunk = KryonNative.generateChunk(this.generator, chunk.getPos().x(), chunk.getPos().z())) {
            ByteBuffer input = generatedChunk.data().asByteBuffer().order(ByteOrder.LITTLE_ENDIAN);
            requireRemaining(input, Integer.BYTES);
            int sectionCount = input.getInt();
            if (sectionCount < 0 || sectionCount > chunk.getSectionsCount()) {
                throw new IllegalStateException("Invalid native section count: " + sectionCount);
            }

            boolean[] seenSections = new boolean[chunk.getSectionsCount()];
            List<SerializedSection> sections = new ArrayList<>(sectionCount);

            for (int index = 0; index < sectionCount; index++) {
                requireRemaining(input, Integer.BYTES * 2);
                int sectionY = input.getInt();
                int blockStatesLength = input.getInt();

                if (blockStatesLength < 0 || blockStatesLength > input.remaining()) {
                    throw new IllegalStateException("Invalid native block state length: " + blockStatesLength);
                }

                int sectionIndex = chunk.getSectionIndexFromSectionY(sectionY);
                if (sectionIndex < 0 || sectionIndex >= chunk.getSectionsCount()) {
                    throw new IllegalStateException("Native section is outside the chunk: " + sectionY);
                }
                if (seenSections[sectionIndex]) {
                    throw new IllegalStateException("Native terrain contains a duplicate section: " + sectionY);
                }
                seenSections[sectionIndex] = true;

                byte[] blockStates = new byte[blockStatesLength];
                input.get(blockStates);
                sections.add(new SerializedSection(sectionIndex, blockStates));
            }

            if (input.hasRemaining()) {
                throw new IllegalStateException("Unexpected bytes after native terrain");
            }

            for (SerializedSection serialized : sections) {
                LevelChunkSection section = chunk.getSections()[serialized.index()];
                FriendlyByteBuf blockStates = new FriendlyByteBuf(Unpooled.wrappedBuffer(serialized.blockStates()));
                try {
                    section.getStates().read(blockStates);
                    if (blockStates.isReadable()) {
                        throw new IllegalStateException("Unexpected bytes after native block states");
                    }
                } finally {
                    blockStates.release();
                }
                section.recalcBlockCounts();
            }

            Heightmap.primeHeightmaps(
                chunk,
                EnumSet.of(Heightmap.Types.OCEAN_FLOOR_WG, Heightmap.Types.WORLD_SURFACE_WG)
            );
        }

        return CompletableFuture.completedFuture(chunk);
    }

    private static void requireRemaining(ByteBuffer input, int required) {
        if (input.remaining() < required) {
            throw new IllegalStateException("Truncated native terrain data");
        }
    }

    private record SerializedSection(int index, byte[] blockStates) {
    }

    @Override
    public int getSeaLevel() {
        return 0;
    }

    @Override
    public int getMinY() {
        return this.dimensionType.minY();
    }

    @Override
    public int getBaseHeight(final int x, final int z, final Heightmap.Types type, final LevelHeightAccessor heightAccessor, final RandomState randomState) {
        return 0;
    }

    @Override
    public NoiseColumn getBaseColumn(final int x, final int z, final LevelHeightAccessor heightAccessor, final RandomState randomState) {
        return null;
    }

    @Override
    public void addDebugScreenInfo(final List<String> result, final RandomState randomState, final BlockPos feetPos, final SamplerContext samplerContext) {

    }
}
