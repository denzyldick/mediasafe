<script>
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import DeviceList from "./components/DeviceList.vue";
import Map from "./components/Map.vue";
import Photos from "./components/Photos.vue";
import People from "./components/People.vue";
import Setting from "./components/Setting.vue";
import Greet from "./components/Greet.vue";
import Connect from "./components/Connect.vue";
import logo from "./assets/logo.png";

export default {
  components: { DeviceList, Map, Photos, People, Setting, Greet, Connect },
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
      videosOnly: false,
      dateRange: 'all',
      folder: null,
    },
    directories: [],
    current_page: "home",
    downloadProgress: {},
    isDownloadingModels: false,
    onboardingStep: 'greet',
    downloadedModels: [],
    deviceConnected: false,
    showConnectUI: false,
    os: '',
    thumbnailQueue: [],
    isProcessingThumb: false,
    thumbRequests: new Set(),
  }),
  async mounted() {
    invoke("get_os").then(os => this.os = os);
    
    // Global Thumbnail Worker
    window.addEventListener('request-thumbnail', (e) => {
      const { id } = e.detail;
      if (this.thumbRequests.has(id)) return;
      this.thumbRequests.add(id);
      this.thumbnailQueue.push(e.detail);
      this.processThumbnailQueue();
    });

    listen("download-progress", (event) => {
      const { model, downloaded, total } = event.payload;
      this.isDownloadingModels = true;
      this.downloadProgress = { ...this.downloadProgress, [model]: { downloaded, total } };
    });

    listen("download-complete", () => {
      // Check if all selected models are done (simplified: just clear after a delay)
      setTimeout(() => {
        this.isDownloadingModels = false;
        this.downloadProgress = {};
      }, 2000);
    });

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
    this.checkModels();

    invoke("get_top_tags").then(response => {
      try {
        const parsed = JSON.parse(response);
        this.objects = Array.isArray(parsed) ? parsed : [];
      } catch (e) {}
    });
  },
  computed: {
    isMobile() {
      return this.os === 'android' || this.os === 'ios';
    },
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
      this.filters = { favoritesOnly: false, videosOnly: false, dateRange: 'all', folder: null };
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
      this.current_page = "home";
    },
    async checkModels() {
      const downloaded = await invoke("check_models");
      this.downloadedModels = downloaded;
    },
    async finishSetupAndScan() {
      this.clean_install = false;
      this.onboardingStep = 'complete';
      this.current_page = 'home';
      
      // If we linked a device during onboarding, we are a Guest
      // We don't need to scan local folders, we are receiving from host
      if (this.deviceConnected) {
        console.log("Device linked during onboarding, skipping local scan.");
        return;
      }

      // Give UI time to switch before starting heavy scan
      setTimeout(() => {
        invoke("scan_files", { scan: true });
      }, 500);
    },
    async processThumbnailQueue() {
      if (this.isProcessingThumb || this.thumbnailQueue.length === 0) return;
      
      this.isProcessingThumb = true;
      const item = this.thumbnailQueue.shift();
      const video = this.$refs.thumbVideo;
      const canvas = this.$refs.thumbCanvas;

      if (!video || !canvas) {
        this.isProcessingThumb = false;
        return;
      }

      video.src = item.videoUrl;
      video.onloadedmetadata = () => {
        video.currentTime = Math.min(1, video.duration / 2);
      };

      video.onseeked = async () => {
        try {
          const ctx = canvas.getContext('2d');
          canvas.width = 400;
          canvas.height = (video.videoHeight / video.videoWidth) * 400;
          ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
          const b64 = canvas.toDataURL('image/jpeg', 0.7);
          
          // Save to backend DB permanently
          await invoke("update_video_thumbnail", { id: item.id, b64 });
          
          // Notify any listening image components
          window.dispatchEvent(new CustomEvent(`thumbnail-ready-${item.id}`, { detail: { b64 } }));
        } catch (e) {
          console.error("Worker failed to capture thumb", e);
        }
        
        // Clean up and process next
        video.src = "";
        this.isProcessingThumb = false;
        this.processThumbnailQueue();
      };

      video.onerror = () => {
        video.src = "";
        this.isProcessingThumb = false;
        this.processThumbnailQueue();
      };
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
  <v-app class="bg-siegu-main">
    <!-- Global Download Bar -->
    <v-system-bar v-if="isDownloadingModels" color="black" theme="dark" class="justify-center py-1" height="auto">
      <div class="d-flex align-center py-1">
        <v-progress-circular indeterminate size="14" width="2" class="mr-2" color="white"></v-progress-circular>
        <span class="text-caption font-weight-bold">Downloading AI Models...</span>
      </div>
    </v-system-bar>

    <!-- Hidden Thumbnail Worker -->
    <div style="display: none;">
      <video ref="thumbVideo" muted preload="metadata"></video>
      <canvas ref="thumbCanvas"></canvas>
    </div>

    <!-- Guided Onboarding -->
    <template v-if="clean_install">
      <Greet v-if="onboardingStep === 'greet'" @setup-local="onboardingStep = 'folders'" @setup-sync="onboardingStep = 'sync'"></Greet>
      
      <!-- Step 2: Folders -->
      <v-container v-else-if="onboardingStep === 'folders'" class="fill-height bg-siegu-white" fluid>
        <v-row justify="center">
          <v-col cols="12" sm="10" md="8" lg="6">
            <v-card variant="flat" rounded="xl" class="pa-8 border-subtle">
              <div class="text-center mb-8">
                <div class="siegu-icon-circle mx-auto mb-4">
                  <v-icon color="white">mdi-folder-plus</v-icon>
                </div>
                <h2 class="text-h4 font-weight-bold text-zinc-primary">Add your media</h2>
                <p class="text-zinc-secondary">Select the folders where you keep your photos and videos.</p>
              </div>

              <Setting :embedded="true" hide-ai-section @folder-added="list_directories" />

              <v-btn block color="black" height="56" class="siegu-btn mt-8" :disabled="directories.length === 0" @click="onboardingStep = 'models'">
                Continue to AI Setup
              </v-btn>
            </v-card>
          </v-col>
        </v-row>
      </v-container>

      <!-- Step 3: Models -->
      <v-container v-else-if="onboardingStep === 'models'" class="fill-height bg-siegu-white" fluid>
        <v-row justify="center">
          <v-col cols="12" sm="10" md="8" lg="6">
            <v-card variant="flat" rounded="xl" class="pa-8 border-subtle">
              <div class="text-center mb-8">
                <div class="siegu-icon-circle-dark mx-auto mb-4">
                  <v-icon color="white">mdi-auto-fix</v-icon>
                </div>
                <h2 class="text-h4 font-weight-bold text-zinc-primary">AI Intelligence</h2>
                <p class="text-zinc-secondary">Download the neural models to enable face recognition and semantic search.</p>
              </div>

              <v-alert v-if="!isDownloadingModels && downloadedModels.length < 2" border="start" color="zinc-50" class="border-subtle mb-6">
                <template v-slot:prepend>
                  <v-icon color="zinc-primary">mdi-information-outline</v-icon>
                </template>
                <div class="text-caption text-zinc-secondary">
                  The models are approx. 600MB. We recommend using Wi-Fi.
                </div>
              </v-alert>

              <Setting :embedded="true" hide-folder-section @models-ready="checkModels" />

              <v-btn block color="black" height="56" class="siegu-btn mt-8" 
                :loading="isDownloadingModels"
                :disabled="downloadedModels.length < 2 && !isDownloadingModels" 
                @click="onboardingStep = 'sync'">
                {{ downloadedModels.length < 2 ? 'Download Required' : 'Continue' }}
              </v-btn>
            </v-card>
          </v-col>
        </v-row>
      </v-container>

      <!-- Step 4: Sync & Devices (Skippable) -->
      <v-container v-else-if="onboardingStep === 'sync'" class="fill-height bg-siegu-white" fluid>
        <v-row justify="center">
          <v-col cols="12" sm="10" md="8" lg="6">
            <v-card variant="flat" rounded="xl" class="pa-8 border-subtle">
              <div class="text-center mb-8">
                <div class="siegu-icon-circle mx-auto mb-4">
                  <v-icon color="white">mdi-cellphone-link</v-icon>
                </div>
                <h2 class="text-h4 font-weight-bold text-zinc-primary">Sync & Protect</h2>
                <p class="text-zinc-secondary">
                  Sync your library across your phone and other computers. 
                  This creates a private, serverless backup of your memories.
                </p>
              </div>

              <!-- Download Links for other platforms -->
              <div v-if="!showConnectUI" class="mb-8">
                <div class="text-caption font-weight-bold text-zinc-muted mb-4 tracking-widest uppercase text-center">Get Siegu on other devices</div>
                <v-row dense>
                  <v-col cols="6">
                    <v-btn block variant="flat" class="siegu-btn-sm" href="https://siegu.io/download/android" target="_blank">
                      <v-icon start size="small">mdi-android</v-icon>
                      Android
                    </v-btn>
                  </v-col>
                  <v-col cols="6">
                    <v-btn block variant="flat" class="siegu-btn-sm" href="https://siegu.io/download/ios" target="_blank">
                      <v-icon start size="small">mdi-apple</v-icon>
                      iOS
                    </v-btn>
                  </v-col>
                  <v-col cols="12" class="mt-2">
                    <v-btn block variant="flat" class="siegu-btn-sm" href="https://siegu.io/download" target="_blank">
                      <v-icon start size="small">mdi-monitor</v-icon>
                      Other Desktop
                    </v-btn>
                  </v-col>
                </v-row>
              </div>

              <div v-if="showConnectUI" class="d-flex justify-center mb-8">
                <Connect :embedded="true" @connected="deviceConnected = true" />
              </div>

              <v-fade-transition>
                <div v-if="deviceConnected" class="bg-success-light border-success pa-4 rounded-xl mb-6 text-center d-flex align-center justify-center">
                  <v-icon color="success" class="mr-2">mdi-check-circle</v-icon>
                  <span class="text-success font-weight-bold">Device Linked Successfully!</span>
                </div>
              </v-fade-transition>

              <div class="d-flex flex-column ga-3">
                <template v-if="!showConnectUI">
                  <v-btn block color="black" height="56" class="siegu-btn" @click="showConnectUI = true">
                    <v-icon start class="mr-2">mdi-link-variant</v-icon>
                    Link a Device
                  </v-btn>
                  <v-btn block variant="text" color="zinc-muted" @click="onboardingStep = 'finalize'">
                    Skip for now
                  </v-btn>
                </template>
                <template v-else>
                  <v-btn v-if="deviceConnected" block color="black" height="56" class="siegu-btn" @click="onboardingStep = 'finalize'">
                    Continue
                  </v-btn>
                  <v-btn v-else block variant="text" color="zinc-muted" @click="onboardingStep = 'finalize'">
                    Skip for now
                  </v-btn>
                </template>
              </div>
            </v-card>
          </v-col>
        </v-row>
      </v-container>

      <!-- Step 5: Finalize & Scan -->
      <v-container v-else-if="onboardingStep === 'finalize'" class="fill-height bg-siegu-white" fluid>
        <v-row justify="center">
          <v-col cols="12" sm="10" md="8" lg="6">
            <v-card variant="flat" rounded="xl" class="pa-8 border-subtle text-center">
              <div class="success-check-animation mb-8">
                <v-icon size="80" color="success">mdi-check-decagram</v-icon>
              </div>
              <h2 class="text-h3 font-weight-black text-zinc-primary mb-4">Ready to go!</h2>
              <p class="text-body-1 text-zinc-secondary mb-10">
                Setup is complete. Now it's time to find your memories.
              </p>

              <v-btn block color="black" height="64" class="siegu-btn mb-4" @click="finishSetupAndScan">
                <v-icon start class="mr-2">{{ deviceConnected ? 'mdi-sync' : 'mdi-magnify-scan' }}</v-icon>
                {{ deviceConnected ? 'Finish Setup & Sync' : 'Start Initial Scan' }}
              </v-btn>
              
              <div class="text-caption text-zinc-muted">
                This will find all your photos and start AI processing.
              </div>
            </v-card>
          </v-col>
        </v-row>
      </v-container>
    </template>

    <v-layout v-else class="bg-siegu-main">
      <v-app-bar elevation="0" v-if="current_page === 'home'" color="#ffffff" class="border-bottom-subtle px-2">
        <v-row class="px-2 align-center no-gutters">
          <v-col cols="auto">
            <v-menu offset-y transition="scale-transition">
              <template v-slot:activator="{ props }">
                <v-btn v-bind="props" color="#000000" theme="dark" variant="flat" :class="isMobile ? 'px-2' : 'px-4'" height="40" rounded="lg">
                  <div class="d-flex align-center">
                    <div :class="isMobile ? '' : 'mr-2'">
                      <v-progress-circular v-if="scanStatus === 'scanning' || indexingCount > 0" indeterminate size="16" width="2" color="white"></v-progress-circular>
                      <v-icon v-else size="18" color="white">mdi-sync</v-icon>
                    </div>
                    <span v-if="!isMobile" class="text-white font-weight-bold">{{ scanStatus === 'scanning' ? 'Scanning...' : (indexingCount > 0 ? 'Indexing...' : 'Refresh') }}</span>
                  </div>
                </v-btn>
              </template>
              <v-card min-width="320" border class="mt-2 border-subtle overflow-hidden" color="#ffffff" rounded="xl">
                <div class="bg-zinc-50 pa-4 border-bottom-subtle">
                  <div class="text-overline font-weight-black text-zinc-muted mb-1">LIBRARY STATUS</div>
                  <div class="d-flex align-center justify-space-between">
                    <div class="text-subtitle-1 font-weight-bold text-zinc-primary">Siegu Sync</div>
                    <v-chip v-if="scanStatus === 'scanning'" size="x-small" color="black" variant="flat" class="text-white">ACTIVE</v-chip>
                  </div>
                </div>

                <v-card-text class="pa-4">
                  <v-list density="compact" bg-color="transparent" class="pa-0">
                    <v-list-item class="px-0 mb-4">
                      <template v-slot:prepend>
                        <v-icon color="zinc-muted" class="mr-3">mdi-folder-outline</v-icon>
                      </template>
                      <v-list-item-title class="text-zinc-primary font-weight-bold">File Scanner</v-list-item-title>
                      <v-list-item-subtitle class="text-zinc-secondary">
                        {{ scanStatus === 'scanning' ? `Processing folder ${scanProgress.current}/${scanProgress.total}` : 'Idle' }}
                      </v-list-item-subtitle>
                      <div v-if="scanStatus === 'scanning'" class="mt-2">
                        <v-progress-linear :model-value="scanProgress.progress" color="black" height="4" rounded></v-progress-linear>
                      </div>
                    </v-list-item>

                    <v-list-item class="px-0">
                      <template v-slot:prepend>
                        <v-icon color="zinc-muted" class="mr-3">mdi-auto-fix</v-icon>
                      </template>
                      <v-list-item-title class="text-zinc-primary font-weight-bold">AI Intelligence</v-list-item-title>
                      <v-list-item-subtitle class="text-zinc-secondary">
                        {{ indexingCount > 0 ? `${indexingCount} faces remaining` : 'All memories indexed' }}
                      </v-list-item-subtitle>
                    </v-list-item>
                  </v-list>

                  <v-divider class="my-4 border-subtle"></v-divider>
                  
                  <div class="d-flex align-center justify-space-between mb-6">
                    <span class="text-caption text-zinc-muted">Last sync: <b>{{ lastScanTime }}</b></span>
                  </div>

                  <v-btn 
                    v-if="scanStatus !== 'scanning' && indexingCount === 0" 
                    @click="scan()" 
                    variant="flat" 
                    color="black"
                    block 
                    height="56"
                    class="siegu-btn"
                  >
                    <div class="d-flex align-center">
                      <div class="siegu-icon-circle mr-3">
                        <v-icon color="white">mdi-sync</v-icon>
                      </div>
                      <div class="text-left">
                        <div class="font-weight-bold text-white">Sync Library</div>
                        <div class="text-caption text-zinc-muted" style="font-size: 10px; opacity: 0.7;">Refresh files & AI index</div>
                      </div>
                    </div>
                  </v-btn>
                  <div v-else class="text-center py-2">
                    <v-progress-circular indeterminate color="black" size="24"></v-progress-circular>
                    <div class="text-caption mt-2 text-zinc-muted">Processing in background...</div>
                  </div>
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
              class="search-autocomplete w-100"
              bg-color="#ffffff"
              :menu-props="{ contentClass: 'siegu-list', elevation: 4, disabled: !objects.length && !filteredPeople.length }"
              no-data-text=""
            >
                <template v-slot:prepend-item>
                  <div v-if="filteredPeople.length > 0">
                    <v-list-subheader class="text-zinc-muted text-uppercase tracking-widest text-caption py-2">People</v-list-subheader>
                    <div class="pa-2 d-flex flex-nowrap overflow-x-auto ga-2">
                      <div v-for="person in filteredPeople" :key="person.id" class="d-flex flex-column align-center cursor-pointer min-w-60" @click="addPersonToSearch(person)">
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
                  <v-btn icon size="small" variant="text" v-bind="props" color="#18181b">
                    <v-badge :model-value="hasActiveFilters" color="black" dot px="1">
                      <v-icon size="20">mdi-filter-variant</v-icon>
                    </v-badge>
                  </v-btn>
                </template>
                <v-card min-width="250" border class="mt-2 border-subtle" color="#ffffff" rounded="xl">
                  <v-list bg-color="transparent" density="compact" class="px-2 ga-2">
                    <v-list-item class="px-0">
                      <v-switch v-model="filters.favoritesOnly" label="Favorites only" color="#000000" hide-details density="compact" inset class="text-zinc-secondary px-2"></v-switch>
                    </v-list-item>
                    <v-list-item class="px-0">
                      <v-switch v-model="filters.videosOnly" label="Videos only" color="#000000" hide-details density="compact" inset class="text-zinc-secondary px-2"></v-switch>
                    </v-list-item>
                    <v-divider class="border-subtle my-2"></v-divider>
                    <v-list-subheader class="text-zinc-muted text-uppercase tracking-widest text-caption px-0">Date Range</v-list-subheader>
                    <v-list-item class="px-0">
                      <v-btn-toggle v-model="filters.dateRange" mandatory variant="flat" density="compact" class="ga-2 w-100 bg-transparent">
                        <v-btn value="all" size="x-small" class="siegu-btn flex-grow-1">All</v-btn>
                        <v-btn value="month" size="x-small" class="siegu-btn flex-grow-1">Month</v-btn>
                        <v-btn value="year" size="x-small" class="siegu-btn flex-grow-1">Year</v-btn>
                      </v-btn-toggle>
                    </v-list-item>
                    <v-divider class="border-subtle my-2"></v-divider>
                    <v-list-subheader class="text-zinc-muted text-uppercase tracking-widest text-caption px-0">Folder</v-list-subheader>
                    <v-list-item class="px-0">
                      <v-select v-model="filters.folder" :items="directories" placeholder="All folders" variant="solo-filled" density="compact" hide-details flat rounded="lg" class="siegu-field"></v-select>
                    </v-list-item>
                  </v-list>
                  <v-card-actions class="pa-4">
                    <v-btn variant="flat" class="siegu-btn w-100 py-4" @click="resetFilters">
                       <div class="d-flex align-center">
                         <div class="siegu-icon-circle siegu-icon-circle-sm mr-2">
                           <v-icon size="12" color="white">mdi-refresh</v-icon>
                         </div>
                         <span class="text-white">Reset Filters</span>
                       </div>
                    </v-btn>
                  </v-card-actions>
                </v-card>
              </v-menu>
            </v-col>
          </v-row>
        </v-app-bar>
  
        <v-main class="bg-siegu-main">
          <Photos v-if="current_page === 'home'" :search-query="search" :filters="filters" @clear-search="search = null" />
          <People v-if="current_page === 'people'" @search-person="addPersonToSearch" />
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
