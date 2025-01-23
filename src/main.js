import { createApp } from "vue";
import App from "./App.vue";
import "vuetify/styles";
import "@mdi/font/css/materialdesignicons.css";

import * as components from "vuetify/components";
//import { directives } from "vuetify/directives";
import { createVuetify } from "vuetify/dist/vuetify.js";

const vuetify = createVuetify({
  components,
 // directives
});
let app =createApp(App).use(vuetify).mount("#app");
