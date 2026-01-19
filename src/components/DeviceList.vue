<template>
  <v-container class="pa-6">
    <div class="d-flex align-center justify-space-between mb-8">
      <div>
        <h1 class="text-h4 font-weight-bold mb-1">Devices</h1>
        <div class="text-subtitle-1 text-medium-emphasis">Manage your connected devices</div>
      </div>
      <Connect />
    </div>

    <!-- Active Device Card (if needed, or just list all devices grid) -->
    
    <v-row>
      <v-col cols="12" sm="6" md="4" v-for="device in devices" :key="device.title">
        <v-card class="glass-panel" height="100%">
          <v-card-item>
            <template v-slot:prepend>
              <v-avatar :color="device.color" variant="tonal" rounded="lg">
                <v-icon>{{ device.icon }}</v-icon>
              </v-avatar>
            </template>
            <v-card-title>{{ device.title }}</v-card-title>
            <v-card-subtitle>{{ device.subtitle || 'Connected' }}</v-card-subtitle>
            
            <template v-slot:append>
               <v-icon
                v-if="device.up_to_date && !device.syncing"
                color="success"
                icon="mdi-check-circle"
              ></v-icon>
              <v-icon
                v-if="!device.up_to_date && !device.syncing"
                color="warning"
                icon="mdi-alert-circle"
              ></v-icon>
              <v-icon
                v-if="device.syncing"
                color="primary"
                icon="mdi-loading"
                class="mdi-spin"
              ></v-icon>
            </template>
          </v-card-item>
          
          <v-card-text>
               <div class="d-flex align-center mt-2">
                   <div class="text-caption text-medium-emphasis">Status</div>
                   <v-spacer></v-spacer>
                   <v-chip size="x-small" :color="device.syncing ? 'primary' : device.up_to_date ? 'success' : 'warning'" variant="flat">
                       {{ device.syncing ? 'Syncing...' : device.up_to_date ? 'Synced' : 'Attention' }}
                   </v-chip>
               </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

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
