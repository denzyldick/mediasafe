<template>
  <v-container class="pb-16 bg-siegu-main" fluid>
    <v-row justify="center">
      <v-col cols="12" md="8" lg="6">
        <div class="d-flex align-center justify-space-between mb-8">
          <div>
            <div class="d-flex align-center mb-1">
              <v-icon color="#18181b" size="28" class="mr-3">mdi-cog-outline</v-icon>
              <h1 class="text-h4 font-weight-bold text-zinc-primary">Settings</h1>
            </div>
            <div class="text-subtitle-1 text-zinc-secondary">Configure your library and AI preferences</div>
          </div>
        </div>

        <!-- Authorized Folders Card -->
        <v-card variant="flat" color="#ffffff" rounded="xl" class="mb-6 overflow-hidden border-subtle">
          <v-card-item class="bg-zinc-100 py-4">
            <template v-slot:prepend>
              <div class="siegu-icon-circle-dark mr-3">
                <v-icon color="#ffffff" size="small">mdi-folder-lock</v-icon>
              </div>
            </template>
            <v-card-title class="text-h6 text-zinc-primary font-weight-bold">Authorized Folders</v-card-title>
            <template v-slot:append>
              <v-btn
                variant="flat"
                color="#000000"
                theme="dark"
                @click="select_directory"
                class="siegu-btn px-6"
              >
                <div class="d-flex align-center">
                  <div class="siegu-icon-circle siegu-icon-circle-sm mr-2">
                    <v-icon size="14" color="white">mdi-folder-plus</v-icon>
                  </div>
                  <span class="text-white font-weight-bold">Add Folder</span>
                </div>
              </v-btn>
            </template>
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
                    <v-list-item-title class="text-zinc-primary font-weight-medium">{{ directory.title }}</v-list-item-title>
                    <v-list-item-subtitle class="text-zinc-muted text-caption">{{ directory.value }}</v-list-item-subtitle>
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
        </v-card>

        <!-- Performance Config Card -->
        <v-card variant="flat" color="#ffffff" rounded="xl" class="mb-6 border-subtle">
          <v-card-item class="bg-zinc-100 py-4">
            <template v-slot:prepend>
              <div class="siegu-icon-circle-dark mr-3">
                <v-icon color="#ffffff" size="small">mdi-speedometer</v-icon>
              </div>
            </template>
            <v-card-title class="text-h6 text-zinc-primary font-weight-bold">Performance</v-card-title>
          </v-card-item>

          <v-card-text class="pt-6">
            <div class="mb-6 px-2">
              <div class="d-flex justify-space-between align-center mb-2">
                <div class="text-subtitle-1 font-weight-bold text-zinc-primary">Scanning Threads</div>
                <v-chip size="x-small" color="#000000" variant="flat" class="font-weight-bold text-white">{{ performance.scanThreads }} threads</v-chip>
              </div>
              <v-slider
                v-model="performance.scanThreads"
                :min="1"
                :max="maxThreads"
                :step="1"
                thumb-label
                hide-details
                color="black"
                track-color="#f4f4f5"
                @update:model-value="savePerformanceConfig"
              ></v-slider>
            </div>

            <v-divider class="my-4 border-subtle"></v-divider>

            <div>
              <v-list-item class="px-0">
                <v-list-item-title class="text-zinc-primary font-weight-bold">AI Indexing Mode</v-list-item-title>
                <v-list-item-subtitle class="text-zinc-secondary">When should the AI process your photos?</v-list-item-subtitle>
                <template v-slot:append>
                  <v-menu offset-y>
                    <template v-slot:activator="{ props }">
                      <v-btn
                        variant="flat"
                        class="siegu-btn px-4"
                        v-bind="props"
                      >
                        <div class="d-flex align-center">
                          <span class="text-white mr-2 text-none font-weight-bold">{{ getModeLabel(performance.indexingMode) }}</span>
                          <v-icon size="14" color="white">mdi-chevron-down</v-icon>
                        </div>
                      </v-btn>
                    </template>
                    <v-list class="siegu-list">
                      <v-list-item v-for="mode in indexingModes" :key="mode.value" @click="setIndexingMode(mode.value)">
                        <v-list-item-title :class="{'font-weight-bold text-black': performance.indexingMode === mode.value}">{{ mode.title }}</v-list-item-title>
                      </v-list-item>
                    </v-list>
                  </v-menu>
                </template>
              </v-list-item>
            </div>
          </v-card-text>
        </v-card>

        <!-- AI Models Section -->
        <v-card variant="flat" color="#ffffff" rounded="xl" class="mb-6 overflow-hidden border-subtle">
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
                <div v-if="downloadProgress.clip" class="mt-2 px-2">
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
              @click="downloadModels(true)"
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
        <v-card variant="flat" color="#ffffff" rounded="xl" class="mb-6 border-subtle overflow-hidden">
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
          </v-card-text>
        </v-card>

        <!-- Debug Logs Expansion Panel -->
        <v-expansion-panels class="mb-6 siegu-expansion">
          <v-expansion-panel bg-color="#ffffff" class="text-zinc-primary font-weight-bold border-subtle">
            <template v-slot:title>
              <div class="d-flex align-center justify-space-between w-100">
                <span>Debug Logs</span>
                <v-btn size="x-small" variant="flat" class="siegu-btn px-4" @click.stop="clearLogs">
                   <div class="d-flex align-center">
                     <div class="siegu-icon-circle siegu-icon-circle-sm mr-2">
                       <v-icon size="8" color="white">mdi-trash-can-outline</v-icon>
                     </div>
                     <span class="text-white">Clear History</span>
                   </div>
                </v-btn>
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
              </v-sheet>
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
import { open } from "@tauri-apps/plugin-dialog";
import { homeDir } from "@tauri-apps/api/path";
import { platform } from "@tauri-apps/plugin-os";
import { listen } from "@tauri-apps/api/event";
import FolderPicker from "./FolderPicker.vue";

export default {
  name: "Setting",
  components: { FolderPicker },
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
        if (downloaded.length < 2 && !this.isDownloading) {
            this.downloadModels(false);
        }
    },
    getProgress(model) {
      const progress = this.downloadProgress[model];
      if (!progress || !progress.total) return 0;
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
