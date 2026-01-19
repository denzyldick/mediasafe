<script>
export default {
  name: "Greet",
  data() {
    return {
      active: true,
    };
  },
};
</script>

<template>
  <v-container class="fill-height bg-grey-lighten-4" fluid>
    <v-row justify="center" align="center">
      <v-col cols="12" md="10" lg="8">
        <!-- Hero Section -->
        <div class="text-center mb-12">
          <v-icon color="primary" size="80" class="mb-4">mdi-shield-lock-outline</v-icon>
          <h1 class="text-h2 font-weight-bold text-grey-darken-3 mb-4">
            Welcome to MediaSafe
          </h1>
          <p class="text-h6 text-grey-darken-1 font-weight-regular mx-auto" style="max-width: 600px">
            Your photos. Your devices. <span class="text-primary font-weight-bold">No third parties.</span><br>
            Securely back up your memories to your own hardware.
          </p>
        </div>

        <!-- Selection Cards -->
        <v-row justify="center" spacing="4">
          <!-- Option 1: New Device -->
          <v-col cols="12" md="5">
          <v-hover v-slot="{ isHovering, props }">
            <v-card
              v-bind="props"
              :elevation="isHovering ? 8 : 2"
              class="cursor-pointer h-100 pa-4 transition-swing"
              @click="$emit('new_device')"
              rounded="xl"
            >
              <v-card-item>
                <div class="d-flex justify-space-between align-center mb-4">
                   <v-avatar color="primary" size="64" variant="tonal">
                    <v-icon size="32">mdi-laptop-plus</v-icon>
                   </v-avatar>
                   <v-icon v-if="isHovering" color="primary">mdi-arrow-right</v-icon>
                </div>
                <v-card-title class="text-h5 font-weight-bold mb-2">
                  Start Fresh
                </v-card-title>
                <div class="text-body-1 text-grey-darken-1 mb-4">
                  Set up this computer as a new, independent MediaSafe library.
                </div>
              </v-card-item>
            </v-card>
            </v-hover>
          </v-col>

          <!-- Option 2: Join Group -->
          <v-col cols="12" md="5">
            <v-hover v-slot="{ isHovering, props }">
            <v-card
              v-bind="props"
              :elevation="isHovering ? 8 : 2"
              class="cursor-pointer h-100 pa-4 transition-swing"
              @click="$emit('join_group')"
              rounded="xl"
            >
              <v-card-item>
                 <div class="d-flex justify-space-between align-center mb-4">
                   <v-avatar color="secondary" size="64" variant="tonal">
                    <v-icon size="32">mdi-connection</v-icon>
                   </v-avatar>
                    <v-icon v-if="isHovering" color="secondary">mdi-arrow-right</v-icon>
                </div>
                <v-card-title class="text-h5 font-weight-bold mb-2">
                  Connect Devices
                </v-card-title>
                <div class="text-body-1 text-grey-darken-1 mb-4">
                  Join an existing MediaSafe network to sync photos with other devices.
                </div>
              </v-card-item>
            </v-card>
            </v-hover>
          </v-col>
        </v-row>
        
        <div class="text-center mt-12 text-caption text-grey">
          MediaSafe v0.1.0 &copy; 2025
        </div>

      </v-col>
    </v-row>
  </v-container>
</template>
