package com.arcticlauncher.client.menu;

import com.arcticlauncher.client.ArcticClient;
import com.arcticlauncher.client.gfx.Draw;
import com.arcticlauncher.client.gfx.Gfx;

/** Arctic's title artwork, shared by the native menu and the page preview. */
public final class TitleBranding {
	private static final int ICON = 24;
	private static final float WORDMARK_SCALE = 3f;

	private TitleBranding() {}

	public static void draw(Gfx g, int width, int height) {
		int top = Math.max(6, height / 4 - 36);
		g.texture("icon", width / 2 - ICON / 2, top, ICON, ICON, 0, 0, 256, 256, 256, 256);
		int wordTop = top + ICON + 6;
		int wordWidth = Math.round(g.textWidth("RICO") * WORDMARK_SCALE);
		int wordHeight = Math.round(8 * WORDMARK_SCALE);
		Draw.big(g, "RICO", width / 2 - wordWidth / 2, wordTop, WORDMARK_SCALE, 0xFFFFFFFF);
		Draw.centered(g, "Minecraft " + ArcticClient.platform().minecraftVersion(), width / 2,
				wordTop + wordHeight + 5, ArcticClient.style().accent, true);
	}
}
