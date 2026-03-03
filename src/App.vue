<script>
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
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
    clean_install: false,
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
    query: null,
    objects: [],
    faces: [],
    current_page: "home",
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
    // Check if we should show onboarding
    const initialized = await invoke("is_initialized");
    this.clean_install = !initialized;

    // Load last scan time
    invoke("get_last_scan_time").then((time) => {
      if (time !== "Never") {
        const timestamp = parseInt(time);
        const date = new Date(timestamp * 1000);
        this.lastScanTime = date.toLocaleString();
      }
    });

    // Fetch faces for search
    invoke("get_faces").then(response => {
      this.faces = JSON.parse(response);
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
    list_objects: function (val) {
      if (val && val.length > 0) {
        invoke("list_objects", { query: val }).then(
          function (response) {
            this.objects = JSON.parse(response);
          }.bind(this),
        );
      }
    },
    getFaceImageSrc(crop_path) {
      if (!crop_path) return '';
      const converted = convertFileSrc(crop_path);
      if (converted === crop_path && crop_path.startsWith('/')) {
         return `http://asset.localhost${encodeURI(crop_path)}`;
      }
      return converted;
    },
    addFaceToSearch(face) {
      this.search = (this.search || '') ? this.search + ' ' + face.photo_id : face.photo_id;
    },
  },
  watch: {
    group() {
      this.drawer = false;
    },
    query(val) {
      this.list_objects(val);
    }
  },
};
</script>

<template>
  <v-app>
    <!-- Onboarding Only -->
    <Greet v-if="clean_install" @new_device="
      clean_install = false;
      current_page = 'settings';
    " @join_group="
      clean_install = false;
      current_page = 'devices';
    "></Greet>

    <!-- Main Application -->
    <v-layout v-else>
      <v-main>
        <v-app-bar elevation="0" v-if="clean_install === false" class="border-subtle" color="background">
          <v-row class="px-4 align-center no-gutters">
            <v-col cols="auto" v-if="current_page === 'home'">
              <v-menu offset-y>
                <template v-slot:activator="{ props }">
                  <v-btn 
                    v-bind="props"
                    variant="outlined"
                    color="#a1a1aa"
                    :loading="scanStatus === 'scanning'"
                    size="small"
                    class="text-none border-subtle"
                    style="color: #a1a1aa !important;"
                  >
                    <v-icon size="18">{{ scanStatus === 'scanning' ? 'mdi-reload mdi-spin' : 'mdi-sync' }}</v-icon>
                  </v-btn>
                </template>
                <v-card min-width="300" border class="mt-2 border-subtle">
                  <v-card-text>
                    <div class="text-subtitle-2 mb-2 text-zinc-primary">Library Status</div>
                    <div class="d-flex align-center mb-4">
                      <v-chip 
                        color="#27272a"
                        size="x-small"
                        variant="flat"
                        class="mr-2 text-zinc-secondary"
                      >
                        {{ scanStatus === 'scanning' ? 'Active' : scanStatus === 'complete' ? 'Finished' : 'Ready' }}
                      </v-chip>
                      <span class="text-caption text-zinc-muted">{{ scanStatus.toUpperCase() }}</span>
                    </div>
                    
                    <div v-if="scanStatus === 'scanning'" class="mb-4">
                      <div class="text-caption mb-1 text-zinc-muted">{{ scanProgress.current }} / {{ scanProgress.total }} folders</div>
                      <v-progress-linear 
                        :model-value="scanProgress.progress"
                        color="#71717a"
                        height="2"
                        rounded
                      ></v-progress-linear>
                    </div>
                    
                    <div class="text-caption text-zinc-muted mb-4">
                      Last update: {{ lastScanTime }}
                    </div>
                    
                    <v-btn 
                      v-if="scanStatus !== 'scanning'"
                      @click="scan()"
                      variant="flat"
                      color="#27272a"
                      block
                      class="text-none text-zinc-primary"
                    >
                      Refresh Now
                    </v-btn>
                  </v-card-text>
                </v-card>
              </v-menu>
            </v-col>

            <v-col class="mx-4" v-if="current_page === 'home'">
              <v-autocomplete
                v-model="search"
                v-model:search="query"
                :items="objects"
                prepend-inner-icon="mdi-magnify"
                variant="solo-filled"
                density="compact"
                placeholder="Search memories..."
                hide-details
                flat
                rounded="lg"
                class="search-autocomplete"
                bg-color="rgba(255,255,255,0.03)"
                :menu-props="{ contentClass: 'search-menu', elevation: 4, maxWidth: '100%' }"
              >
                <template v-slot:prepend-item>
                  <div v-if="faces.length > 0" class="faces-scroller pa-2 d-flex flex-nowrap" style="overflow-x: auto; gap: 8px;">
                    <v-avatar
                      v-for="face in faces"
                      :key="face.face_id"
                      size="40"
                      class="cursor-pointer face-avatar"
                      @click="addFaceToSearch(face)"
                    >
                      <v-img :src="getFaceImageSrc(face.crop_path)"></v-img>
                    </v-avatar>
                  </div>
                  <v-divider v-if="faces.length > 0" class="opacity-10 my-1"></v-divider>
                </template>
              </v-autocomplete>
            </v-col>
            
            <v-spacer v-if="current_page !== 'home'"></v-spacer>

            <v-col cols="auto" v-if="current_page === 'home'">
              <v-btn icon size="small" variant="text" color="#71717a">
                <v-icon size="20">mdi-filter-variant</v-icon>
              </v-btn>
            </v-col>
          </v-row>
        </v-app-bar>

        <Photos v-if="current_page === 'home'" :search-query="search" />
        <Map v-if="current_page === 'location'" />
        <DeviceList v-if="current_page === 'devices'" />
        <Setting v-if="current_page === 'settings'" @done="current_page = 'home'" />
      </v-main>

      <div class="dock-container">
        <v-sheet
          class="dock d-flex justify-space-around align-center pa-2 border-subtle rounded-pill mb-8"
          elevation="0"
          width="100%"
          max-width="320"
          color="rgba(24, 24, 27, 0.8)"
        >
          <v-btn 
            icon 
            variant="text" 
            :color="current_page === 'home' ? '#e4e4e7' : '#52525b'"
            @click="current_page = 'home'"
            size="small"
          >
            <v-icon size="24">mdi-grid</v-icon>
          </v-btn>

          <v-btn 
            icon 
            variant="text" 
            :color="current_page === 'location' ? '#e4e4e7' : '#52525b'"
            @click="current_page = 'location'"
            size="small"
          >
            <v-icon size="24">mdi-map-outline</v-icon>
          </v-btn>

          <v-btn 
            icon 
            variant="text" 
            :color="current_page === 'devices' ? '#e4e4e7' : '#52525b'"
            @click="current_page = 'devices'"
            size="small"
          >
            <v-icon size="24">mdi-devices</v-icon>
          </v-btn>

          <v-btn 
            icon 
            variant="text" 
            :color="current_page === 'settings' ? '#e4e4e7' : '#52525b'"
            @click="current_page = 'settings'"
            size="small"
          >
            <v-icon size="24">mdi-tune-variant</v-icon>
          </v-btn>
        </v-sheet>
      </div>
    </v-layout>
  </v-app>
</template>

<style scoped>
.search-autocomplete :deep(.v-field__outline) {
  display: none;
}

.search-autocomplete :deep(.v-field__input) {
  font-size: 0.875rem;
  color: #f4f4f5 !important; /* Zinc-100 */
}

:global(.search-menu) {
  background: #ffffff !important;
  border: 1px solid #e4e4e7 !important;
  border-radius: 0 0 8px 8px !important;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1) !important;
}

:global(.search-menu .v-list) {
  background: transparent !important;
  color: #27272a !important; /* Zinc-800 */
}

:global(.search-menu .v-list-item:hover) {
  background: #f4f4f5 !important; /* Zinc-100 */
}

:global(.search-menu .v-list-item--active) {
  background: #e4e4e7 !important; /* Zinc-200 */
  color: #09090b !important;
}

:global(.search-menu .v-divider) {
  border-color: #e4e4e7 !important;
}

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
