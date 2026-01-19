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
    defaultTheme: 'dark',
    themes: {
      dark: {
        dark: true,
        colors: {
          background: '#0f172a', // Deep Indigo/Slate
          surface: '#1e293b',
          primary: '#2979FF',    // Electric Blue
          secondary: '#00E676',  // Neon Mint
          error: '#FF5252',
          info: '#2196F3',
          success: '#4CAF50',
          warning: '#FB8C00',
        }
      }
    }
  },
  defaults: {
    global: {
      fontFamily: 'Outfit, sans-serif'
    },
    VBtn: {
      rounded: 'lg',
      variant: 'flat'
    },
    VCard: {
      rounded: 'xl',
      elevation: 0,
      color: 'rgba(30, 41, 59, 0.7)',
      class: 'glass-panel'
    },
    VDialog: {
      cardProps: {
        rounded: 'xl',
        class: 'glass-panel'
      }
    },
    VTextField: {
      variant: 'solo-filled',
      rounded: 'lg'
    }
  }
});
let app = createApp(App).use(vuetify).mount("#app");
