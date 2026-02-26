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
          background: "#09090b", // Ultra-Deep Zinc/Charcoal
          surface: "#18181b", // Elevated Zinc
          primary: "#8b5cf6", // Vibrant Violet for modern pop
          secondary: "#14b8a6", // Sleek Teal/Emerald contrast
          accent: "#ec4899", // Neon Pink for micro interactions
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
      fontFamily: "Outfit, sans-serif",
    },
    VBtn: {
      rounded: "lg",
      variant: "flat",
    },
    VCard: {
      rounded: "xl",
      elevation: 0,
      color: "rgba(24, 24, 27, 0.7)", // Matches new surface color with transparency
      class: "glass-panel",
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
