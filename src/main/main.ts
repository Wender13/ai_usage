import { createApp } from "vue";
import { initI18n } from "../i18n";
import App from "./App.vue";

// Loads the language first so the window never flashes texts in the wrong one.
void initI18n().finally(() => createApp(App).mount("#app"));
