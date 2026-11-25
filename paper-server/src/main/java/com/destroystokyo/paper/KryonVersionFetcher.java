package com.destroystokyo.paper;

import com.destroystokyo.paper.util.VersionFetcher;
import io.papermc.paper.ServerBuildInfo;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;
import net.kyori.adventure.text.format.TextDecoration;
import net.kyori.adventure.text.logger.slf4j.ComponentLogger;
import org.apache.logging.log4j.LogManager;
import org.checkerframework.checker.nullness.qual.Nullable;

public class KryonVersionFetcher implements VersionFetcher {
    private static final ComponentLogger LOGGER = ComponentLogger.logger(LogManager.getRootLogger().getName());
    private static final ServerBuildInfo BUILD_INFO = ServerBuildInfo.buildInfo();

    @Override
    public long getCacheTime() {
        return -1;
    }

    @Override
    public Component getVersionMessage() {
        final Component message = Component.text(
            "You are running a development build of Kryon; automatic update checks are not available yet.",
            NamedTextColor.YELLOW
        );
        final @Nullable Component history = this.getHistory();
        return history == null ? message : Component.textOfChildren(message, Component.newline(), history);
    }

    public static void getUpdateStatusStartupMessage() {
        LOGGER.warn(Component.text(
            "You are running Kryon "
                + BUILD_INFO.asString(ServerBuildInfo.StringRepresentation.VERSION_SIMPLE)
                + "; automatic update checks are not available yet.",
            NamedTextColor.YELLOW
        ));
    }

    private @Nullable Component getHistory() {
        final VersionHistoryManager.@Nullable VersionData data = VersionHistoryManager.INSTANCE.getVersionData();
        if (data == null || data.getOldVersion() == null) {
            return null;
        }
        return Component.text("Previous version: " + data.getOldVersion(), NamedTextColor.GRAY, TextDecoration.ITALIC);
    }
}
