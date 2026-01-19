<template>
  <v-dialog v-model="dialog" width="auto" scrim="black" transition="dialog-bottom-transition">
    <template v-slot:activator="{ props }">
      <v-btn icon v-bind="props" color="primary" class="glass-panel" variant="text">
        <v-icon>mdi-plus</v-icon>
      </v-btn>
    </template>
    
    <v-card class="glass-panel pa-6 text-center" min-width="350" max-width="400">
      <div class="text-h5 font-weight-bold mb-2">Connect New Device</div>
      <div class="text-body-2 text-medium-emphasis mb-6">
        Scan this code with the MediaSafe app on your other device to connect.
      </div>

      <div class="d-flex justify-center mb-6">
        <v-sheet color="white" rounded="xl" class="pa-4" elevation="4">
             <qrcode-vue :value="connectionUrl" :size="200" level="H" />
        </v-sheet>
      </div>

      <div class="text-caption text-medium-emphasis mb-1">Manual Connection URL</div>
      <v-chip color="primary" variant="flat" class="mb-6 font-weight-medium">
        {{ connectionUrl }}
      </v-chip>

      <div class="d-flex align-center justify-center text-caption text-medium-emphasis">
         <v-progress-circular indeterminate color="primary" size="20" width="2" class="mr-2"></v-progress-circular>
         Listening for devices...
      </div>

      <v-card-actions class="justify-center mt-4">
        <v-btn color="white" variant="text" @click="dialog = false">Close</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script>
import { invoke } from "@tauri-apps/api/core";
import QrcodeVue from 'qrcode.vue'

export default {
  name: "Connect",
  components: {
    QrcodeVue,
  },
  data: () => ({
    dialog: false,
    ip: "Loading...",
    port: "9489",
  }),
  computed: {
      connectionUrl() {
          return `http://${this.ip}:${this.port}`;
      }
  },
  watch: {
    dialog(val) {
      if (val) {
        this.initialize();
      }
    },
  },
  methods: {
    async initialize() {
        this.ip = await invoke("get_ip");
        this.listen();
    },
    listen() {
      console.log("Listening for incomming connection");
      invoke("listen_for_incomming_connect").then((response) => {
        console.log("Connection established", response);
        this.dialog = false;
        // Optionally emit event to refresh device list
      });
    },
  },
};
</script>

<style scoped>
.glass-panel {
    border-radius: 24px !important; 
}
</style>
