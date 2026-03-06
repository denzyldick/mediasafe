<script>
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import DeviceList from "./components/DeviceList.vue";
import Map from "./components/Map.vue";
import Photos from "./components/Photos.vue";
import People from "./components/People.vue";
import Setting from "./components/Setting.vue";
import Greet from "./components/Greet.vue";
import logo from "./assets/logo.png";

export default {
  components: { DeviceList, Map, Photos, People, Setting, Greet },
  data: () => ({
    logoUrl: logo,
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
  }),
  async mounted() {
    const initialized = await invoke("is_initialized");
    this.clean_install = !initialized;

    invoke("get_last_scan_time").then((time) => {
      if (time !== "Never") {
        const timestamp = parseInt(time);
        const date = new Date(timestamp * 1000);
        this.lastScanTime = date.toLocaleString();
      }
    });

    invoke("get_indexing_status").then(count => {
      this.indexingCount = count;
    });

    listen("indexing-progress", (event) => {
      this.indexingCount = event.payload;
    });

    invoke("get_people").then(response => {
      try {
        const parsed = JSON.parse(response);
        this.faces = Array.isArray(parsed) ? parsed : [];
      } catch (e) {
        console.error("Failed to parse people:", e);
      }
    });
    
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
        this.lastScanTime = new Date().toLocaleString();
        setTimeout(() => { this.scanStatus = 'idle'; }, 3000);
      }
    });

    this.list_directories();

    invoke("get_top_tags").then(response => {
      try {
        const parsed = JSON.parse(response);
        this.objects = Array.isArray(parsed) ? parsed : [];
      } catch (e) {}
    });
  },
  computed: {
    hasActiveFilters() {
      return this.filters.favoritesOnly || this.filters.dateRange !== 'all' || this.filters.folder;
    },
    filteredPeople() {
      if (!this.faces) return [];
      if (!this.query) return this.faces.slice(0, 10);
      const q = this.query.toLowerCase();
      return this.faces.filter(p => p.name.toLowerCase().includes(q)).slice(0, 10);
    }
  },
  methods: {
    resetFilters() {
      this.filters = { favoritesOnly: false, dateRange: 'all', folder: null };
    },
    list_directories() {
      invoke("list_directories").then((response) => {
        this.directories = JSON.parse(response);
      });
    },
    scan: async function () {
      this.scanStatus = 'scanning';
      this.scanning = true;
      await invoke("scan_files");
    },
    list_objects: function (val) {
      if (val && val.length > 0) {
        invoke("list_objects", { query: val }).then(
          response => { this.objects = JSON.parse(response); }
        );
      } else {
        invoke("get_top_tags").then(response => { this.objects = JSON.parse(response); });
      }
    },
    getFaceImageSrc(crop_path, encoded) {
      return encoded || '';
    },
    addPersonToSearch(person) {
      this.search = person.name;
      this.query = person.name;
    },
  },
  watch: {
    query(val) {
      this.list_objects(val);
    }
  },
};
</script>

<template>
  <v-app style="background-color: #fafafa !important;">
    <Greet v-if="clean_install" @setup-local="clean_install = false; current_page = 'settings';" @setup-sync="clean_install = false; current_page = 'devices';"></Greet>

    <v-layout v-else style="background-color: #fafafa !important;">
      <v-app-bar elevation="0" v-if="current_page === 'home'" color="#ffffff" class="border-bottom-subtle">
        <v-row class="px-4 align-center no-gutters">
          <v-col cols="auto">
            <v-menu offset-y>
              <template v-slot:activator="{ props }">
                <v-btn v-bind="props" variant="flat" class="siegu-btn" size="small">
                  <template v-slot:prepend>
                    <v-progress-circular v-if="scanStatus === 'scanning' || indexingCount > 0" indeterminate size="16" width="2" color="currentColor" class="mr-1"></v-progress-circular>
                    <v-icon v-else size="18">mdi-sync</v-icon>
                  </template>
                  {{ scanStatus === 'scanning' ? 'Scanning...' : (indexingCount > 0 ? 'Indexing...' : 'Refresh') }}
                </v-btn>
              </template>
              <v-card min-width="300" border class="mt-2 border-subtle" color="#ffffff" rounded="xl">
                <v-card-text>
                  <div class="text-subtitle-2 mb-2 text-zinc-primary">Library Status</div>
                  <div class="d-flex align-center mb-4">
                    <v-chip color="#f4f4f5" size="x-small" variant="flat" class="mr-2 text-zinc-secondary border-subtle">
                      File Scan: {{ scanStatus === 'scanning' ? 'Active' : 'Ready' }}
                    </v-chip>
                  </div>
                  <div v-if="scanStatus === 'scanning'" class="mb-4">
                    <div class="text-caption mb-1 text-zinc-muted">{{ scanProgress.current }} / {{ scanProgress.total }} folders</div>
                    <v-progress-linear :model-value="scanProgress.progress" color="#18181b" height="2" rounded bg-color="#f4f4f5" bg-opacity="1"></v-progress-linear>
                  </div>
                  <div class="d-flex align-center mb-4">
                    <v-chip color="#f4f4f5" size="x-small" variant="flat" class="mr-2 text-zinc-secondary border-subtle">
                      AI Indexing: {{ indexingCount > 0 ? 'Active' : 'Complete' }}
                    </v-chip>
                    <span v-if="indexingCount > 0" class="text-caption text-zinc-muted">{{ indexingCount }} left</span>
                  </div>
                  <v-divider class="my-4 border-subtle"></v-divider>
                  <div class="text-caption text-zinc-muted mb-4">Last scan: {{ lastScanTime }}</div>
                  <v-btn v-if="scanStatus !== 'scanning' && indexingCount === 0" @click="scan()" variant="flat" block class="siegu-btn">Sync Library</v-btn>
                </v-card-text>
              </v-card>
            </v-menu>
          </v-col>

          <v-col class="mx-2 flex-grow-1">
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
              class="search-autocomplete w-100 siegu-field"
              bg-color="#f4f4f5"
              :menu-props="{ contentClass: 'siegu-list', elevation: 4, disabled: !objects.length && !filteredPeople.length }"
              no-data-text=""
            >
                <template v-slot:prepend-item>
                  <div v-if="filteredPeople.length > 0">
                    <v-list-subheader class="text-zinc-muted text-uppercase tracking-widest text-caption py-2">People</v-list-subheader>
                    <div class="pa-2 d-flex flex-nowrap" style="overflow-x: auto; gap: 8px;">
                      <div v-for="person in filteredPeople" :key="person.id" class="d-flex flex-column align-center cursor-pointer" @click="addPersonToSearch(person)" style="min-width: 60px;">
                        <v-avatar size="40" class="mb-1 border-subtle">
                          <v-img :src="getFaceImageSrc(person.representative_crop, person.encoded)"></v-img>
                        </v-avatar>
                        <span class="text-caption text-zinc-muted text-truncate w-100 text-center">{{ person.name }}</span>
                      </div>
                    </div>
                    <v-divider class="border-subtle my-1"></v-divider>
                  </div>
                  <v-list-subheader v-if="!query" class="text-zinc-muted text-uppercase tracking-widest text-caption py-2">Top Suggestions</v-list-subheader>
                </template>
              </v-autocomplete>
            </v-col>

            <v-col cols="auto">
              <v-menu :close-on-content-click="false" offset-y>
                <template v-slot:activator="{ props }">
                  <v-btn icon size="small" variant="text" v-bind="props" :color="hasActiveFilters ? '#18181b' : '#71717a'">
                    <v-badge :model-value="hasActiveFilters" color="black" dot px="1">
                      <v-icon size="20">mdi-filter-variant</v-icon>
                    </v-badge>
                  </v-btn>
                </template>
                <v-card min-width="250" border class="mt-2 border-subtle" color="#ffffff" rounded="xl">
                  <v-list bg-color="transparent" density="compact" class="px-2 ga-2">
                    <v-list-item class="px-0">
                      <v-switch v-model="filters.favoritesOnly" label="Favorites only" color="#18181b" hide-details density="compact" inset class="text-zinc-secondary px-2"></v-switch>
                    </v-list-item>
                    <v-divider class="border-subtle my-2"></v-divider>
                    <v-list-subheader class="text-zinc-muted text-uppercase tracking-widest text-caption px-0">Date Range</v-list-subheader>
                    <v-list-item class="px-0">
                      <v-btn-toggle v-model="filters.dateRange" mandatory variant="flat" density="compact" class="ga-2 w-100 bg-transparent" color="#18181b">
                        <v-btn value="all" size="x-small" class="siegu-btn flex-grow-1">All</v-btn>
                        <v-btn value="month" size="x-small" class="siegu-btn flex-grow-1">Month</v-btn>
                        <v-btn value="year" size="x-small" class="siegu-btn flex-grow-1">Year</v-btn>
                      </v-btn-toggle>
                    </v-list-item>
                    <v-divider class="border-subtle my-2"></v-divider>
                    <v-list-subheader class="text-zinc-muted text-uppercase tracking-widest text-caption px-0">Folder</v-list-subheader>
                    <v-list-item class="px-0">
                      <v-select v-model="filters.folder" :items="directories" placeholder="All folders" variant="solo-filled" density="compact" hide-details flat bg-color="#f4f4f5" clearable rounded="lg" class="text-zinc-secondary"></v-select>
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
  
        <v-main style="background-color: #fafafa !important;">
          <Photos v-if="current_page === 'home'" :search-query="search" :filters="filters" @clear-search="search = null" />
          <People v-if="current_page === 'people'" />
          <Map v-if="current_page === 'location'" />
          <DeviceList v-if="current_page === 'devices'" />
          <Setting v-if="current_page === 'settings'" @done="current_page = 'home'" />
        </v-main>

      <div class="dock-container" v-if="!clean_install">
        <v-sheet class="dock d-flex justify-space-around align-center pa-2 rounded-pill mb-8" elevation="0" width="100%" max-width="380" color="rgba(255, 255, 255, 0.8)">
          <v-btn icon variant="text" @click="current_page = 'home'" size="small" class="siegu-dock-btn" :class="{'siegu-dock-btn--active': current_page === 'home'}">
            <v-img :src="logoUrl" width="24" height="24" :class="current_page === 'home' ? 'opacity-100' : 'opacity-40'"></v-img>
          </v-btn>
          <v-btn icon variant="text" @click="current_page = 'people'" size="small" class="siegu-dock-btn" :class="{'siegu-dock-btn--active': current_page === 'people'}">
            <v-icon size="24">mdi-account-group-outline</v-icon>
          </v-btn>
          <v-btn icon variant="text" @click="current_page = 'location'" size="small" class="siegu-dock-btn" :class="{'siegu-dock-btn--active': current_page === 'location'}">
            <v-icon size="24">mdi-map-outline</v-icon>
          </v-btn>
          <v-btn icon variant="text" @click="current_page = 'devices'" size="small" class="siegu-dock-btn" :class="{'siegu-dock-btn--active': current_page === 'devices'}">
            <v-icon size="24">mdi-laptop</v-icon>
          </v-btn>
          <v-btn icon variant="text" @click="current_page = 'settings'" size="small" class="siegu-dock-btn" :class="{'siegu-dock-btn--active': current_page === 'settings'}">
            <v-icon size="24">mdi-cog-outline</v-icon>
          </v-btn>
        </v-sheet>
      </div>
    </v-layout>
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
  border: 1px solid rgba(0, 0, 0, 0.1);
}

.siegu-dock-btn {
  color: #71717a !important;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1) !important;
  border-radius: 50% !important;
}

.siegu-dock-btn:hover {
  background: #f4f4f5 !important;
  color: #18181b !important;
  transform: translateY(-2px);
}

.siegu-dock-btn--active {
  color: #18181b !important;
  background: rgba(0, 0, 0, 0.05) !important;
}
</style>
