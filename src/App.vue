<script>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import DeviceList from "./components/DeviceList.vue";
import Map from "./components/Map.vue";
import Photos from "./components/Photos.vue";
import Setting from "./components/Setting.vue";
import Greet from "./components/Greet.vue";
import * as path from "@tauri-apps/api/path";

export default {
  components: { DeviceList, Map, Photos, Setting, Greet },
  data: () => ({
    clean_install: true,
    scanning: false,
    scanStatus: 'idle',
    scanProgress: {
      current: 0,
      total: 0,
      progress: 0,
      current_directory: ''
    },
    lastScanTime: 'Never',
    search: null,
    current_page: "photos",
    group: null,
    items: [
      {
        title: "Devices",
        value: "devices",
        icon: "mdi-laptop",
      },
      {
        title: "Folders",
        value: "bar",
        icon: "mdi-folder",
      },
      {
        title: "Settings",
        value: "settings",
        icon: "mdi-wrench",
      },
    ],
  }),
  async mounted() {
    // Load last scan time
    invoke("get_last_scan_time").then((time) => {
      if (time !== "Never") {
        const timestamp = parseInt(time);
        const date = new Date(timestamp * 1000);
        this.lastScanTime = date.toLocaleString();
      }
    });
    
    // Auto-scan disabled on load per user request

    // Listen for scan progress events
    listen("scan-progress", (event) => {
      const data = event.payload;
      this.scanStatus = data.status;
      
      if (data.status === 'scanning') {
        this.scanning = true;
        this.scanProgress = {
          current: data.current || 0,
          total: data.total || 0,
          progress: data.progress || 0,
          current_directory: data.current_directory || ''
        };
      } else if (data.status === 'complete') {
        this.scanning = false;
        this.scanStatus = 'complete';
        this.scanProgress.progress = 100;
        // Update last scan time
        const now = new Date();
        this.lastScanTime = now.toLocaleString();
        // Reset to idle after 3 seconds
        setTimeout(() => {
          this.scanStatus = 'idle';
        }, 3000);
      }
    });
  },
  methods: {
    generate_offer: function () {
      console.log("Generating offer.");
      this.sdp = invoke("generate_offer").then(function (response) {
        return response;
      });
    },
    scan: async function () {
      console.log("Scanning files");
      this.scanStatus = 'scanning';
      this.scanning = true;
      await invoke("scan_files");
    },
  },
  watch: {
    group() {
      this.drawer = false;
    },
  },
};
</script>

<template>
  <v-app>
    <v-layout>
      <v-main>
        <v-app-bar elevation="0" v-if="current_page === 'home' && clean_install === false" border="b" color="background">
          <v-row class="px-4 align-center">
            <v-col cols="auto">
              <v-menu offset-y>
                <template v-slot:activator="{ props }">
                  <v-btn 
                    v-bind="props"
                    variant="outlined"
                    :color="scanStatus === 'scanning' ? 'white' : 'grey-lighten-1'"
                    :loading="scanStatus === 'scanning'"
                    size="small"
                    class="text-none"
                  >
                    <v-icon start>{{ scanStatus === 'scanning' ? 'mdi-reload mdi-spin' : 'mdi-magnify' }}</v-icon>
                    {{ scanStatus === 'scanning' ? 'Scanning...' : 'Scan Library' }}
                  </v-btn>
                </template>
                <v-card min-width="300" border class="mt-2">
                  <v-card-text>
                    <div class="text-subtitle-2 mb-2 text-white">Library Status</div>
                    <div class="d-flex align-center mb-4">
                      <v-chip 
                        :color="scanStatus === 'scanning' ? 'white' : scanStatus === 'complete' ? 'white' : 'grey'"
                        size="x-small"
                        variant="flat"
                        class="mr-2"
                      >
                        {{ scanStatus === 'scanning' ? 'Active' : scanStatus === 'complete' ? 'Finished' : 'Ready' }}
                      </v-chip>
                      <span class="text-caption text-grey">{{ scanStatus.toUpperCase() }}</span>
                    </div>
                    
                    <div v-if="scanStatus === 'scanning'" class="mb-4">
                      <div class="text-caption mb-1 text-grey">{{ scanProgress.current }} / {{ scanProgress.total }} folders</div>
                      <v-progress-linear 
                        :model-value="scanProgress.progress"
                        color="white"
                        height="2"
                        rounded
                      ></v-progress-linear>
                    </div>
                    
                    <div class="text-caption text-grey mb-4">
                      Last update: {{ lastScanTime }}
                    </div>
                    
                    <v-btn 
                      v-if="scanStatus !== 'scanning'"
                      @click="scan()"
                      variant="flat"
                      color="white"
                      block
                      class="text-none"
                    >
                      Refresh Now
                    </v-btn>
                  </v-card-text>
                </v-card>
              </v-menu>
            </v-col>
            <v-spacer></v-spacer>
            <v-col cols="auto">
              <v-btn icon size="small" variant="text" color="grey-lighten-1">
                <v-icon>mdi-filter-variant</v-icon>
              </v-btn>
            </v-col>
          </v-row>
        </v-app-bar>
        <Greet v-if="clean_install" @new_device="
          clean_install = false;
        current_page = 'settings';
        " @join_group="
            clean_install = false;
          current_page = 'devices';
          "></Greet>

        <Photos v-if="current_page === 'home'" />
        <Map v-if="current_page === 'location'" />
        <DeviceList v-if="current_page === 'devices'" />
        <Setting v-if="current_page === 'settings'" @done="current_page = 'home'" />
      </v-main>
    </v-layout>
    <div class="dock-container">
      <v-sheet
        class="dock d-flex justify-space-around align-center pa-2 border rounded-pill mb-8"
        elevation="0"
        width="100%"
        max-width="320"
        color="rgba(9, 9, 11, 0.8)"
      >
        <v-btn 
          icon 
          variant="text" 
          :color="current_page === 'home' ? 'white' : 'grey-darken-1'"
          @click="current_page = 'home'"
          size="small"
        >
          <v-icon size="24">mdi-grid</v-icon>
        </v-btn>

        <v-btn 
          icon 
          variant="text" 
          :color="current_page === 'location' ? 'white' : 'grey-darken-1'"
          @click="current_page = 'location'"
          size="small"
        >
          <v-icon size="24">mdi-map-outline</v-icon>
        </v-btn>

        <v-btn 
          icon 
          variant="text" 
          :color="current_page === 'devices' ? 'white' : 'grey-darken-1'"
          @click="current_page = 'devices'"
          size="small"
        >
          <v-icon size="24">mdi-devices</v-icon>
        </v-btn>

        <v-btn 
          icon 
          variant="text" 
          :color="current_page === 'settings' ? 'white' : 'grey-darken-1'"
          @click="current_page = 'settings'"
          size="small"
        >
          <v-icon size="24">mdi-tune-variant</v-icon>
        </v-btn>
      </v-sheet>
    </div>
  </v-app>
</template>

<style scoped>
.dock-container {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  justify-content: center;
  pointer-events: none;
  z-index: 2000;
}

.dock {
  pointer-events: auto;
  backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.15);
}

.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
