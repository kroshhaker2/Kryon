package com.destroystokyo.paper;

/**
 * Compatibility shim for Paper's Minecraft source patch.
 *
 * @deprecated use {@link KryonVersionFetcher}
 */
@Deprecated
public final class PaperVersionFetcher extends KryonVersionFetcher {
    public static void getUpdateStatusStartupMessage() {
        KryonVersionFetcher.getUpdateStatusStartupMessage();
    }
}
