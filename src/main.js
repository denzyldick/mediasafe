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
          background: "#09090b", // Deep Zinc
          surface: "#09090b", 
          primary: "#3f3f46", // Neutral Zinc-600
          secondary: "#27272a", // Zinc-800
          accent: "#ffffff",
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
      border: "1px solid rgba(255, 255, 255, 0.1)",
      color: "#09090b",
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
