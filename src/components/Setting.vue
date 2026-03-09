<template>
  <v-container :class="embedded ? 'pa-0' : 'pb-16 bg-siegu-main'" fluid>
    <v-row justify="center">
      <v-col cols="12" :md="embedded ? 12 : 8" :lg="embedded ? 12 : 6">
        <div v-if="!embedded" class="d-flex align-center justify-space-between mb-8">
          <div>
            <div class="d-flex align-center mb-1">
              <v-icon color="#18181b" size="28" class="mr-3">mdi-cog-outline</v-icon>
              <h1 class="text-h4 font-weight-bold text-zinc-primary">Settings</h1>
            </div>
            <div class="text-subtitle-1 text-zinc-secondary">Configure your library and AI preferences</div>
          </div>
        </div>

        <!-- Authorized Folders Card -->
        <v-card v-if="!hideFolderSection" variant="flat" color="#ffffff" rounded="xl" class="mb-6 overflow-hidden border-subtle">
          <v-card-item class="bg-zinc-100 py-4">
            <template v-slot:prepend>
              <div class="siegu-icon-circle-dark mr-3">
                <v-icon color="#ffffff" size="small">mdi-folder-lock</v-icon>
              </div>
            </template>
            <v-card-title class="text-h6 text-zinc-primary font-weight-bold">Authorized Folders</v-card-title>
          </v-card-item>

          <v-card-text class="pt-4">
            <v-expand-transition>
              <div v-if="directories.length > 0">
                <v-list bg-color="transparent">
                  <v-list-item
                    v-for="(directory, index) in directories"
                    :key="directory.value"
                    class="px-0"
                  >
                    <template v-slot:prepend>
                      <v-icon color="#71717a" class="mr-2">mdi-folder</v-icon>
                    </template>
                    <v-list-item-title class="text-zinc-primary font-weight-medium text-truncate">{{ directory.title }}</v-list-item-title>
                    <v-list-item-subtitle class="text-zinc-muted text-caption text-truncate">{{ directory.value }}</v-list-item-subtitle>
                    <template v-slot:append>
                      <v-menu>
                        <template v-slot:activator="{ props }">
                          <v-btn icon="mdi-dots-vertical" variant="text" size="small" color="#71717a" v-bind="props"></v-btn>
                        </template>
                        <v-list size="small" class="siegu-list">
                          <v-list-item @click="remove_directory(directory.value)">
                            <v-list-item-title>Remove Folder Reference</v-list-item-title>
                          </v-list-item>
                          <v-list-item @click="remove_directory_full(directory.value)" color="error">
                            <v-list-item-title>Wipe Local Data & Remove</v-list-item-title>
                          </v-list-item>
                        </v-list>
                      </v-menu>
                    </template>
                     <v-divider v-if="index < directories.length - 1" class="border-subtle"></v-divider>
                  </v-list-item>
                </v-list>
              </div>
              <div v-else class="text-center py-8 text-zinc-muted border border-dashed rounded-lg border-subtle">
                <div>No folders added yet.</div>
              </div>
            </v-expand-transition>
          </v-card-text>

          <v-card-actions class="pa-4 bg-zinc-50 border-top-subtle">
            <v-btn
              variant="flat"
              color="#000000"
              theme="dark"
              @click="select_directory"
              block
              height="48"
              class="siegu-btn rounded-xl"
            >
              <div class="d-flex align-center">
                <div class="siegu-icon-circle siegu-icon-circle-sm mr-2">
                  <v-icon size="14" color="white">mdi-folder-plus</v-icon>
                </div>
                <span class="text-white font-weight-bold">Add Folder</span>
              </div>
            </v-btn>
          </v-card-actions>
        </v-card>


        <!-- AI Models Card -->
        <v-card v-if="!hideAiSection" variant="flat" color="#ffffff" rounded="xl" class="mb-6 overflow-hidden border-subtle">
          <v-card-item class="bg-zinc-100 py-4">
            <template v-slot:prepend>
              <div class="siegu-icon-circle-dark mr-3">
                <v-icon color="#ffffff" size="small">mdi-robot-outline</v-icon>
              </div>
            </template>
            <v-card-title class="text-h6 text-zinc-primary font-weight-bold">AI Models</v-card-title>
          </v-card-item>

          <v-card-text class="pt-4">
            <v-list lines="two" class="bg-transparent">
              <v-list-item class="px-0">
                <template v-slot:prepend>
                  <v-checkbox v-model="selectedModels" value="clip" hide-details class="mt-0 siegu-checkbox" color="black"></v-checkbox>
                </template>
                <template v-slot:title>
                  <div class="font-weight-bold text-zinc-primary d-flex align-center">
                    CLIP Model
                    <v-chip v-if="downloadedModels.includes('clip')" size="x-small" variant="flat" class="ml-2" color="success">Ready</v-chip>
                  </div>
                </template>
                <template v-slot:subtitle>
                  <span class="text-zinc-secondary">Semantic search and content classification (350MB)</span>
                </template>
                <div v-if="downloadProgress['clip-visual'] || downloadProgress['clip-text'] || downloadProgress['clip-tokenizer']" class="mt-2 px-2">
                  <v-progress-linear
                    :model-value="getProgress('clip')"
                    color="black"
                    bg-color="#f4f4f5"
                    height="4"
                    rounded
                  ></v-progress-linear>
                </div>
              </v-list-item>

              <v-list-item class="px-0">
                <template v-slot:prepend>
                  <v-checkbox v-model="selectedModels" value="ultraface" hide-details class="mt-0 siegu-checkbox" color="black"></v-checkbox>
                </template>
                <template v-slot:title>
                  <div class="font-weight-bold text-zinc-primary d-flex align-center">
                    UltraFace Model
                    <v-chip v-if="downloadedModels.includes('ultraface')" size="x-small" variant="flat" class="ml-2" color="success">Ready</v-chip>
                  </div>
                </template>
                <template v-slot:subtitle>
                  <span class="text-zinc-secondary">Fast local face detection and grouping (2MB)</span>
                </template>
                <div v-if="downloadProgress.ultraface" class="mt-2 px-2">
                  <v-progress-linear
                    :model-value="getProgress('ultraface')"
                    color="black"
                    bg-color="#f4f4f5"
                    height="4"
                    rounded
                  ></v-progress-linear>
                </div>
              </v-list-item>
            </v-list>
          </v-card-text>

          <v-card-actions class="pa-4 bg-zinc-50">
             <v-spacer></v-spacer>
             <v-btn
              variant="flat"
              color="#000000"
              theme="dark"
              :disabled="selectedModels.length === 0 || isDownloading"
              :loading="isDownloading"
              @click="confirmDownload()"
              class="siegu-btn px-6"
              height="44"
            >
              <div class="d-flex align-center">
                <div class="siegu-icon-circle siegu-icon-circle-sm mr-3">
                  <v-icon size="14" color="white">mdi-download</v-icon>
                </div>
                <span class="text-white font-weight-bold">{{ downloadedModels.length === 2 ? 'Update Models' : 'Download Models' }}</span>
              </div>
            </v-btn>
          </v-card-actions>
        </v-card>

        <!-- Maintenance Section -->
        <v-card v-if="!embedded" variant="flat" color="#ffffff" rounded="xl" class="mb-6 border-subtle overflow-hidden">
          <v-card-item class="bg-zinc-100 py-4">
            <template v-slot:prepend>
              <div class="siegu-icon-circle-dark mr-3">
                <v-icon color="#ffffff" size="small">mdi-wrench-outline</v-icon>
              </div>
            </template>
            <v-card-title class="text-h6 text-zinc-primary font-weight-bold">Maintenance</v-card-title>
          </v-card-item>

          <v-card-text class="pt-2">
            <v-list lines="two" class="bg-transparent">
              <v-list-item class="px-0">
                <template v-slot:title>
                  <span class="font-weight-bold text-zinc-primary">Background Sync</span>
                </template>
                <template v-slot:subtitle>
                  <span class="text-zinc-secondary">Allow syncing when app is in background</span>
                </template>
                <template v-slot:append>
                  <v-switch v-model="bgSync" hide-details color="black" inset density="compact"></v-switch>
                </template>
              </v-list-item>

              <v-list-item class="px-0">
                <template v-slot:title>
                  <span class="font-weight-bold text-zinc-primary">Cleanup Database</span>
                </template>
                <template v-slot:subtitle>
                  <span class="text-zinc-secondary">Optimize storage and remove orphaned entries</span>
                </template>
                <template v-slot:append>
                  <v-btn size="small" variant="flat" @click="cleanupDb" :loading="isCleaning" class="siegu-btn px-4">
                    <div class="d-flex align-center">
                      <div class="siegu-icon-circle siegu-icon-circle-md mr-3">
                        <v-icon color="#ffffff" size="small">mdi-wrench-outline</v-icon>
                      </div>
                      <span class="text-white">Run</span>
                    </div>
                  </v-btn>
                </template>
              </v-list-item>
            </v-list>

            <v-divider class="my-4 border-subtle"></v-divider>

            <!-- Advanced Performance (Hidden for normal users) -->
            <v-expansion-panels variant="flat" class="advanced-perf">
              <v-expansion-panel bg-color="transparent">
                <v-expansion-panel-title class="px-0 text-zinc-muted text-caption font-weight-bold">
                  ADVANCED PERFORMANCE SETTINGS
                </v-expansion-panel-title>
                <v-expansion-panel-text class="px-0">
                  <div class="pt-2">
                    <div class="d-flex justify-space-between align-center mb-2">
                      <div class="text-caption font-weight-bold text-zinc-primary">Scanning Threads</div>
                      <v-chip size="x-small" color="#000000" variant="flat" class="font-weight-bold text-white">{{ performance.scanThreads }}</v-chip>
                    </div>
                    <v-slider
                      v-model="performance.scanThreads"
                      :min="1"
                      :max="maxThreads"
                      :step="1"
                      hide-details
                      color="black"
                      track-color="#f4f4f5"
                      @update:model-value="savePerformanceConfig"
                    ></v-slider>

                    <v-list-item class="px-0 mt-4">
                      <v-list-item-title class="text-caption font-weight-bold text-zinc-primary">Indexing Mode</v-list-item-title>
                      <template v-slot:append>
                        <v-menu offset-y>
                          <template v-slot:activator="{ props }">
                            <v-btn variant="tonal" size="x-small" color="black" v-bind="props" class="font-weight-bold">
                              {{ getModeLabel(performance.indexingMode) }}
                              <v-icon size="12" class="ml-1">mdi-chevron-down</v-icon>
                            </v-btn>
                          </template>
                          <v-list density="compact" class="siegu-list">
                            <v-list-item v-for="mode in indexingModes" :key="mode.value" @click="setIndexingMode(mode.value)">
                              <v-list-item-title class="text-caption" :class="{'font-weight-bold': performance.indexingMode === mode.value}">{{ mode.title }}</v-list-item-title>
                            </v-list-item>
                          </v-list>
                        </v-menu>
                      </template>
                    </v-list-item>
                  </div>
                </v-expansion-panel-text>
              </v-expansion-panel>
            </v-expansion-panels>
          </v-card-text>
        </v-card>

        <!-- Debug Logs Expansion Panel -->
        <v-expansion-panels v-if="!embedded" class="mb-6 siegu-expansion">
          <v-expansion-panel bg-color="#ffffff" class="text-zinc-primary font-weight-bold border-subtle">
            <template v-slot:title>
              <div class="d-flex align-center justify-space-between w-100">
                <span>Debug Logs</span>
              </div>
            </template>
            <template v-slot:text>
               <v-sheet
                color="#f4f4f5"
                class="pa-4 rounded-lg overflow-y-auto border-subtle debug-logs-sheet"
                max-height="400"
              >
                <div v-for="(log, i) in logs" :key="i" :class="log.type === 'error' ? 'text-error' : 'text-zinc-secondary'" class="mb-1">
                  <span class="text-zinc-muted">[{{ log.time }}]</span> {{ log.message }}
                </div>
                <div v-if="logs.length === 0" class="text-zinc-muted text-center py-4">No logs recorded yet.</div>
                
                <div v-if="logs.length > 0" class="mt-6 d-flex justify-center">
                  <v-btn 
                    variant="text" 
                    size="small" 
                    class="text-none font-weight-bold" 
                    color="error"
                    prepend-icon="mdi-trash-can-outline"
                    @click.stop="clearLogs"
                  >
                    Clear Log History
                  </v-btn>
                </div>
              </v-sheet>
            </template>
          </v-expansion-panel>
        </v-expansion-panels>
      </v-col>
    </v-row>
    <!-- Download Confirmation Dialog -->
    <v-dialog v-model="downloadDialog.show" max-width="400" persistent rounded="xl">
      <v-card color="#ffffff" border class="border-subtle overflow-hidden">
        <v-card-item class="bg-zinc-100 py-4">
          <template v-slot:prepend>
            <div class="siegu-icon-circle-dark mr-3">
              <v-icon color="#ffffff" size="small">mdi-cloud-download-outline</v-icon>
            </div>
          </template>
          <v-card-title class="text-h6 text-zinc-primary font-weight-bold">{{ downloadDialog.title }}</v-card-title>
        </v-card-item>
        
        <v-card-text class="py-6 text-center">
          <div class="text-subtitle-1 text-zinc-secondary px-2">
            {{ downloadDialog.message }}
          </div>
        </v-card-text>

        <v-card-actions class="pa-4 bg-zinc-50 border-top-subtle ga-2">
          <v-btn variant="tonal" block color="zinc-muted" @click="downloadDialog.show = false" class="siegu-btn flex-grow-1" height="44">Cancel</v-btn>
          <v-btn variant="flat" block color="black" @click="startConfirmedDownload" class="siegu-btn flex-grow-1" height="44">Download</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <FolderPicker
        v-model="showFolderPicker"
        @select="onFolderSelected"
    />
  </v-container>
</template>

<script>
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { homeDir } from "@tauri-apps/api/path";
import { platform } from "@tauri-apps/plugin-os";
import { listen } from "@tauri-apps/api/event";
import FolderPicker from "./FolderPicker.vue";

export default {
  name: "Setting",
  components: { FolderPicker },
  props: {
    embedded: { type: Boolean, default: false },
    hideAiSection: { type: Boolean, default: false },
    hideFolderSection: { type: Boolean, default: false }
  },
  emits: ["folder-added", "models-ready", "done"],
  data: () => ({
    directories: [],
    showFolderPicker: false,
    isAndroid: false,
    dataDir: "",
    configDir: "",
    checkResults: "",
    isDownloading: false,
    isCleaning: false,
    bgSync: false,
    downloadedModels: [],
    selectedModels: [],
    downloadProgress: {},
    logs: [],
    performance: {
      scanThreads: 4,
      indexingMode: "immediate",
    },
    maxThreads: 8,
    indexingModes: [
      { title: "Immediate", value: "immediate" },
      { title: "Idle Only", value: "idle" },
      { title: "Manual Only", value: "manual" },
    ],
    downloadDialog: {
      show: false,
      title: "",
      message: "",
      models: []
    },
  }),
  async mounted() {
    listen("log-message", (event) => {
      const log = {
        time: new Date().toLocaleTimeString(),
        message: event.payload,
        type: event.payload.toLowerCase().includes("error") ? "error" : "info"
      };
      this.logs.unshift(log);
      if (this.logs.length > 100) this.logs.pop();
    });

    listen("download-progress", (event) => {
        const { model, downloaded, total } = event.payload;
        this.downloadProgress = { ...this.downloadProgress, [model]: { downloaded, total } };
    });

    listen("download-complete", () => {
        this.isDownloading = false;
        this.checkExistingModels();
        this.$emit('models-ready');
    });

    this.dataDir = await homeDir();
    this.isAndroid = (await platform()) === 'android';
    await this.checkExistingModels();
    await this.loadPerformanceConfig();

    const bgSyncVal = await invoke("get_config", { key: "bg_sync" });
    this.bgSync = bgSyncVal === "true";
    this.fetchLogs();
    this.list_directories();
  },
  methods: {
    async fetchLogs() {
      try {
        const logsStr = await invoke("get_logs", { limit: 100 });
        const parsed = JSON.parse(logsStr);
        this.logs = parsed.map(l => ({
          time: new Date(l.timestamp).toLocaleTimeString(),
          message: l.message,
          type: l.level === 'error' ? 'error' : 'info'
        }));
      } catch (err) {}
    },
    async clearLogs() {
      await invoke("clear_logs");
      this.logs = [];
    },
    getModeLabel(val) {
      return this.indexingModes.find(m => m.value === val)?.title || val;
    },
    async loadPerformanceConfig() {
      const configStr = await invoke("get_config");
      const config = JSON.parse(configStr);

      if (config.scan_threads) {
        const val = parseInt(config.scan_threads);
        if (!isNaN(val)) this.performance.scanThreads = val;
      }

      if (config.indexing_mode) {
        this.performance.indexingMode = config.indexing_mode;
      }
    },
    async savePerformanceConfig() {
      await invoke("save_config", { key: "scan_threads", value: this.performance.scanThreads.toString() });
    },
    async setIndexingMode(mode) {
      this.performance.indexingMode = mode;
      await invoke("save_config", { key: "indexing_mode", value: mode });
    },
    async checkExistingModels() {
        const downloaded = await invoke("check_models");
        this.downloadedModels = downloaded;
        this.checkResults = JSON.stringify(downloaded);
        this.selectedModels = ["clip", "ultraface"];
        // Auto-download removed for better user experience/data saving
    },
    confirmDownload() {
      const missing = ["clip", "ultraface"].filter(m => !this.downloadedModels.includes(m));
      if (missing.length === 0) {
        this.downloadModels(true); // Force update if already ready
        return;
      }
      
      let size = "approx. 1MB";
      if (missing.includes('clip')) size = "approx. 600MB";
      
      this.downloadDialog = {
        show: true,
        title: "Download AI Models?",
        message: `Siegu needs to download ${missing.join(" and ")} models (${size}) to enable local search and face detection. This will use your data connection.`,
        models: missing
      };
    },
    startConfirmedDownload() {
      this.downloadDialog.show = false;
      this.downloadModels(false);
    },
    getProgress(model) {
      if (model === 'clip') {
        const parts = ['clip-visual', 'clip-text', 'clip-tokenizer'];
        let downloaded = 0;
        let total = 0;
        parts.forEach(p => {
          if (this.downloadProgress[p]) {
            downloaded += this.downloadProgress[p].downloaded;
            total += this.downloadProgress[p].total || 0;
          }
        });
        if (total === 0) return this.downloadedModels.includes('clip') ? 100 : 0;
        return (downloaded / total) * 100;
      }
      const progress = this.downloadProgress[model];
      if (!progress || !progress.total) return this.downloadedModels.includes(model) ? 100 : 0;
      return (progress.downloaded / progress.total) * 100;
    },
    formatBytes(bytes) {
      if (!bytes) return '0 B';
      const k = 1024;
      const sizes = ['B', 'KB', 'MB', 'GB'];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    },
    async downloadModels(forceUpdate = false) {
      let modelsToDownload = this.selectedModels;
      if (!forceUpdate) {
        modelsToDownload = ["clip", "ultraface"].filter(m => !this.downloadedModels.includes(m));
      }
      if (modelsToDownload.length === 0) return;
      this.isDownloading = true;
      this.downloadProgress = {};
      try {
        await invoke('download_models', { models: modelsToDownload });
      } catch (err) {
        this.isDownloading = false;
      }
    },
    async cleanupDb() {
      this.isCleaning = true;
      await invoke("cleanup_database");
      this.isCleaning = false;
    },
    async remove_directory_full(path) {
      if (confirm("This will remove the folder AND delete all indexed AI data. Continue?")) {
        await invoke("remove_directory_full", { path });
        this.list_directories();
      }
    },
    async select_directory() {
      if (this.isAndroid) {
        this.showFolderPicker = true;
        return;
      }
      try {
        const selection = await open({ multiple: true, directory: true });
        if (Array.isArray(selection)) {
          for (const path of selection) { await invoke("add_directory", { path }); }
        } else if (selection) {
          await invoke("add_directory", { path: selection });
        }
        this.list_directories();
      } catch (err) {}
    },
    list_directories() {
      invoke("list_directories").then((response) => {
        const dirs = JSON.parse(response);
        this.directories = dirs.map(dir => ({ title: dir.split('/').pop() || dir, value: dir }));
      });
    },
    remove_directory(path) {
      invoke("remove_directory", { path }).then(() => { this.list_directories(); });
    },
    onFolderSelected(path) {
      invoke("add_directory", { path }).then(() => {
        this.list_directories();
        this.$emit('folder-added', path);
      });
    }
  }
};
</script>

<style scoped>
.bg-zinc-100 {
  background-color: #f4f4f5 !important;
}
.siegu-expansion :deep(.v-expansion-panel-title) {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}
</style>
