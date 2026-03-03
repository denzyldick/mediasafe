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
  <v-container class="fill-height pb-16" fluid>
    <v-row justify="center" align="center">
      <v-col cols="12" md="10" lg="8">
        <!-- Hero Section -->
        <div class="text-center mb-16">
          <v-icon color="grey-lighten-2" size="64" class="mb-6 opacity-60">mdi-shield-lock-outline</v-icon>
          <h1 class="text-h3 font-weight-bold text-grey-lighten-4 mb-4 tracking-tight">
            MediaSafe
          </h1>
          <p class="text-body-1 text-grey-lighten-1 mx-auto" style="max-width: 500px; line-height: 1.6;">
            A simple, private way to back up and sync your memories across your own devices.
          </p>
        </div>

        <!-- Selection Cards -->
        <v-row justify="center" class="px-4">
          <!-- Option 1: New Device -->
          <v-col cols="12" md="5" class="mb-4">
            <v-card
              variant="outlined"
              class="cursor-pointer h-100 pa-6 transition-all welcome-card"
              @click="$emit('new_device')"
            >
              <div class="d-flex align-center mb-4">
                <v-icon size="28" color="grey-lighten-2" class="mr-3">mdi-plus-circle-outline</v-icon>
                <v-card-title class="text-h5 font-weight-bold pa-0 text-grey-lighten-3">
                  Start Fresh
                </v-card-title>
              </div>
              <div class="text-body-2 text-grey-lighten-1">
                Initialize a new library on this device.
              </div>
            </v-card>
          </v-col>

          <!-- Option 2: Join Group -->
          <v-col cols="12" md="5" class="mb-4">
            <v-card
              variant="outlined"
              class="cursor-pointer h-100 pa-6 transition-all welcome-card"
              @click="$emit('join_group')"
            >
              <div class="d-flex align-center mb-4">
                <v-icon size="28" color="grey-lighten-2" class="mr-3">mdi-link-variant</v-icon>
                <v-card-title class="text-h5 font-weight-bold pa-0 text-grey-lighten-3">
                  Connect
                </v-card-title>
              </div>
              <div class="text-body-2 text-grey-lighten-1">
                Sync with an existing MediaSafe network.
              </div>
            </v-card>
          </v-col>
        </v-row>
        
        <div class="text-center mt-16">
          <span class="text-caption text-grey-darken-1 font-weight-medium uppercase tracking-widest">v0.1.0</span>
        </div>

      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>
.tracking-tight {
  letter-spacing: -0.05em;
}

.welcome-card {
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  background: #09090b !important;
}

.welcome-card:hover {
  border-color: rgba(255, 255, 255, 0.3) !important;
  background: #18181b !important;
}

.transition-all {
  transition: all 0.2s ease;
}

.uppercase {
  text-transform: uppercase;
}

.tracking-widest {
  letter-spacing: 0.1em;
}
</style>
