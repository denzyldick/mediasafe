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
          <v-icon color="#a1a1aa" size="64" class="mb-6 opacity-40">mdi-shield-lock-outline</v-icon>
          <h1 class="text-h3 font-weight-bold text-zinc-primary mb-4 tracking-tight">
            MediaSafe
          </h1>
          <p class="text-body-1 text-zinc-secondary mx-auto" style="max-width: 500px; line-height: 1.6;">
            A simple, private way to back up and sync your memories across your own devices.
          </p>
        </div>

        <!-- Selection Cards -->
        <v-row justify="center" class="px-4">
          <!-- Option 1: New Device -->
          <v-col cols="12" md="5" class="mb-4">
            <v-card
              variant="flat"
              class="cursor-pointer h-100 pa-6 transition-all welcome-card border-subtle"
              @click="$emit('new_device')"
            >
              <div class="d-flex align-center mb-4">
                <v-icon size="28" color="#e4e4e7" class="mr-3 opacity-70">mdi-plus-circle-outline</v-icon>
                <v-card-title class="text-h5 font-weight-bold pa-0 text-zinc-primary">
                  Start Fresh
                </v-card-title>
              </div>
              <div class="text-body-2 text-zinc-secondary">
                Initialize a new library on this device.
              </div>
            </v-card>
          </v-col>

          <!-- Option 2: Join Group -->
          <v-col cols="12" md="5" class="mb-4">
            <v-card
              variant="flat"
              class="cursor-pointer h-100 pa-6 transition-all welcome-card border-subtle"
              @click="$emit('join_group')"
            >
              <div class="d-flex align-center mb-4">
                <v-icon size="28" color="#e4e4e7" class="mr-3 opacity-70">mdi-link-variant</v-icon>
                <v-card-title class="text-h5 font-weight-bold pa-0 text-zinc-primary">
                  Connect
                </v-card-title>
              </div>
              <div class="text-body-2 text-zinc-secondary">
                Sync with an existing MediaSafe network.
              </div>
            </v-card>
          </v-col>
        </v-row>
        
        <div class="text-center mt-16">
          <span class="text-caption text-zinc-muted font-weight-medium uppercase tracking-widest">v0.1.0</span>
        </div>

      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>
.tracking-tight {
  letter-spacing: -0.05em;
}

.welcome-card:hover {
  border-color: rgba(255, 255, 255, 0.15) !important;
  background: #27272a !important; /* Zinc-800 on hover */
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
