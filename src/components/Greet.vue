<script>
import logo from "../assets/logo.png";

export default {
  name: "Greet",
  data() {
    return {
      active: true,
      logoUrl: logo
    };
  },
};
</script>

<template>
  <v-container class="fill-height pb-16" fluid style="background-color: #ffffff !important;">
    <v-row justify="center" align="center">
      <v-col cols="12" sm="10" md="8" lg="6" class="text-center">
        <div class="onboarding-card animate-fade-in">
          <v-avatar size="120" color="transparent" class="mb-8 elevation-0">
            <v-img :src="logoUrl"></v-img>
          </v-avatar>
          
          <h1 class="text-h2 font-weight-black mb-4 tracking-tighter" style="color: #000000 !important;">
            Siegu
          </h1>
          <p class="text-subtitle-1 text-zinc-secondary mb-10 max-w-500 mx-auto">
            Your private, AI-powered media sanctuary. Organize, search, and protect your memories entirely on your own terms.
          </p>

          <div class="d-flex flex-column ga-4">
            <v-card 
              variant="flat" 
              class="setup-button" 
              @click="$emit('setup-local')"
            >
              <div class="d-flex align-center py-4 px-6">
                <v-avatar color="#f4f4f5" size="48" class="mr-4 border-subtle">
                  <v-icon color="#18181b">mdi-folder-plus</v-icon>
                </v-avatar>
                <div class="text-left">
                  <div class="font-weight-bold text-zinc-primary">Setup Local Library</div>
                  <div class="text-caption text-zinc-secondary">Index photos from folders on this device.</div>
                </div>
              </div>
            </v-card>

            <v-card 
              variant="flat" 
              class="setup-button" 
              @click="$emit('setup-sync')"
            >
              <div class="d-flex align-center py-4 px-6">
                <v-avatar color="#f4f4f5" size="48" class="mr-4 border-subtle">
                  <v-icon color="#18181b">mdi-sync</v-icon>
                </v-avatar>
                <div class="text-left">
                  <div class="font-weight-bold text-zinc-primary">Join Library</div>
                  <div class="text-caption text-zinc-secondary">Sync with an existing Siegu network.</div>
                </div>
              </div>
            </v-card>
          </div>

          <div class="text-caption text-zinc-muted px-10 mt-10">
            By continuing, you agree that AI processing happens locally on your device.
          </div>
        </div>
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>
.onboarding-card {
  padding: 40px 20px;
}

.max-w-500 {
  max-width: 500px;
}

.setup-button {
  background: #ffffff !important;
  border: 1px solid rgba(0, 0, 0, 0.1) !important;
  border-radius: 16px !important;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.setup-button:hover {
  background: #f4f4f5 !important;
  border-color: rgba(0, 0, 0, 0.2) !important;
  transform: scale(1.01);
}

.setup-button:active {
  transform: scale(0.98);
}

.animate-fade-in {
  animation: fadeIn 0.8s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
