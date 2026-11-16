package com.arcticlauncher.mod;

import com.arcticlauncher.client.ArcticClient;
import net.fabricmc.api.ClientModInitializer;
//#if MC >= 1.18
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
//#else
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
//#endif

/** RICO Client for Minecraft 26.3: starts the shared core on this version. */
public final class ArcticMod implements ClientModInitializer {
	public static final String ID = "arctic";
	//#if MC >= 1.18
	public static final Logger LOG = LoggerFactory.getLogger("RICO");
	//#else
	public static final Logger LOG = LogManager.getLogger("RICO");
	//#endif

	@Override
	public void onInitializeClient() {
		ArcticClient.init(new FabricPlatform());
		com.arcticlauncher.mod.svc.Greeter.listenToFabricApi();
		//#if MC >= 1.13
		com.arcticlauncher.client.replay.Replays.backend(com.arcticlauncher.mod.replay.ReplayPlayback.INSTANCE);
		//#endif
		SelfTest.maybeStart();
	}
}
