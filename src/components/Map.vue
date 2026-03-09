<template>
  <div style="height: 100vh; width: 100%; position: relative;">
    <!-- Empty State Overlay -->
    <div v-if="!loading && photos.length === 0" class="map-empty-state">
      <div class="d-flex flex-column align-center justify-center h-100 px-6 text-center animate-fade-in">
        <v-icon size="48" color="#3f3f46" class="mb-4">mdi-map-marker-off-outline</v-icon>
        <div class="text-h6 text-zinc-secondary font-weight-bold">No location data found</div>
        <p class="text-body-2 text-zinc-muted mt-1 max-w-400">Photos with EXIF GPS coordinates will automatically appear on this map after indexing.</p>
      </div>
    </div>

    <!-- Loading Overlay -->
    <div v-if="loading" class="map-empty-state">
      <div class="d-flex flex-column align-center justify-center h-100">
        <v-progress-circular indeterminate color="white" size="32" width="3"></v-progress-circular>
      </div>
    </div>

    <l-map
      ref="map"
      v-model:zoom="zoom"
      :center="initialCenter"
      @ready="onMapReady"
      :minZoom="2"
      :options="{ zoomControl: false, attributionControl: false, preferCanvas: true }"
      style="height: 100%; width: 100%; background: #e2e2e7;"
      class="light-map"
    >
      <l-tile-layer
        url="https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}{r}.png"
        layer-type="base"
        name="CartoDB Basemap"
        :options="{
          updateWhenZooming: false,
          updateWhenIdle: true,
          keepBuffer: 2,
          crossOrigin: true
        }"
      />
    </l-map>

    <PhotoViewer
      v-model="viewerOpen"
      :photos="viewerPhotos"
      v-model:index="currentPhotoIndex"
    />
  </div>
</template>

<script>
import "leaflet/dist/leaflet.css";
import { LMap, LTileLayer } from "@vue-leaflet/vue-leaflet";
import L from "leaflet";
// Leaflet plugins like leaflet.heat often expect window.L to be defined in module environments
if (typeof window !== 'undefined') {
  window.L = L;
}
import "leaflet.heat";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from '@tauri-apps/api/core';
import * as path from "@tauri-apps/api/path";
import PhotoViewer from "./PhotoViewer.vue";

export default {
  components: {
    LMap,
    LTileLayer,
    PhotoViewer
  },
  data() {
    return {
      zoom: 2,
      initialCenter: [20, 0], // Better default than [0,0]
      map: null,
      photos: [],
      loading: true,
      heatLayer: null,
      viewerOpen: false,
      viewerPhotos: [],
      currentPhotoIndex: 0,
    };
  },
  methods: {
    async onMapReady(map) {
      console.log("Leaflet Map ready event received");
      this.map = map;
      // Wait for next tick and a small delay to ensure container has dimensions
      this.$nextTick(async () => {
        setTimeout(async () => {
          if (this.map) {
            console.log("Map instance confirmed, invalidating size...");
            this.map.invalidateSize();
            await this.loadHeatmapData();
          }
        }, 100);
      });
    },
    async loadHeatmapData() {
        if (!this.map) {
            console.log("Map not ready yet");
            return;
        }

        try {
            console.log("Fetching heatmap data...");
            const photosJson = await invoke("get_heatmap_data");
            this.photos = JSON.parse(photosJson);
            console.log(`Loaded ${this.photos.length} photos with location data`);

            if (this.photos.length === 0) {
                console.log("No photos with location data found in database");
                this.loading = false;
                return;
            }

            const heatData = this.photos.map(p => [p.latitude, p.longitude, 5]);

            if (this.heatLayer) {
                this.map.removeLayer(this.heatLayer);
            }

            // Retry loop for getSize() on Android: it can be (0,0) early on
            let size = this.map.getSize();
            console.log(`Initial map size: ${size.x}x${size.y}`);
            let retries = 0;
            while ((size.x === 0 || size.y === 0) && retries < 15) {
                console.log(`Map size is 0, retrying... (${retries+1}/15)`);
                await new Promise(r => setTimeout(r, 300));
                this.map.invalidateSize();
                size = this.map.getSize();
                retries++;
            }

            console.log(`Final map size for heatmap: ${size.x}x${size.y}`);

            if (size.x > 0 && size.y > 0 && typeof L.heatLayer === 'function') {
                console.log("Adding heat layer to map...");
                this.heatLayer = L.heatLayer(heatData, {
                    radius: 25,
                    blur: 15,
                    maxZoom: 10,
                }).addTo(this.map);
                console.log("Heat layer added successfully");
            } else if (typeof L.heatLayer !== 'function') {
                console.error("CRITICAL: Leaflet Heat layer plugin NOT found (L.heatLayer is undefined)");
            } else {
                console.error("FAILED: Map size remains 0x0 after retries. Leaflet cannot render.");
            }

            // Find location with most photos or fit all
            if (this.photos.length > 0) {
                const points = this.photos.map(p => [p.latitude, p.longitude]);
                if (points.length > 1) {
                    // Show all points with some padding
                    this.map.fitBounds(points, { padding: [50, 50], maxZoom: 10 });
                } else {
                    this.map.setView(points[0], 4);
                }
            }

            // Add click listener to map to find photos near click logic
            this.map.on('click', this.handleMapClick);

        } catch (e) {
            console.error("Failed to load map data", e);
        } finally {
            this.loading = false;
        }
    },
    handleMapClick(e) {
        // Simple radius search for demo: find photos within ~50km (approx 0.5 degrees)
        const clickLat = e.latlng.lat;
        const clickLng = e.latlng.lng;
        const radius = 0.5;

        const nearbyPhotos = this.photos.filter(p => {
            const dLat = Math.abs(p.latitude - clickLat);
            const dLng = Math.abs(p.longitude - clickLng);
            return dLat < radius && dLng < radius;
        });

        if (nearbyPhotos.length > 0) {
            this.viewerPhotos = nearbyPhotos;
            this.currentPhotoIndex = 0;
            this.viewerOpen = true;
        }
    },
  }
};
</script>

<style scoped>
/* Ensure map takes full height of container */
:deep(.leaflet-container) {
    height: 100%;
    background: #f4f4f5;
}

.map-empty-state {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(4px);
  z-index: 1001;
  pointer-events: none;
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
