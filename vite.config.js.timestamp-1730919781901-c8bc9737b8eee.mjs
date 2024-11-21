// vite.config.js
import { defineConfig } from "file:///home/denzyl/Projects/mediasafe/node_modules/vite/dist/node/index.js";
import vue from "file:///home/denzyl/Projects/mediasafe/node_modules/@vitejs/plugin-vue/dist/index.mjs";
import { internalIpV4 } from "file:///home/denzyl/Projects/mediasafe/node_modules/internal-ip/index.js";
var mobile = !!/android|ios/.exec(process.env.TAURI_ENV_PLATFORM);
var vite_config_default = defineConfig(async () => ({
  plugins: [vue()],
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: mobile ? "0.0.0.0" : false,
    hmr: mobile ? {
      protocol: "ws",
      host: await internalIpV4(),
      port: 1421
    } : void 0,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"]
    }
  }
}));
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcuanMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvaG9tZS9kZW56eWwvUHJvamVjdHMvbWVkaWFzYWZlXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ZpbGVuYW1lID0gXCIvaG9tZS9kZW56eWwvUHJvamVjdHMvbWVkaWFzYWZlL3ZpdGUuY29uZmlnLmpzXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ltcG9ydF9tZXRhX3VybCA9IFwiZmlsZTovLy9ob21lL2Rlbnp5bC9Qcm9qZWN0cy9tZWRpYXNhZmUvdml0ZS5jb25maWcuanNcIjtpbXBvcnQgeyBkZWZpbmVDb25maWcgfSBmcm9tIFwidml0ZVwiO1xuaW1wb3J0IHZ1ZSBmcm9tIFwiQHZpdGVqcy9wbHVnaW4tdnVlXCI7XG5pbXBvcnQgeyBpbnRlcm5hbElwVjQgfSBmcm9tIFwiaW50ZXJuYWwtaXBcIjtcblxuY29uc3QgbW9iaWxlID0gISEvYW5kcm9pZHxpb3MvLmV4ZWMocHJvY2Vzcy5lbnYuVEFVUklfRU5WX1BMQVRGT1JNKTtcblxuLy8gaHR0cHM6Ly92aXRlanMuZGV2L2NvbmZpZy9cbmV4cG9ydCBkZWZhdWx0IGRlZmluZUNvbmZpZyhhc3luYyAoKSA9PiAoe1xuICBwbHVnaW5zOiBbdnVlKCldLFxuXG4gIC8vIFZpdGUgb3B0aW9ucyB0YWlsb3JlZCBmb3IgVGF1cmkgZGV2ZWxvcG1lbnQgYW5kIG9ubHkgYXBwbGllZCBpbiBgdGF1cmkgZGV2YCBvciBgdGF1cmkgYnVpbGRgXG4gIC8vXG4gIC8vIDEuIHByZXZlbnQgdml0ZSBmcm9tIG9ic2N1cmluZyBydXN0IGVycm9yc1xuICBjbGVhclNjcmVlbjogZmFsc2UsXG4gIC8vIDIuIHRhdXJpIGV4cGVjdHMgYSBmaXhlZCBwb3J0LCBmYWlsIGlmIHRoYXQgcG9ydCBpcyBub3QgYXZhaWxhYmxlXG4gIHNlcnZlcjoge1xuICAgIHBvcnQ6IDE0MjAsXG4gICAgc3RyaWN0UG9ydDogdHJ1ZSxcbiAgICBob3N0OiBtb2JpbGUgPyBcIjAuMC4wLjBcIiA6IGZhbHNlLFxuICAgIGhtcjogbW9iaWxlXG4gICAgICA/IHtcbiAgICAgICAgICBwcm90b2NvbDogXCJ3c1wiLFxuICAgICAgICAgIGhvc3Q6IGF3YWl0IGludGVybmFsSXBWNCgpLFxuICAgICAgICAgIHBvcnQ6IDE0MjEsXG4gICAgICAgIH1cbiAgICAgIDogdW5kZWZpbmVkLFxuICAgIHdhdGNoOiB7XG4gICAgICAvLyAzLiB0ZWxsIHZpdGUgdG8gaWdub3JlIHdhdGNoaW5nIGBzcmMtdGF1cmlgXG4gICAgICBpZ25vcmVkOiBbXCIqKi9zcmMtdGF1cmkvKipcIl0sXG4gICAgfSxcbiAgfSxcbn0pKTtcbiJdLAogICJtYXBwaW5ncyI6ICI7QUFBK1EsU0FBUyxvQkFBb0I7QUFDNVMsT0FBTyxTQUFTO0FBQ2hCLFNBQVMsb0JBQW9CO0FBRTdCLElBQU0sU0FBUyxDQUFDLENBQUMsY0FBYyxLQUFLLFFBQVEsSUFBSSxrQkFBa0I7QUFHbEUsSUFBTyxzQkFBUSxhQUFhLGFBQWE7QUFBQSxFQUN2QyxTQUFTLENBQUMsSUFBSSxDQUFDO0FBQUE7QUFBQTtBQUFBO0FBQUEsRUFLZixhQUFhO0FBQUE7QUFBQSxFQUViLFFBQVE7QUFBQSxJQUNOLE1BQU07QUFBQSxJQUNOLFlBQVk7QUFBQSxJQUNaLE1BQU0sU0FBUyxZQUFZO0FBQUEsSUFDM0IsS0FBSyxTQUNEO0FBQUEsTUFDRSxVQUFVO0FBQUEsTUFDVixNQUFNLE1BQU0sYUFBYTtBQUFBLE1BQ3pCLE1BQU07QUFBQSxJQUNSLElBQ0E7QUFBQSxJQUNKLE9BQU87QUFBQTtBQUFBLE1BRUwsU0FBUyxDQUFDLGlCQUFpQjtBQUFBLElBQzdCO0FBQUEsRUFDRjtBQUNGLEVBQUU7IiwKICAibmFtZXMiOiBbXQp9Cg==
