import { createApp } from "vue";
import App from "./App.vue";
import "vuetify/styles";
import "@mdi/font/css/materialdesignicons.css";

import * as components from "vuetify/components";
//import { directives } from "vuetify/directives";
import { createVuetify } from "vuetify/dist/vuetify.js";

const vuetify = createVuetify({
  components,
  theme: {
    defaultTheme: "light",
    themes: {
      dark: {
        dark: true,
        colors: {
          background: "#09090b", // Zinc-950
          surface: "#09090b", 
          primary: "#27272a", // Zinc-800
          secondary: "#18181b", // Zinc-900
          accent: "#d4d4d8", // Zinc-300
          error: "#ef4444",
          info: "#3b82f6",
          success: "#22c55e",
          warning: "#f59e0b",
        },
      },
    },
  },
  defaults: {
    global: {
      fontFamily: "'Inter', sans-serif",
    },
    VBtn: {
      rounded: "md",
      variant: "flat",
      class: "text-none font-weight-medium",
    },
    VCard: {
      rounded: "lg",
      elevation: 0,
      color: "#18181b", // Zinc-900
      class: "border-subtle",
    },
    VDialog: {
      cardProps: {
        rounded: "xl",
        class: "glass-panel",
      },
    },
    VTextField: {
      variant: "solo-filled",
      rounded: "lg",
    },
  },
});
let app = createApp(App).use(vuetify).mount("#app");
