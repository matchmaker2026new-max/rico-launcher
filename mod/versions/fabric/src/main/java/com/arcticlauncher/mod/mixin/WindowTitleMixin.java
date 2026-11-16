package com.arcticlauncher.mod.mixin;

import com.mojang.blaze3d.platform.Window;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

	/** The window's title names RICO, so the taskbar tells it from a plain Minecraft. */
@Mixin(Window.class)
abstract class WindowTitleMixin {
	//#if MC >= 1.15
	@ModifyVariable(method = "setTitle", at = @At("HEAD"), argsOnly = true, require = 0)
	private String arctic$title(String title) {
		if (System.getProperty("arctic.launcher") == null || title == null) {
			return title;
		}
		String branded = title.replace("RICO", "RICO");
		return branded.contains("RICO") ? branded : branded + " - RICO";
	}
	//#endif
}
