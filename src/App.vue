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
    
    // Start scan on load
    invoke("scan_files");

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
        <v-app-bar elevation="1" v-if="current_page === 'home' && clean_install === false">
          <v-row>
            <v-col md="3" sm="3" lg="3">
              <v-menu offset-y>
                <template v-slot:activator="{ props }">
                  <v-btn 
                    v-bind="props"
                    :color="scanStatus === 'scanning' ? 'green' : 'grey'"
                    :loading="scanStatus === 'scanning'"
                  >
                    <v-icon>{{ scanStatus === 'scanning' ? 'mdi-reload mdi-spin' : 'mdi-magnify' }}</v-icon>
                    &nbsp;{{ scanStatus === 'scanning' ? 'Scanning...' : 'Scan' }}
                  </v-btn>
                </template>
                <v-card min-width="300">
                  <v-card-text>
                    <div class="text-subtitle-2 mb-2">Scan Status</div>
                    <v-chip 
                      :color="scanStatus === 'scanning' ? 'green' : scanStatus === 'complete' ? 'blue' : 'grey'"
                      size="small"
                      class="mb-3"
                    >
                      {{ scanStatus === 'scanning' ? 'Scanning' : scanStatus === 'complete' ? 'Complete' : 'Idle' }}
                    </v-chip>
                    
                    <div v-if="scanStatus === 'scanning'" class="mb-3">
                      <div class="text-caption mb-1">{{ scanProgress.current }} / {{ scanProgress.total }} folders</div>
                      <v-progress-linear 
                        :model-value="scanProgress.progress"
                        color="green"
                        height="6"
                        rounded
                      ></v-progress-linear>
                      <div class="text-caption mt-1 text-grey">{{ scanProgress.current_directory }}</div>
                    </div>
                    
                    <div class="text-caption text-grey">
                      Last scan: {{ lastScanTime }}
                    </div>
                    
                    <v-btn 
                      v-if="scanStatus !== 'scanning'"
                      @click="scan()"
                      color="primary"
                      block
                      class="mt-3"
                    >
                      Start Scan
                    </v-btn>
                  </v-card-text>
                </v-card>
              </v-menu>
            </v-col>
            <v-col md="1" sm="1" lg="1">
              <v-btn color="gray">
                <v-icon>mdi-filter</v-icon>
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
        class="dock glass-panel d-flex justify-space-around align-center pa-2 rounded-xl mb-6"
        elevation="0"
        width="100%"
        max-width="400"
      >
        <v-btn 
          icon 
          variant="text" 
          :color="current_page === 'home' ? 'primary' : 'grey-lighten-1'"
          @click="current_page = 'home'"
        >
          <v-icon size="large">mdi-image-multiple</v-icon>
        </v-btn>

        <v-btn 
          icon 
          variant="text" 
          :color="current_page === 'location' ? 'primary' : 'grey-lighten-1'"
          @click="current_page = 'location'"
        >
          <v-icon size="large">mdi-map-marker</v-icon>
        </v-btn>

        <v-btn 
          icon 
          variant="text" 
          :color="current_page === 'devices' ? 'primary' : 'grey-lighten-1'"
          @click="current_page = 'devices'"
        >
          <v-icon size="large">mdi-laptop</v-icon>
        </v-btn>

        <v-btn 
          icon 
          variant="text" 
          :color="current_page === 'settings' ? 'primary' : 'grey-lighten-1'"
          @click="current_page = 'settings'"
        >
          <v-icon size="large">mdi-cog</v-icon>
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
