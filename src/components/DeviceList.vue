<template>
  <v-container class="pa-6" style="background-color: #fafafa !important;">
    <div class="d-flex align-center justify-space-between mb-8">
      <div>
        <div class="d-flex align-center mb-1">
          <v-icon color="#18181b" size="28" class="mr-3">mdi-devices</v-icon>
          <h1 class="text-h4 font-weight-bold text-zinc-primary">Devices</h1>
        </div>
        <div class="text-subtitle-1 text-zinc-secondary">Manage your connected devices</div>
      </div>
      <Connect />
    </div>

    <!-- Empty State -->
    <div v-if="devices.length === 0" class="d-flex flex-column align-center justify-center py-16 text-center animate-fade-in">
      <v-icon size="64" color="#d4d4d8" class="mb-4">mdi-laptop-off</v-icon>
      <div class="text-h6 text-zinc-secondary font-weight-bold">No connected devices</div>
      <p class="text-body-2 text-zinc-muted mt-1 max-w-400 mx-auto">Link your mobile devices or other computers to sync and access your media library from anywhere.</p>
    </div>
    
    <v-row v-else>
      <v-col cols="12" sm="6" md="4" v-for="device in devices" :key="device.title">
        <v-card variant="flat" height="100%" class="device-card border-subtle ga-2" rounded="xl" color="#ffffff">
          <v-card-item class="py-4">
            <template v-slot:prepend>
              <v-avatar color="#f4f4f5" variant="flat" rounded="lg" class="border-subtle">
                <v-icon color="#18181b">{{ device.icon }}</v-icon>
              </v-avatar>
            </template>
            <v-card-title class="text-zinc-primary text-subtitle-1 font-weight-bold">{{ device.title }}</v-card-title>
            <v-card-subtitle class="text-zinc-secondary text-caption">{{ device.subtitle || 'Connected' }}</v-card-subtitle>
            
            <template v-slot:append>
               <v-icon
                v-if="device.up_to_date && !device.syncing"
                color="#18181b"
                size="small"
                class="opacity-50"
                icon="mdi-check-circle-outline"
              ></v-icon>
              <v-icon
                v-if="!device.up_to_date && !device.syncing"
                color="#71717a"
                size="small"
                icon="mdi-alert-circle-outline"
              ></v-icon>
              <v-icon
                v-if="device.syncing"
                color="#18181b"
                size="small"
                icon="mdi-loading"
                class="mdi-spin"
              ></v-icon>
            </template>
          </v-card-item>
          
          <v-card-text class="pt-0">
               <div v-if="device.syncing" class="mt-2">
                   <div class="d-flex align-center justify-space-between mb-1">
                       <span class="text-caption text-zinc-muted">Synchronizing...</span>
                       <span class="text-caption text-zinc-muted font-weight-bold">{{ device.speed }}</span>
                   </div>
                   <v-progress-linear
                     :model-value="device.progress"
                     color="#18181b"
                     height="4"
                     rounded
                     bg-color="#f4f4f5"
                     bg-opacity="1"
                   ></v-progress-linear>
               </div>
               <div v-else class="d-flex align-center mt-2">
                   <div class="text-caption text-zinc-muted">Status</div>
                   <v-spacer></v-spacer>
                   <v-chip size="x-small" color="#f4f4f5" variant="flat" class="text-none border-subtle text-zinc-secondary">
                       {{ device.up_to_date ? 'Up to date' : 'Idle' }}
                   </v-chip>
               </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>
.device-card {
  transition: all 0.2s ease;
  border: 1px solid rgba(0, 0, 0, 0.05) !important;
}

.device-card:hover {
  background: #ffffff !important;
  transform: translateY(-2px);
  border-color: rgba(0, 0, 0, 0.1) !important;
  box-shadow: 0 4px 12px rgba(0,0,0,0.05) !important;
}

.animate-fade-in {
  animation: fadeIn 0.4s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.max-w-400 {
  max-width: 400px;
}
</style>

<script>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Connect from "./Connect.vue";

export default {
  name: "DeviceList",
  components: {
    Connect,
  },
  data: () => ({
      devices: [],
      syncStates: {},
  }),
  async mounted() {
      await this.list_devices();
      
      // Listen for real-time sync updates
      listen("sync-progress", (event) => {
          const payload = event.payload;
          this.syncStates[payload.device_id] = payload;
          
          // Update device in list if it exists
          const device = this.devices.find(d => d.id === payload.device_id);
          if (device) {
              device.syncing = payload.status === 'syncing';
              device.progress = payload.progress;
              device.speed = this.formatSpeed(payload.bytes_per_second);
          }
      });
  },
  methods: {
    formatSpeed(bytes) {
        if (!bytes) return '0 B/s';
        const k = 1024;
        const sizes = ['B/s', 'KB/s', 'MB/s'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    },
    async list_devices() {
      try {
        const realDevicesStr = await invoke("list_devices");
        const realDevices = JSON.parse(realDevicesStr);
        
        this.devices = (realDevices || []).map(d => ({
            ...d,
            syncing: false,
            progress: 0,
            speed: '0 B/s'
        }));
      } catch (err) {
        console.error("Failed to list devices:", err);
      }
    },
  },
};
</script>
