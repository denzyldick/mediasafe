<template>
  <v-dialog v-model="dialog" width="auto" scrim="black" transition="dialog-bottom-transition">
    <template v-slot:activator="{ props }">
      <v-btn
        prepend-icon="mdi-plus"
        v-bind="props"
        variant="flat"
        class="text-none siegu-btn"
      >
        Add Device
      </v-btn>
    </template>
    
    <v-card class="border-subtle pa-6 text-center" color="#000000" rounded="xl" min-width="350" max-width="400">
      <div class="text-h5 font-weight-bold text-zinc-primary mb-2">Link Device</div>
      <div class="text-body-2 text-zinc-muted mb-6">
        Sync your library across your own hardware.
      </div>

      <div class="d-flex justify-center mb-6 mt-2">
         <v-btn-toggle v-model="mode" color="white" mandatory variant="outlined" divided class="border-subtle siegu-toggle">
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
          color="#18181b"
          variant="flat"
          class="font-weight-medium mx-1 text-zinc-secondary border-subtle"
          size="small"
        >
          {{ word }}
        </v-chip>
      </div>

      <div class="d-flex justify-center mb-6 flex-column ga-4" v-if="mode === 'join'">
         <v-text-field
           v-model="joinPassphrase"
           placeholder="Enter 4-word phrase"
           variant="solo-filled"
           bg-color="#18181b"
           density="comfortable"
           hide-details
           flat
           rounded="lg"
           class="text-center siegu-field"
           @keyup.enter="joinWebRTC"
         ></v-text-field>
         <v-btn variant="flat" @click="joinWebRTC" class="text-none siegu-btn" block>Link Device</v-btn>
      </div>

      <div class="text-caption text-zinc-muted mb-1 text-center py-2" v-if="connectionStatus">
         <v-progress-circular v-if="!isConnected" indeterminate color="white" size="16" width="2" class="mr-2 opacity-50"></v-progress-circular>
         <v-icon v-else color="success" size="16" class="mr-2">mdi-check-circle-outline</v-icon>
         {{ connectionStatus }}
      </div>

      <v-divider class="opacity-10 my-4"></v-divider>

      <v-card-actions class="justify-center">
        <v-btn color="#a1a1aa" variant="text" class="text-none" @click="dialog = false">Cancel</v-btn>
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

.siegu-btn {
  background: #18181b !important;
  border: 1px solid rgba(255, 255, 255, 0.2) !important;
  border-radius: 12px !important;
  color: #ffffff !important;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1) !important;
}

.siegu-btn:hover {
  background: #27272a !important;
  border-color: rgba(255, 255, 255, 0.4) !important;
  transform: translateY(-1px);
}

.siegu-toggle {
  background: #18181b !important;
  border-radius: 12px !important;
}

.siegu-toggle .v-btn {
  border: none !important;
}

.siegu-toggle .v-btn--active {
  background: #27272a !important;
}

.siegu-field :deep(.v-field) {
  border: 1px solid rgba(255, 255, 255, 0.2) !important;
  border-radius: 12px !important;
}

.siegu-field :deep(.v-field--focused) {
  border-color: rgba(255, 255, 255, 0.4) !important;
  background: #27272a !important;
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
          const codes = await invoke("server::generate_pairing_codes");
          this.uuid = codes.uuid;
          this.passphrase = codes.passphrase;
          
          const roomId = await invoke("server::hash_pairing_code", { input: this.uuid });
          this.listen(roomId);
        } catch (error) {
           console.error("Failed to generate code", error);
           this.connectionStatus = "Pairing Error.";
        }
    },
    async listen(roomId) {
      this.connectionStatus = "Waiting for partner device to scan or type phrase...";
      const args = {
          room_id: roomId,
          is_initiator: true,
          signaling_url: "wss://siegu.denzyl.io"
      };
      console.log("Invoking start_webrtc_session with:", args);
      try {
          await invoke("start_webrtc_session", args);
          this.connectionStatus = "Signaling channel requested. Awaiting WebRTC connection.";
      } catch (error) {
          console.error("WebRTC Error:", error);
          this.connectionStatus = "Error connecting: " + error;
      }
    },
    async joinWebRTC() {
        if (!this.joinPassphrase) return;
        this.connectionStatus = "Joining room...";
        try {
           const roomId = await invoke("server::hash_pairing_code", { input: this.joinPassphrase });
           const args = {
              room_id: roomId,
              is_initiator: false,
              signaling_url: "wss://siegu.denzyl.io"
           };
           console.log("Invoking start_webrtc_session with:", args);
           await invoke("start_webrtc_session", args);
           this.connectionStatus = "Signaling channel requested. Awaiting WebRTC Receiver connection.";
        } catch(error) {
           console.error("WebRTC Join Error", error);
           this.connectionStatus = "Error joining: " + error;
        }
    }
  },
};
</script>
