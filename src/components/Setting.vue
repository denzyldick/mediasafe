<template>
  <v-container class="pb-16" fluid>
    <v-row justify="center">
      <v-col cols="12" md="8" lg="6">
        <div class="d-flex align-center mb-6">
          <v-icon color="white" size="x-large" class="mr-3 opacity-80">mdi-cog</v-icon>
          <h1 class="text-h4 font-weight-bold text-white">Settings</h1>
        </div>

        <!-- Storage Config Card -->
        <v-card class="mb-6" color="white" variant="flat" rounded="lg">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon color="grey-darken-3" size="large">mdi-database</v-icon>
            </template>
            <v-card-title class="text-h6 text-grey-darken-4 font-weight-bold">Storage Configuration</v-card-title>
            <v-card-subtitle class="text-grey-darken-1">Where your media database is stored</v-card-subtitle>
          </v-card-item>

          <v-card-text>
            <v-alert
              variant="flat"
              border="start"
              color="grey-lighten-4"
              class="mb-2 text-grey-darken-4"
              style="border-left: 4px solid #18181b !important;"
            >
              Your configuration database is located at:
              <div class="font-weight-bold mt-1 text-body-2">{{ dataDir }}/database.sql</div>
            </v-alert>
          </v-card-text>
        </v-card>

        <!-- Library Config Card -->
        <v-card variant="flat" color="white" rounded="lg" class="mb-6">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon color="grey-darken-3" size="large">mdi-image-multiple</v-icon>
            </template>
            <v-card-title class="text-h6 text-grey-darken-4 font-weight-bold">Media Library</v-card-title>
            <v-card-subtitle class="text-grey-darken-1">Manage folders</v-card-subtitle>
          </v-card-item>

          <v-card-text>
            <v-row class="align-center mb-2 mt-2">
              <v-col>
                <div class="text-subtitle-1 font-weight-bold text-grey-darken-4">Watched Folders</div>
              </v-col>
              <v-col cols="auto">
                 <v-btn
                  color="grey-darken-4"
                  prepend-icon="mdi-folder-plus"
                  variant="flat"
                  @click="select_directory"
                  class="text-none font-weight-bold"
                >
                  Add Folder
                </v-btn>
              </v-col>
            </v-row>

            <v-expand-transition>
              <div v-if="directories.length > 0">
                 <v-list lines="one" class="bg-grey-lighten-4 rounded-lg">
                  <v-list-item
                    v-for="(directory, index) in directories"
                    :key="directory.value"
                    :title="directory.title"
                    class="text-grey-darken-4 font-weight-medium"
                  >
                    <template v-slot:prepend>
                       <v-icon color="grey-darken-1">mdi-folder</v-icon>
                    </template>
                    
                    <template v-slot:append>
                      <v-btn
                        icon="mdi-delete"
                        variant="text"
                        color="error"
                        size="small"
                        @click="remove_directory(directory.value)"
                        title="Remove folder"
                      ></v-btn>
                    </template>
                     <v-divider v-if="index < directories.length - 1" class="opacity-10"></v-divider>
                  </v-list-item>
                </v-list>
              </div>
              <div v-else class="text-center py-8 text-grey-darken-1 border border-dashed rounded-lg" style="border-color: rgba(0,0,0,0.1) !important;">
                <div>No folders added yet.</div>
              </div>
            </v-expand-transition>
          </v-card-text>
          
          <v-divider class="opacity-10"></v-divider>
          
          <v-card-actions class="pa-4">
             <v-spacer></v-spacer>
             <v-btn
              color="grey-darken-4"
              size="large"
              variant="flat"
              :disabled="directories.length === 0"
              @click="finishOnboarding"
              class="text-none px-8 font-weight-bold"
            >
              Continue
            </v-btn>
          </v-card-actions>
        </v-card>

        <!-- Performance Config Card -->
        <v-card variant="flat" color="white" rounded="lg" class="mb-6">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon color="grey-darken-3" size="large">mdi-speedometer</v-icon>
            </template>
            <v-card-title class="text-h6 text-grey-darken-4 font-weight-bold">Performance</v-card-title>
            <v-card-subtitle class="text-grey-darken-1">Adjust system resource usage</v-card-subtitle>
          </v-card-item>

          <v-card-text>
            <div class="mb-6">
              <div class="d-flex justify-space-between align-center mb-2">
                <div class="text-subtitle-1 font-weight-bold text-grey-darken-4">Scanning Threads</div>
                <v-chip size="x-small" color="grey-darken-4" variant="flat" class="font-weight-bold text-white">{{ performance.scanThreads }} threads</v-chip>
              </div>
              <v-slider
                v-model="performance.scanThreads"
                :min="1"
                :max="maxThreads"
                :step="1"
                thumb-label
                hide-details
                color="grey-darken-4"
                @update:model-value="savePerformanceConfig"
              ></v-slider>
              <div class="text-caption text-grey-darken-1 mt-1">Number of parallel threads used for initial file scanning and thumbnail generation.</div>
            </div>

            <v-divider class="my-4 opacity-10"></v-divider>

            <div>
              <v-list-item class="px-0">
                <v-list-item-title class="text-grey-darken-4 font-weight-bold">AI Indexing Mode</v-list-item-title>
                <v-list-item-subtitle class="text-grey-darken-1">When should the AI process your photos?</v-list-item-subtitle>
                <template v-slot:append>
                  <v-select
                    v-model="performance.indexingMode"
                    :items="[
                      { title: 'Immediate', value: 'immediate' },
                      { title: 'On Idle', value: 'idle' },
                      { title: 'Manual Only', value: 'manual' }
                    ]"
                    variant="solo-filled"
                    flat
                    density="compact"
                    hide-details
                    bg-color="grey-lighten-4"
                    class="custom-select font-weight-bold"
                    width="150"
                    @update:model-value="savePerformanceConfig"
                  ></v-select>
                </template>
              </v-list-item>
            </div>
          </v-card-text>
        </v-card>
        
        <!-- AI Models Config Card -->
        <v-card variant="flat" color="white" rounded="lg" class="mb-6">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon color="grey-darken-3" size="large">mdi-brain</v-icon>
            </template>
            <v-card-title class="text-h6 text-grey-darken-4 font-weight-bold">AI Models</v-card-title>
            <v-card-subtitle class="text-grey-darken-1">Offline detection & search</v-card-subtitle>
          </v-card-item>

          <v-card-text>
            <v-list lines="two" class="bg-transparent">
              <v-list-item class="px-0">
                <template v-slot:prepend>
                  <v-checkbox v-model="selectedModels" value="clip" hide-details class="mt-0" color="grey-darken-4" :disabled="downloadedModels.includes('clip')"></v-checkbox>
                </template>
                <v-list-item-title class="font-weight-bold text-grey-darken-4">
                  CLIP Model
                  <v-chip v-if="downloadedModels.includes('clip')" size="x-small" variant="flat" class="ml-2" color="success">Ready</v-chip>
                </v-list-item-title>
                <v-list-item-subtitle class="text-grey-darken-1">Smart search indexing</v-list-item-subtitle>
                <template v-if="downloadProgress.clip !== undefined">
                  <v-progress-linear
                    :model-value="(downloadProgress.clip.downloaded / downloadProgress.clip.total) * 100"
                    color="grey-darken-4"
                    height="2"
                    rounded
                    class="mt-2"
                  ></v-progress-linear>
                </template>
              </v-list-item>

              <v-list-item class="px-0">
                <template v-slot:prepend>
                  <v-checkbox v-model="selectedModels" value="ultraface" hide-details class="mt-0" color="grey-darken-4" :disabled="downloadedModels.includes('ultraface')"></v-checkbox>
                </template>
                <v-list-item-title class="font-weight-bold text-grey-darken-4">
                  UltraFace Model
                  <v-chip v-if="downloadedModels.includes('ultraface')" size="x-small" variant="flat" class="ml-2" color="success">Ready</v-chip>
                </v-list-item-title>
                <v-list-item-subtitle class="text-grey-darken-1">Offline face detection</v-list-item-subtitle>
                <template v-if="downloadProgress.ultraface !== undefined">
                  <v-progress-linear
                    :model-value="(downloadProgress.ultraface.downloaded / downloadProgress.ultraface.total) * 100"
                    color="grey-darken-4"
                    height="2"
                    rounded
                    class="mt-2"
                  ></v-progress-linear>
                </template>
              </v-list-item>
            </v-list>
          </v-card-text>
          
          <v-divider class="opacity-10"></v-divider>
          
          <v-card-actions class="pa-4">
             <v-spacer></v-spacer>
             <v-btn
              color="grey-darken-4"
              variant="flat"
              :disabled="selectedModels.length === 0 || isDownloading"
              :loading="isDownloading"
              @click="downloadModels"
              prepend-icon="mdi-download"
              class="text-none font-weight-bold"
            >
              Download
            </v-btn>
          </v-card-actions>
        </v-card>

        <!-- Debug Logs Expansion Panel -->
        <v-expansion-panels class="mb-6">
          <v-expansion-panel title="Debug Logs" bg-color="white" class="text-grey-darken-4 font-weight-bold">
            <template v-slot:text>
               <v-sheet
                 color="grey-lighten-4"
                 class="pa-2 overflow-y-auto"
                 rounded
                 height="300"
                 id="log-container"
               >
                 <div v-for="(log, i) in logs" :key="i" class="text-caption text-mono font-weight-medium" style="font-family: monospace; white-space: pre-wrap; word-break: break-all; color: #18181b;">
                   <span :class="log.type === 'error' ? 'text-red-darken-2' : 'text-grey-darken-3'">
                     [{{ log.time }}] {{ log.message }}
                   </span>
                 </div>
               </v-sheet>
               <v-btn size="small" variant="text" color="grey-darken-1" class="mt-2 text-none font-weight-bold" @click="logs = []">Clear Logs</v-btn>
            </template>
          </v-expansion-panel>
        </v-expansion-panels>
      </v-col>
    </v-row>
    <FolderPicker
        v-model="showFolderPicker"
        @select="onFolderSelected"
    />
  </v-container>
</template>
<script>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import * as path from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
import { platform } from "@tauri-apps/plugin-os";
import FolderPicker from "./FolderPicker.vue";
export default {
  components: {
    FolderPicker
  },
  data: () => ({
    directories: [],
    dataDir: null,
    logs: [],
    name: "",
    nameRules: [
      (v) => !!v || "Path is required",
      (v) => (v && v.length <= 10) || "Name must be 10 characters or less",
    ],
    select: null,
    items: ["Gpu", "Cpu"],
    checkbox: false,
    showFolderPicker: false,
    selectedModels: [],
    isDownloading: false,
    downloadProgress: {},
    isAndroid: false,
  }),

  methods: {
    formatBytes(bytes) {
      if (bytes === 0) return '0 B';
      const k = 1024;
      const sizes = ['B', 'KB', 'MB', 'GB'];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    },
    async downloadModels() {
      // Only download models that aren't already downloaded
      const modelsToDownload = this.selectedModels.filter(m => !this.downloadedModels.includes(m));
      if (modelsToDownload.length === 0) return;
      
      this.isDownloading = true;
      this.downloadProgress = {};
      
      try {
        await invoke('download_models', { models: modelsToDownload });
        console.log("Model downloads initiated!");
        // Refresh downloaded status after a bit or listen for complete (optional)
      } catch (err) {
        console.error("Failed to download models:", err);
      } finally {
        this.isDownloading = false;
      }
    },

    async select_directory() {
      if (this.isAndroid) {
        this.showFolderPicker = true;
        return;
      }

      try {
        const selection = await open({
          multiple: true,
          directory: true,
        });

        if (selection === null) {
          console.log("No directory selected");
          return;
        }

        const dirs = Array.isArray(selection) ? selection : [selection];

        for (const dir of dirs) {
          try {
            await invoke("add_directory", { path: dir });
            console.log("Added directory:", dir);
          } catch (err) {
            console.error("Failed to add directory:", err);
          }
        }
        
        this.list_directories();
        // Remove immediate scan_files call to prevent background CPU spike during onboarding
      } catch (err) {
        console.error("Error selecting directory:", err);
        if (err.toString().includes("not implemented") || err.toString().includes("picker")) {
             this.showFolderPicker = true;
        } else {
             alert("Failed to select directory: " + err);
        }
      }
    },
    async onFolderSelected(path) {
        console.log("Android folder selected:", path);
        try {
            await invoke("add_directory", { path: path });
            this.list_directories();
        } catch (err) {
            console.error("Failed to add directory:", err);
            alert("Failed to add directory: " + err);
        }
    },
    async finishOnboarding() {
        // Trigger the first scan only when onboarding is finished
        invoke("scan_files");
        this.$emit('done');
    },
    remove_directory(path) {
      this.directories = this.directories.filter((dir) => dir.value !== path);
      invoke("remove_directory", { path: path }).then(() => {
        this.list_directories();
      });
    },
    async checkExistingModels() {
        const downloaded = await invoke("check_models");
        this.downloadedModels = downloaded;
        // Auto-select downloaded models
        this.selectedModels = [...downloaded];
    },
    list_directories() {
      invoke("list_directories").then((response) => {
        const dirs = JSON.parse(response);
        this.directories = dirs.map(dir => ({
          title: dir,
          value: dir
        }));
      }).catch(err => {
        console.error("Failed to list directories:", err);
      });
    },
    async savePerformanceConfig() {
      try {
        await invoke("save_config", { key: "scan_threads", value: this.performance.scanThreads.toString() });
        await invoke("save_config", { key: "indexing_mode", value: this.performance.indexingMode });
      } catch (err) {
        console.error("Failed to save performance config:", err);
      }
    },
    async loadPerformanceConfig() {
      try {
        const threads = await invoke("get_config", { key: "scan_threads" });
        if (threads) this.performance.scanThreads = parseInt(threads);
        
        const mode = await invoke("get_config", { key: "indexing_mode" });
        if (mode) this.performance.indexingMode = mode;
      } catch (err) {
        console.error("Failed to load performance config:", err);
      }
    }
  },
  data: () => ({
    directories: [],
    downloadedModels: [],
    dataDir: null,
    logs: [],
    selectedModels: [],
    isDownloading: false,
    downloadProgress: {},
    isAndroid: false,
    showFolderPicker: false,
    performance: {
      scanThreads: 4,
      indexingMode: 'immediate'
    },
    maxThreads: 8,
  }),
  async mounted() {
    // Listen for background logs
    listen("log-message", (event) => {
      const log = {
        time: new Date().toLocaleTimeString(),
        message: event.payload,
        type: event.payload.toLowerCase().includes("error") ? "error" : "info"
      };
      this.logs.unshift(log); // Show newest logs at the top
      if (this.logs.length > 100) this.logs.pop(); // Keep last 100 logs
    });

    this.dataDir = await path.homeDir();
    await this.checkExistingModels();
    await this.loadPerformanceConfig();
    this.list_directories();
    // ... existing platform detection ...
  },
};
</script>
