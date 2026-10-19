import { createApp } from "vue";
import { initI18n } from "../i18n";
import OverlayApp from "./OverlayApp.vue";

// Loads the language first so the window never flashes texts in the wrong one.
void initI18n().finally(() => createApp(OverlayApp).mount("#app"));
