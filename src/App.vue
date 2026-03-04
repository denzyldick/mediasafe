<script>
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import DeviceList from "./components/DeviceList.vue";
import Map from "./components/Map.vue";
import Photos from "./components/Photos.vue";
import People from "./components/People.vue";
import Setting from "./components/Setting.vue";
import Greet from "./components/Greet.vue";
import * as path from "@tauri-apps/api/path";

export default {
  components: { DeviceList, Map, Photos, People, Setting, Greet },
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
    indexingCount: 0,
    lastScanTime: 'Never',
    search: '',
    query: '',
    objects: [],
    faces: [],
    filters: {
      favoritesOnly: false,
      dateRange: 'all',
      folder: null,
    },
    directories: [],
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

    // Check initial indexing status
    invoke("get_indexing_status").then(count => {
      this.indexingCount = count;
    });

    // Listen for indexing progress
    listen("indexing-progress", (event) => {
      this.indexingCount = event.payload;
    });

    // Fetch faces for search
    invoke("get_faces").then(response => {
      try {
        const parsed = JSON.parse(response);
        this.faces = Array.isArray(parsed) ? parsed : [];
      } catch (e) {
        console.error("Failed to parse faces:", e);
        this.faces = [];
      }
    }).catch(err => {
      console.error("Failed to get faces:", err);
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

    this.list_directories();

    // Fetch top tags for initial search suggestions
    invoke("get_top_tags").then(response => {
      try {
        const parsed = JSON.parse(response);
        this.objects = Array.isArray(parsed) ? parsed : [];
      } catch (e) {
        console.error("Failed to parse top tags:", e);
      }
    });
  },
  computed: {
    hasActiveFilters() {
      return this.filters.favoritesOnly || this.filters.dateRange !== 'all' || this.filters.folder;
    }
  },
  methods: {
    resetFilters() {
      this.filters = {
        favoritesOnly: false,
        dateRange: 'all',
        folder: null,
      };
    },
    list_directories() {
      invoke("list_directories").then((response) => {
        this.directories = JSON.parse(response);
      });
    },
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
      } else {
        // Restore top tags when search is cleared
        invoke("get_top_tags").then(response => {
          this.objects = JSON.parse(response);
        });
      }
    },
    getFaceImageSrc(crop_path) {
      if (!crop_path) return '';
      const converted = convertFileSrc(crop_path);
      if (converted === crop_path && crop_path.startsWith('/')) {
         return `http://asset.localhost${encodeURIComponent(crop_path)}`;
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
      <v-app-bar elevation="0" v-if="clean_install === false && current_page === 'home'" color="white" theme="light">
        <v-row class="px-4 align-center no-gutters">
          <v-col cols="auto" v-if="current_page === 'home'">
            <v-menu offset-y>
              <template v-slot:activator="{ props }">
                <v-btn 
                  v-bind="props"
                  variant="outlined"
                  :color="(scanStatus === 'scanning' || indexingCount > 0) ? 'primary' : 'grey-darken-1'"
                  size="small"
                  class="text-none"
                >
                  <template v-slot:prepend>
                    <v-progress-circular
                      v-if="scanStatus === 'scanning' || indexingCount > 0"
                      indeterminate
                      size="16"
                      width="2"
                      color="currentColor"
                      class="mr-1"
                    ></v-progress-circular>
                    <v-icon v-else size="18">mdi-sync</v-icon>
                  </template>
                  {{ scanStatus === 'scanning' ? 'Scanning...' : (indexingCount > 0 ? 'Indexing...' : 'Refresh') }}
                </v-btn>
              </template>
              <v-card min-width="300" border class="mt-2 border-subtle" color="rgba(24, 24, 27, 0.8)" style="backdrop-filter: blur(16px);">
                <v-card-text>
                  <div class="text-subtitle-2 mb-2 text-zinc-primary">Library Status</div>
                  
                  <!-- Scan Status -->
                  <div class="d-flex align-center mb-4">
                    <v-chip 
                      :color="scanStatus === 'scanning' ? '#27272a' : '#18181b'"
                      size="x-small"
                      variant="flat"
                      class="mr-2 text-zinc-secondary"
                    >
                      File Scan: {{ scanStatus === 'scanning' ? 'Active' : 'Ready' }}
                    </v-chip>
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

                  <!-- Indexing Status -->
                  <div class="d-flex align-center mb-4">
                    <v-chip 
                      :color="indexingCount > 0 ? '#27272a' : '#18181b'"
                      size="x-small"
                      variant="flat"
                      class="mr-2 text-zinc-secondary"
                    >
                      AI Indexing: {{ indexingCount > 0 ? 'Active' : 'Complete' }}
                    </v-chip>
                    <span v-if="indexingCount > 0" class="text-caption text-zinc-muted">{{ indexingCount }} left</span>
                  </div>
                  
                  <v-divider class="my-4 opacity-5"></v-divider>
                  
                  <div class="text-caption text-zinc-muted mb-4">
                    Last scan: {{ lastScanTime }}
                  </div>
                  
                  <v-btn 
                    v-if="scanStatus !== 'scanning' && indexingCount === 0"
                    @click="scan()"
                    variant="flat"
                    color="#27272a"
                    block
                    class="text-none text-zinc-primary"
                  >
                    Sync Library
                  </v-btn>
                </v-card-text>
              </v-card>
            </v-menu>
          </v-col>

          <!-- Global Search - Only on Photos page -->
          <v-col class="mx-2 flex-grow-1" v-if="current_page === 'home'">
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
              class="search-autocomplete w-100"
              bg-color="rgba(0,0,0,0.05)"
              :menu-props="{ contentClass: 'search-menu', elevation: 4, disabled: !objects.length && !faces.length }"
              no-data-text=""
            >
                <template v-slot:prepend-item>
                  <v-list-subheader v-if="!query" class="text-zinc-muted text-uppercase tracking-widest text-caption py-2">Top Suggestions</v-list-subheader>
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
              <v-menu :close-on-content-click="false" offset-y>
                <template v-slot:activator="{ props }">
                  <v-btn icon size="small" variant="text" v-bind="props" :color="hasActiveFilters ? 'white' : '#71717a'">
                    <v-badge :model-value="hasActiveFilters" color="white" dot px="1">
                      <v-icon size="20">mdi-filter-variant</v-icon>
                    </v-badge>
                  </v-btn>
                </template>
                <v-card min-width="250" border class="mt-2 border-subtle" color="rgba(24, 24, 27, 0.8)" style="backdrop-filter: blur(16px);">
                  <v-list bg-color="transparent" density="compact" class="px-2">
                    <v-list-item class="px-0">
                      <v-switch
                        v-model="filters.favoritesOnly"
                        label="Favorites only"
                        color="#e4e4e7"
                        hide-details
                        density="compact"
                        inset
                        class="text-zinc-secondary"
                      ></v-switch>
                    </v-list-item>
                    
                    <v-divider class="opacity-5 my-2"></v-divider>
                    
                    <v-list-subheader class="text-zinc-muted text-uppercase tracking-widest text-caption px-0">Date Range</v-list-subheader>
                    <v-list-item class="px-0">
                      <v-btn-toggle v-model="filters.dateRange" mandatory variant="outlined" density="compact" class="border-subtle w-100" color="#e4e4e7">
                        <v-btn value="all" size="x-small" class="text-none flex-grow-1 text-zinc-secondary">All</v-btn>
                        <v-btn value="month" size="x-small" class="text-none flex-grow-1 text-zinc-secondary">Month</v-btn>
                        <v-btn value="year" size="x-small" class="text-none flex-grow-1 text-zinc-secondary">Year</v-btn>
                      </v-btn-toggle>
                    </v-list-item>

                    <v-divider class="opacity-5 my-2"></v-divider>

                    <v-list-subheader class="text-zinc-muted text-uppercase tracking-widest text-caption px-0">Folder</v-list-subheader>
                    <v-list-item class="px-0">
                      <v-select
                        v-model="filters.folder"
                        :items="directories"
                        placeholder="All folders"
                        variant="solo-filled"
                        density="compact"
                        hide-details
                        flat
                        bg-color="rgba(255,255,255,0.05)"
                        clearable
                        class="text-zinc-secondary custom-select"
                      ></v-select>
                    </v-list-item>
                  </v-list>
                  
                  <v-card-actions class="pa-2">
                    <v-btn variant="text" size="x-small" color="#71717a" class="text-none" @click="resetFilters">Reset Filters</v-btn>
                  </v-card-actions>
                </v-card>
              </v-menu>
                      </v-col>
                    </v-row>
                  </v-app-bar>
            
                  <v-main>
                    <Photos v-if="current_page === 'home'" :search-query="search" :filters="filters" @clear-search="search = null" />        <People v-if="current_page === 'people'" />
        <Map v-if="current_page === 'location'" />
        <DeviceList v-if="current_page === 'devices'" />
        <Setting v-if="current_page === 'settings'" @done="current_page = 'home'" />
      </v-main>

      <div class="dock-container" v-if="!clean_install">
        <v-sheet
          class="dock d-flex justify-space-around align-center pa-2 border-subtle rounded-pill mb-8"
          elevation="0"
          width="100%"
          max-width="380"
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
            :color="current_page === 'people' ? '#e4e4e7' : '#52525b'"
            @click="current_page = 'people'"
            size="small"
          >
            <v-icon size="24">mdi-account-group-outline</v-icon>
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
  color: #18181b !important; /* Dark text for white background */
}

.custom-select :deep(.v-field__input) {
  font-size: 0.75rem;
  color: #18181b !important;
}

.custom-select :deep(.v-chip) {
  background: #27272a !important;
  color: #e4e4e7 !important;
}

:global(.search-menu) {
  background: rgba(24, 24, 27, 0.8) !important;
  backdrop-filter: blur(16px) !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  border-radius: 0 0 8px 8px !important;
}

:global(.search-menu .v-list) {
  background: transparent !important;
  color: #a1a1aa !important; /* Zinc-400 */
}

:global(.search-menu .v-list-item:hover) {
  background: rgba(255, 255, 255, 0.05) !important;
}

:global(.search-menu .v-list-item--active) {
  background: rgba(255, 255, 255, 0.1) !important;
  color: #e4e4e7 !important; /* Zinc-200 */
}

:global(.search-menu .v-divider) {
  border-color: rgba(255, 255, 255, 0.05) !important;
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
