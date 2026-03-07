<template>
  <v-dialog v-model="dialog" width="auto" scrim="black" transition="dialog-bottom-transition">
    <template v-slot:activator="{ props }">
      <v-btn
        v-bind="props"
        color="#000000"
        theme="dark"
        variant="flat"
        class="siegu-btn px-6"
        height="44"
      >
        <div class="d-flex align-center">
          <div class="siegu-icon-circle siegu-icon-circle-md mr-3">
            <v-icon size="14" color="white">mdi-plus</v-icon>
          </div>
          <span class="text-white font-weight-bold">Add Device</span>
        </div>
      </v-btn>
    </template>
    
    <v-card class="border-subtle pa-6 text-center bg-siegu-white" rounded="xl" min-width="350" max-width="400">
      <div class="text-h5 font-weight-bold text-zinc-primary mb-2">Link Device</div>
      <div class="text-body-2 text-zinc-secondary mb-6">
        Sync your library across your own hardware.
      </div>

      <div class="d-flex justify-center mb-6 mt-2">
         <v-btn-toggle v-model="mode" mandatory variant="flat" class="ga-2 bg-transparent">
            <v-btn value="host" class="siegu-btn text-none px-6">Host</v-btn>
            <v-btn value="join" class="siegu-btn text-none px-6">Join</v-btn>
         </v-btn-toggle>
      </div>

      <div class="d-flex justify-center mb-6" v-if="uuid && mode === 'host'">
        <v-sheet class="bg-siegu-white rounded-lg pa-4 shadow-lg border-subtle">
             <qrcode-vue :value="uuid" :size="180" level="H" />
        </v-sheet>
      </div>

      <div class="text-caption text-zinc-muted mb-2 uppercase tracking-widest" v-if="mode === 'host'">Manual Phrase</div>
      <div class="d-flex justify-center flex-wrap gap-2 mb-6" v-if="passphrase.length > 0 && mode === 'host'">
        <v-chip
          v-for="(word, index) in passphrase"
          :key="index"
          color="#f4f4f5"
          variant="flat"
          class="font-weight-medium mx-1 text-zinc-primary border-subtle"
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
           density="comfortable"
           hide-details
           flat
           rounded="lg"
           class="text-center siegu-field"
           @keyup.enter="joinWebRTC"
         ></v-text-field>
         <v-btn variant="flat" @click="joinWebRTC" class="siegu-btn py-6" block>
            <div class="d-flex align-center">
              <div class="siegu-icon-circle mr-3">
                <v-icon size="14" color="white">mdi-link-variant</v-icon>
              </div>
              <span class="text-white">Link Device</span>
            </div>
         </v-btn>
      </div>

      <div class="text-caption text-zinc-muted mb-1 text-center py-2" v-if="connectionStatus">
         <v-progress-circular v-if="!isConnected" indeterminate color="black" size="16" width="2" class="mr-2 opacity-50"></v-progress-circular>
         <v-icon v-else color="success" size="16" class="mr-2">mdi-check-circle-outline</v-icon>
         {{ connectionStatus }}
      </div>

      <v-divider class="opacity-10 my-4"></v-divider>

      <v-card-actions class="justify-center">
        <v-btn variant="flat" class="siegu-btn px-6" @click="dialog = false">
           <div class="d-flex align-center">
             <div class="siegu-icon-circle siegu-icon-circle-sm mr-2">
               <v-icon size="12" color="white">mdi-close</v-icon>
             </div>
             <span class="text-white">Cancel</span>
           </div>
        </v-btn>
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
           console.error("Pairing Error:", error);
           this.connectionStatus = "Pairing Error.";
        }
    },
    async listen(roomId) {
      this.connectionStatus = "Waiting for partner device to scan or type phrase...";
      const args = {
          roomId: roomId,
          isInitiator: true,
          signalingUrl: "wss://siegu.denzyl.io"
      };
      try {
          await invoke("start_webrtc_session", args);
          this.connectionStatus = "Signaling channel requested. Awaiting WebRTC connection.";
      } catch (error) {
          this.connectionStatus = "Error connecting: " + error;
      }
    },
    async joinWebRTC() {
        if (!this.joinPassphrase) return;
        this.connectionStatus = "Joining room...";
        try {
           const roomId = await invoke("hash_pairing_code", { input: this.joinPassphrase });
           const args = {
              roomId: roomId,
              isInitiator: false,
              signalingUrl: "wss://siegu.denzyl.io"
           };
           await invoke("start_webrtc_session", args);
           this.connectionStatus = "Signaling channel requested. Awaiting WebRTC Receiver connection.";
        } catch(error) {
           this.connectionStatus = "Error joining: " + error;
        }
    }
  },
};
</script>
