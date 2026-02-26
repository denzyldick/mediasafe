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

      <div class="d-flex justify-center mb-6 mt-2">
         <v-btn-toggle v-model="mode" color="primary" mandatory variant="outlined" divided>
            <v-btn value="host">Host</v-btn>
            <v-btn value="join">Join</v-btn>
         </v-btn-toggle>
      </div>

      <div class="d-flex justify-center mb-6" v-if="uuid && mode === 'host'">
        <v-sheet color="white" rounded="xl" class="pa-4" elevation="4">
             <qrcode-vue :value="uuid" :size="200" level="H" />
        </v-sheet>
      </div>

      <div class="text-caption text-medium-emphasis mb-1">Manual Connection Passphrase</div>
      <div class="d-flex justify-center flex-wrap gap-2 mb-6" v-if="passphrase.length > 0 && mode === 'host'">
        <v-chip
          v-for="(word, index) in passphrase"
          :key="index"
          color="primary"
          variant="flat"
          class="font-weight-medium mx-1"
        >
          {{ word }}
        </v-chip>
      </div>

      <div class="d-flex justify-center mb-6" v-if="mode === 'join'">
         <v-text-field
           v-model="joinPassphrase"
           label="Enter 4-word Passphrase"
           variant="outlined"
           density="compact"
           hide-details
           class="mb-2"
           @keyup.enter="joinWebRTC"
         ></v-text-field>
         <v-btn color="primary" @click="joinWebRTC" class="ml-2 mt-1">Join</v-btn>
      </div>

      <div class="text-caption text-medium-emphasis mb-1 text-center" v-if="connectionStatus">
         <v-progress-circular v-if="!isConnected" indeterminate color="primary" size="20" width="2" class="mr-2"></v-progress-circular>
         <v-icon v-else color="success" class="mr-2">mdi-check-circle</v-icon>
         {{ connectionStatus }}
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
    mode: "host",
    uuid: "",
    passphrase: [],
    joinPassphrase: "",
    connectionStatus: "",
    isConnected: false,
  }),
  watch: {
    dialog(val) {
      if (val) {
        this.initialize();
      }
    },
    mode(newMode) {
       if (newMode === 'host' && !this.uuid) {
           this.initialize();
       } else if (newMode === 'join') {
           this.connectionStatus = "";
       }
    }
  },
  methods: {
    async initialize() {
        this.connectionStatus = "Generating secure pair key...";
        
        try {
          const codes = await invoke("generate_pairing_codes");
          this.uuid = codes.uuid;
          this.passphrase = codes.passphrase;
          
          const roomId = await invoke("hash_pairing_code", { input: this.uuid });
          this.listen(roomId);
        } catch (error) {
           console.error("Failed to generate code", error);
           this.connectionStatus = "Pairing Error.";
        }
    },
    async listen(roomId) {
      this.connectionStatus = "Waiting for partner device to scan or type phrase...";
      try {
          // Hardcoded external signaling server for testing, will replace with prod later
          const signalingUrl = "ws://localhost:3000";
          await invoke("start_webrtc_session", {
              roomId: roomId,
              isInitiator: true,
              signalingUrl: signalingUrl
          });
          this.connectionStatus = "Signaling channel requested. Awaiting WebRTC connection.";
          // TODO: Listen for WebRTC event complete to actually set isConnected = true
      } catch (error) {
          console.error("WebRTC Error", error);
          this.connectionStatus = "Error connecting to Signaling server.";
      }
    },
    async joinWebRTC() {
        if (!this.joinPassphrase) return;
        this.connectionStatus = "Joining room...";
        try {
           const roomId = await invoke("hash_pairing_code", { input: this.joinPassphrase });
           const signalingUrl = "ws://localhost:3000";
           await invoke("start_webrtc_session", {
              roomId: roomId,
              isInitiator: false,
              signalingUrl: signalingUrl
           });
           this.connectionStatus = "Signaling channel requested. Awaiting WebRTC Receiver connection.";
        } catch(error) {
           console.error("WebRTC Join Error", error);
           this.connectionStatus = "Error joining via Signaling server.";
        }
    }
  },
};
</script>

<style scoped>
.glass-panel {
    border-radius: 24px !important; 
}
</style>
