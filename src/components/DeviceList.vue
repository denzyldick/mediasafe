<template>
  <v-container class="pa-6">
    <div class="d-flex align-center justify-space-between mb-8">
      <div>
        <h1 class="text-h4 font-weight-bold text-zinc-primary mb-1">Devices</h1>
        <div class="text-subtitle-1 text-zinc-secondary">Manage your connected devices</div>
      </div>
      <Connect />
    </div>

    <!-- Empty State -->
    <div v-if="devices.length === 0" class="d-flex flex-column align-center justify-center py-16 text-center animate-fade-in">
      <v-icon size="64" color="#3f3f46" class="mb-4">mdi-laptop-off</v-icon>
      <div class="text-h6 text-zinc-secondary font-weight-bold">No connected devices</div>
      <p class="text-body-2 text-zinc-muted mt-1 max-w-400 mx-auto">Link your mobile devices or other computers to sync and access your media library from anywhere.</p>
    </div>
    
    <v-row v-else>
      <v-col cols="12" sm="6" md="4" v-for="device in devices" :key="device.title">
        <v-card variant="outlined" height="100%" class="device-card">
          <v-card-item>
            <template v-slot:prepend>
              <v-avatar :color="device.color === 'blue' ? 'white' : 'grey-darken-3'" variant="flat" rounded="md" class="opacity-80">
                <v-icon :color="device.color === 'blue' ? 'black' : 'white'">{{ device.icon }}</v-icon>
              </v-avatar>
            </template>
            <v-card-title class="text-white text-subtitle-1 font-weight-bold">{{ device.title }}</v-card-title>
            <v-card-subtitle class="text-grey text-caption">{{ device.subtitle || 'Connected' }}</v-card-subtitle>
            
            <template v-slot:append>
               <v-icon
                v-if="device.up_to_date && !device.syncing"
                color="white"
                size="small"
                class="opacity-50"
                icon="mdi-check-circle-outline"
              ></v-icon>
              <v-icon
                v-if="!device.up_to_date && !device.syncing"
                color="grey"
                size="small"
                icon="mdi-alert-circle-outline"
              ></v-icon>
              <v-icon
                v-if="device.syncing"
                color="white"
                size="small"
                icon="mdi-loading"
                class="mdi-spin"
              ></v-icon>
            </template>
          </v-card-item>
          
          <v-card-text>
               <div class="d-flex align-center mt-2">
                   <div class="text-caption text-grey opacity-70">Status</div>
                   <v-spacer></v-spacer>
                   <v-chip size="x-small" :color="device.syncing ? 'white' : 'grey-darken-3'" variant="flat" class="text-none">
                       {{ device.syncing ? 'Syncing' : device.up_to_date ? 'Up to date' : 'Update needed' }}
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
  border-color: rgba(255, 255, 255, 0.1) !important;
  background: #09090b !important;
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
import Connect from "./Connect.vue";

export default {
  name: "DeviceList",
  components: {
    Connect,
  },
  data: () => ({
      devices: [
        {
          color: "red",
          icon: "mdi-android",
          title: "Samsung Galaxy",
          subtitle: "Last sync: 1 week ago",
          up_to_date: false,
          syncing: false,
        },
        {
          color: "blue",
          icon: "mdi-apple",
          title: "iPhone 12",
          subtitle: "Just now",
          up_to_date: true,
          syncing: false,
        },
        {
          color: "yellow",
          icon: "mdi-microsoft",
          title: "Windows Desktop",
          subtitle: "Syncing...",
          up_to_date: false,
          syncing: true,
        },
      ],
  }),
  async mounted() {
      await this.list_devices();
  },
  methods: {
    async list_devices() {
      // Keep dummy data for now, append real devices if any
      const realDevicesStr = await invoke("list_devices");
      const realDevices = JSON.parse(realDevicesStr);
      
      if (realDevices && realDevices.length > 0) {
          // Map real devices to UI format if needed
          // For now, simply logging them or adding to array
          console.log("Real devices:", realDevices);
      }
    },
  },
};
</script>
