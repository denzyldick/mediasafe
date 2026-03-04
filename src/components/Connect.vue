<template>
  <v-dialog v-model="dialog" width="auto" scrim="black" transition="dialog-bottom-transition">
    <template v-slot:activator="{ props }">
      <v-btn
        prepend-icon="mdi-plus"
        v-bind="props"
        variant="outlined"
        color="#a1a1aa"
        class="text-none border-subtle"
      >
        Add Device
      </v-btn>
    </template>
    
    <v-card class="border-subtle pa-6 text-center" color="background" min-width="350" max-width="400">
      <div class="text-h5 font-weight-bold text-zinc-primary mb-2">Link Device</div>
      <div class="text-body-2 text-zinc-muted mb-6">
        Sync your library across your own hardware.
      </div>

      <div class="d-flex justify-center mb-6 mt-2">
         <v-btn-toggle v-model="mode" color="#e4e4e7" mandatory variant="outlined" divided class="border-subtle">
            <v-btn value="host" class="text-none">Host</v-btn>
            <v-btn value="join" class="text-none">Join</v-btn>
         </v-btn-toggle>
      </div>

      <div class="d-flex justify-center mb-6" v-if="uuid && mode === 'host'">
        <v-sheet color="white" rounded="lg" class="pa-4 shadow-lg border-subtle">
             <qrcode-vue :value="uuid" :size="180" level="H" />
        </v-sheet>
      </div>

      <div class="text-caption text-zinc-muted mb-2 uppercase tracking-widest" v-if="mode === 'host'">Manual Phrase</div>
      <div class="d-flex justify-center flex-wrap gap-2 mb-6" v-if="passphrase.length > 0 && mode === 'host'">
        <v-chip
          v-for="(word, index) in passphrase"
          :key="index"
          color="#27272a"
          variant="flat"
          class="font-weight-medium mx-1 text-zinc-secondary border-subtle"
          size="small"
        >
          {{ word }}
        </v-chip>
      </div>

      <div class="d-flex justify-center mb-6 flex-column" v-if="mode === 'join'">
         <v-text-field
           v-model="joinPassphrase"
           placeholder="Enter 4-word phrase"
           variant="solo-filled"
           bg-color="rgba(255,255,255,0.03)"
           density="comfortable"
           hide-details
           class="mb-4 text-center"
           @keyup.enter="joinWebRTC"
         ></v-text-field>
         <v-btn color="white" variant="flat" @click="joinWebRTC" class="text-none" block>Link Device</v-btn>
      </div>

      <div class="text-caption text-zinc-muted mb-1 text-center py-2" v-if="connectionStatus">
         <v-progress-circular v-if="!isConnected" indeterminate color="white" size="16" width="2" class="mr-2 opacity-50"></v-progress-circular>
         <v-icon v-else color="success" size="16" class="mr-2">mdi-check-circle-outline</v-icon>
         {{ connectionStatus }}
      </div>

      <v-divider class="opacity-5 my-4"></v-divider>

      <v-card-actions class="justify-center">
        <v-btn color="#71717a" variant="text" class="text-none" @click="dialog = false">Cancel</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.uppercase {
  text-transform: uppercase;
}
.tracking-widest {
  letter-spacing: 0.1em;
}
</style>

<script>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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
    unlisten: null,
  }),
  watch: {
    async dialog(val) {
      if (val) {
        this.unlisten = await listen("webrtc-state", (event) => {
          this.connectionStatus = event.payload;
          if (event.payload === "Connected" || event.payload === "connected") {
            this.isConnected = true;
          }
          if (event.payload === "disconnected" || event.payload === "failed") {
            this.isConnected = false;
          }
        });
        this.initialize();
      } else {
        if (this.unlisten) {
          this.unlisten();
          this.unlisten = null;
        }
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
          // Production Signaling Server
          const signalingUrl = "wss://mediasafe.denzyl.io";
          await invoke("start_webrtc_session", {
              room_id: roomId,
              is_initiator: true,
              signaling_url: signalingUrl
          });
          this.connectionStatus = "Signaling channel requested. Awaiting WebRTC connection.";
          // TODO: Listen for WebRTC event complete to actually set isConnected = true
      } catch (error) {
          console.error("WebRTC Error", error);
          this.connectionStatus = "Error connecting: " + error;
      }
    },
    async joinWebRTC() {
        if (!this.joinPassphrase) return;
        this.connectionStatus = "Joining room...";
        try {
           const roomId = await invoke("hash_pairing_code", { input: this.joinPassphrase });
           const signalingUrl = "wss://mediasafe.denzyl.io";
           await invoke("start_webrtc_session", {
              room_id: roomId,
              is_initiator: false,
              signaling_url: signalingUrl
           });
           this.connectionStatus = "Signaling channel requested. Awaiting WebRTC Receiver connection.";
        } catch(error) {
           console.error("WebRTC Join Error", error);
           this.connectionStatus = "Error joining: " + error;
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
