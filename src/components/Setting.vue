<template>
  <v-container class="pb-16" fluid>
    <v-row justify="center">
      <v-col cols="12" md="8" lg="6">
        <div class="d-flex align-center mb-6">
          <v-icon color="primary" size="x-large" class="mr-3">mdi-cog</v-icon>
          <h1 class="text-h4 font-weight-bold text-grey-darken-3">Settings</h1>
        </div>

        <!-- Storage Config Card -->
        <v-card class="mb-6" elevation="2" rounded="lg">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon color="secondary" size="large">mdi-database</v-icon>
            </template>
            <v-card-title class="text-h6">Storage Configuration</v-card-title>
            <v-card-subtitle>Where your media database is stored</v-card-subtitle>
          </v-card-item>

          <v-card-text>
            <v-alert
              color="info"
              variant="tonal"
              icon="mdi-information"
              class="mb-2"
            >
              Your configuration database is located at:
              <div class="font-weight-bold mt-1 text-body-2">{{ dataDir }}/database.sql</div>
            </v-alert>
            <div class="text-caption text-grey">
              This is a standard <a href="https://www.sqlite.org" class="text-decoration-none font-weight-bold text-primary">SQLite</a> file. You can back it up or export it at any time.
            </div>
          </v-card-text>
        </v-card>

        <!-- Library Config Card -->
        <v-card elevation="2" rounded="lg">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon color="secondary" size="large">mdi-image-multiple</v-icon>
            </template>
            <v-card-title class="text-h6">Media Library</v-card-title>
            <v-card-subtitle>Manage which folders MediaSafe can access</v-card-subtitle>
          </v-card-item>

          <v-card-text>
            <v-alert
              type="success"
              variant="tonal"
              class="mb-4 text-body-2"
              icon="mdi-shield-check"
            >
              <strong>Privacy First:</strong> MediaSafe only scans the specific folders you add below. It will never access other parts of your system.
            </v-alert>

            <v-row class="align-center mb-2">
              <v-col>
                <div class="text-subtitle-1 font-weight-medium">Watched Folders</div>
              </v-col>
              <v-col cols="auto">
                 <v-btn
                  color="primary"
                  prepend-icon="mdi-folder-plus"
                  variant="flat"
                  @click="select_directory"
                >
                  Add Folder
                </v-btn>
              </v-col>
            </v-row>

            <v-expand-transition>
              <div v-if="directories.length > 0">
                 <v-list lines="one" class="rounded-0 border">
                  <v-list-item
                    v-for="(directory, index) in directories"
                    :key="directory.value"
                    :title="directory.title"
                  >
                    <template v-slot:prepend>
                       <v-icon color="amber-darken-2">mdi-folder</v-icon>
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
                     <v-divider v-if="index < directories.length - 1"></v-divider>
                  </v-list-item>
                </v-list>
              </div>
              <div v-else class="text-center py-8 text-grey-darken-1 border border-dashed rounded-lg">
                <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-folder-open-outline</v-icon>
                <div>No folders added yet.</div>
                <div class="text-caption">Click "Add Folder" to get started.</div>
              </div>
            </v-expand-transition>
          </v-card-text>
          
          <v-divider></v-divider>
          
          <v-card-actions class="pa-4">
             <v-spacer></v-spacer>
             <v-btn
              color="primary"
              size="large"
              variant="elevated"
              :disabled="directories.length === 0"
              @click="$emit('done')"
              append-icon="mdi-arrow-right"
            >
              Continue to Library
            </v-btn>
          </v-card-actions>
        </v-card>
        
        
        <!-- AI Models Config Card -->
        <v-card elevation="2" rounded="lg" class="mb-6">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon color="secondary" size="large">mdi-brain</v-icon>
            </template>
            <v-card-title class="text-h6">AI Models Configuration</v-card-title>
            <v-card-subtitle>Download models for smart search and face detection</v-card-subtitle>
          </v-card-item>

          <v-card-text>
            <v-alert
              color="info"
              variant="tonal"
              icon="mdi-information"
              class="mb-4 text-body-2"
            >
              Models are downloaded natively and run completely offline to preserve privacy.
            </v-alert>

            <v-list lines="two" class="bg-transparent">
              <v-list-item>
                <template v-slot:prepend>
                  <v-checkbox v-model="selectedModels" value="clip" hide-details class="mt-0"></v-checkbox>
                </template>
                <v-list-item-title class="font-weight-bold">CLIP Model</v-list-item-title>
                <v-list-item-subtitle>Enables smart zero-shot search (e.g., searching for "passport", "dog", "car" directly in your photos).</v-list-item-subtitle>
                <template v-if="downloadProgress.clip !== undefined">
                  <div class="mt-2 text-caption text-grey">
                    Downloading: {{ formatBytes(downloadProgress.clip.downloaded) }} / {{ formatBytes(downloadProgress.clip.total) }}
                  </div>
                  <v-progress-linear
                    :model-value="(downloadProgress.clip.downloaded / downloadProgress.clip.total) * 100"
                    color="primary"
                    height="6"
                    rounded
                    class="mt-1"
                  ></v-progress-linear>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-checkbox v-model="selectedModels" value="ultraface" hide-details class="mt-0"></v-checkbox>
                </template>
                <v-list-item-title class="font-weight-bold">UltraFace Model</v-list-item-title>
                <v-list-item-subtitle>Enables offline face detection to group photos by people.</v-list-item-subtitle>
                <template v-if="downloadProgress.ultraface !== undefined">
                  <div class="mt-2 text-caption text-grey">
                    Downloading: {{ formatBytes(downloadProgress.ultraface.downloaded) }} / {{ formatBytes(downloadProgress.ultraface.total) }}
                  </div>
                  <v-progress-linear
                    :model-value="(downloadProgress.ultraface.downloaded / downloadProgress.ultraface.total) * 100"
                    color="primary"
                    height="6"
                    rounded
                    class="mt-1"
                  ></v-progress-linear>
                </template>
              </v-list-item>
            </v-list>
          </v-card-text>
          
          <v-divider></v-divider>
          
          <v-card-actions class="pa-4">
             <v-spacer></v-spacer>
             <v-btn
              color="primary"
              variant="elevated"
              :disabled="selectedModels.length === 0 || isDownloading"
              :loading="isDownloading"
              @click="downloadModels"
              prepend-icon="mdi-download"
            >
              Download Selected Models
            </v-btn>
          </v-card-actions>
        </v-card>

        <!-- Debug Logs Expansion Panel -->
        <v-expansion-panels class="mb-6">
          <v-expansion-panel title="Debug Logs">
            <template v-slot:text>
               <v-sheet
                 color="grey-darken-4"
                 class="pa-2 overflow-y-auto"
                 rounded
                 height="300"
                 id="log-container"
               >
                 <div v-for="(log, i) in logs" :key="i" class="text-caption text-mono font-weight-light" style="font-family: monospace; white-space: pre-wrap; word-break: break-all;">
                   <span :class="log.type === 'error' ? 'text-red-accent-2' : 'text-grey-lighten-2'">
                     [{{ log.time }}] {{ log.message }}
                   </span>
                 </div>
               </v-sheet>
               <v-btn size="small" variant="text" color="grey" class="mt-2" @click="logs = []">Clear Logs</v-btn>
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
      if (this.selectedModels.length === 0) return;
      this.isDownloading = true;
      this.downloadProgress = {};
      
      try {
        await invoke('download_models', { models: this.selectedModels });
        console.log("Model downloads initiated or completed!");
        // The __RELOAD_MODELS__ signal is sent automatically by the backend now
      } catch (err) {
        console.error("Failed to download models:", err);
      } finally {
        this.isDownloading = false;
        // Optionally keep progress bars at 100% or clear them. We'll leave them to show they finished.
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
            await invoke("add_directory", { path: dir, configPath: this.dataDir });
            console.log("Added directory:", dir);
          } catch (err) {
            console.error("Failed to add directory:", err);
          }
        }
        
        this.list_directories();
        invoke("scan_files");
      } catch (err) {
        console.error("Error selecting directory:", err);
        // Fallback or specific error handling for Android if platform check failed
        if (err.toString().includes("not implemented") || err.toString().includes("picker")) {
             console.log("Falling back to custom folder picker due to error");
             this.showFolderPicker = true;
        } else {
             alert("Failed to select directory: " + err);
        }
      }
    },
    async onFolderSelected(path) {
        console.log("Android folder selected:", path);
        try {
            await invoke("add_directory", { path: path, configPath: this.dataDir });
            this.list_directories();
            invoke("scan_files");
        } catch (err) {
            console.error("Failed to add directory:", err);
            alert("Failed to add directory: " + err);
        }
    },
    remove_directory(path) {
      this.directories = this.directories.filter((dir) => dir.value !== path);
      invoke("remove_directory", { path: path, configPath: this.dataDir }).then(() => {
        this.list_directories();
      });
    },
    async validate() {
      const { valid } = await this.$refs.form.validate();

      if (valid) alert("Form is valid");
    },
    reset() {
      this.$refs.form.reset();
    },
    resetValidation() {
      this.$refs.form.resetValidation();
    },
    list_directories() {
      console.log("Listing directories. Config path:", this.dataDir);
      invoke("list_directories", { path: this.dataDir }).then((response) => {
        console.log("Raw response from list_directories:", response);
        const dirs = JSON.parse(response);
        console.log("Parsed directories:", dirs);
        this.directories = dirs.map(dir => ({
          title: dir,
          value: dir,
          props: {
            color: "primary",
            prependIcon: "mdi-folder",
            appendIcon: "mdi-close",
          }
        }));
      }).catch(err => {
        console.error("Failed to list directories:", err);
      });
    },
  },
  async mounted() {
    // Intercept console Logs
    const originalLog = console.log;
    const originalError = console.error;
    
    const addLog = (type, args) => {
        const message = args.map(arg => 
            typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
        ).join(' ');
        
        this.logs.push({
            time: new Date().toLocaleTimeString(),
            type,
            message
        });
        
        // Auto-scroll
        this.$nextTick(() => {
            const container = document.getElementById('log-container');
            if (container) container.scrollTop = container.scrollHeight;
        });
    };

    console.log = (...args) => {
        addLog('info', args);
        originalLog.apply(console, args);
    };
    
    console.error = (...args) => {
        addLog('error', args);
        originalError.apply(console, args);
    };


    // Listen for backend logs
    await listen('log-message', (event) => {
        addLog('info', ["[Backend]", event.payload]);
    });

    // Listen for ML model download progress
    await listen('download-progress', (event) => {
        const payload = event.payload;
        // Due to reactivity of deeply nested objects in some vue versions, assigning a new object is safer
        this.downloadProgress = {
            ...this.downloadProgress,
            [payload.model]: {
                downloaded: payload.downloaded,
                total: payload.total || (payload.downloaded + 1) // prevent zero division
            }
        };
    });


    this.dataDir = await path.homeDir();
    
    try {
        const platformName = await platform();
        console.log("Platform detected via plugin-os:", platformName);
        this.isAndroid = platformName === 'android';
        
        // Double check using error workaround logic if needed, but let's rely on log first.
        if (this.isAndroid) {
            console.log("Android mode enabled");
        }
    } catch (e) {
        console.warn("Failed to detect platform:", e);
        // Fallback heuristic: check user agent or specific tauri window properties if needed
         if (navigator.userAgent.toLowerCase().includes("android")) {
             console.log("Detected android via UserAgent fallback");
             this.isAndroid = true;
         }
    }

    this.list_directories();
  },
};
</script>
