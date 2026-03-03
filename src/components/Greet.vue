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
  <template>
  <v-container class="fill-height welcome-container" fluid>
    <v-row justify="center" align="center">
      <v-col cols="12" md="10" lg="8">
        <!-- Hero Section -->
        <div class="text-center mb-12 animate-fade-in">
          <v-avatar color="primary" size="100" class="mb-6 elevation-10 shadow-glow">
            <v-icon size="50" color="white">mdi-shield-lock</v-icon>
          </v-avatar>
          <h1 class="text-h2 font-weight-black text-white mb-4 tracking-tight">
            Welcome to <span class="text-gradient">MediaSafe</span>
          </h1>
          <p class="text-h6 text-grey-lighten-1 font-weight-regular mx-auto px-4" style="max-width: 600px; line-height: 1.6;">
            Your photos. Your devices. <span class="text-primary font-weight-bold">No third parties.</span><br>
            <span class="text-body-1">Securely back up your memories to your own hardware with end-to-end privacy.</span>
          </p>
        </div>

        <!-- Selection Cards -->
        <v-row justify="center" class="px-4">
          <!-- Option 1: New Device -->
          <v-col cols="12" md="5" class="mb-4">
          <v-hover v-slot="{ isHovering, props }">
            <v-card
              v-bind="props"
              :elevation="isHovering ? 12 : 2"
              class="cursor-pointer h-100 transition-all card-hero overflow-hidden"
              :class="{ 'card-active-primary': isHovering }"
              @click="$emit('new_device')"
            >
              <div class="card-gradient-overlay"></div>
              <v-card-item class="position-relative z-1 pa-6">
                <div class="d-flex justify-space-between align-center mb-6">
                   <v-avatar color="primary" size="64" variant="elevated" class="elevation-4">
                    <v-icon size="32" color="white">mdi-laptop-plus</v-icon>
                   </v-avatar>
                   <v-icon v-if="isHovering" color="primary" size="32" class="animate-bounce-x">mdi-arrow-right</v-icon>
                </div>
                <v-card-title class="text-h4 font-weight-bold mb-3 text-white">
                  Start Fresh
                </v-card-title>
                <div class="text-body-1 text-grey-lighten-1 mb-2">
                  Initialize a new library on this device and start securing your media today.
                </div>
              </v-card-item>
            </v-card>
            </v-hover>
          </v-col>

          <!-- Option 2: Join Group -->
          <v-col cols="12" md="5" class="mb-4">
            <v-hover v-slot="{ isHovering, props }">
            <v-card
              v-bind="props"
              :elevation="isHovering ? 12 : 2"
              class="cursor-pointer h-100 transition-all card-hero overflow-hidden"
              :class="{ 'card-active-secondary': isHovering }"
              @click="$emit('join_group')"
            >
              <div class="card-gradient-overlay"></div>
              <v-card-item class="position-relative z-1 pa-6">
                 <div class="d-flex justify-space-between align-center mb-6">
                   <v-avatar color="secondary" size="64" variant="elevated" class="elevation-4">
                    <v-icon size="32" color="white">mdi-cellphone-link</v-icon>
                   </v-avatar>
                    <v-icon v-if="isHovering" color="secondary" size="32" class="animate-bounce-x">mdi-arrow-right</v-icon>
                </div>
                <v-card-title class="text-h4 font-weight-bold mb-3 text-white">
                  Connect
                </v-card-title>
                <div class="text-body-1 text-grey-lighten-1 mb-2">
                  Sync this device with your existing MediaSafe network using a secure pair code.
                </div>
              </v-card-item>
            </v-card>
            </v-hover>
          </v-col>
        </v-row>
        
        <div class="text-center mt-12 text-caption text-grey-darken-1">
          <v-chip size="x-small" variant="outlined" color="grey" class="mb-2">MediaSafe v0.1.0</v-chip>
          <div class="opacity-50">&copy; 2026 MediaSafe Project</div>
        </div>

      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>
.welcome-container {
  background: transparent !important;
}

.text-gradient {
  background: linear-gradient(45deg, #8b5cf6, #14b8a6);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.card-hero {
  background: rgba(24, 24, 27, 0.6) !important;
  border: 1px solid rgba(255, 255, 255, 0.05) !important;
  backdrop-filter: blur(20px);
}

.card-gradient-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: radial-gradient(circle at top right, rgba(255, 255, 255, 0.03), transparent 70%);
  pointer-events: none;
}

.card-active-primary {
  border-color: rgba(139, 92, 246, 0.4) !important;
  background: rgba(139, 92, 246, 0.05) !important;
  transform: translateY(-4px);
}

.card-active-secondary {
  border-color: rgba(20, 184, 166, 0.4) !important;
  background: rgba(20, 184, 166, 0.05) !important;
  transform: translateY(-4px);
}

.shadow-glow {
  box-shadow: 0 0 30px rgba(139, 92, 246, 0.3) !important;
}

.z-1 {
  z-index: 1;
}

.tracking-tight {
  letter-spacing: -0.025em;
}

.animate-fade-in {
  animation: fadeIn 0.8s ease-out;
}

.animate-bounce-x {
  animation: bounceX 1s infinite;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes bounceX {
  0%, 100% { transform: translateX(0); }
  50% { transform: translateX(5px); }
}

.transition-all {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
</template>
