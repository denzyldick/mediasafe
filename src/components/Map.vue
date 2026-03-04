<template>
  <div style="height: 100%; width: 100%; position: relative">
    <!-- Empty State Overlay -->
    <div v-if="photos.length === 0" class="map-empty-state">
      <div class="d-flex flex-column align-center justify-center h-100 px-6 text-center animate-fade-in">
        <v-icon size="48" color="#3f3f46" class="mb-4">mdi-map-marker-off-outline</v-icon>
        <div class="text-h6 text-zinc-secondary font-weight-bold">No location data found</div>
        <p class="text-body-2 text-zinc-muted mt-1 max-w-400">Photos with EXIF GPS coordinates will automatically appear on this map after indexing.</p>
      </div>
    </div>
    
    <l-map 
      ref="map" 
      v-model:zoom="zoom" 
      :center="initialCenter" 
      @ready="onMapReady" 
      :minZoom="2"
      :options="{ zoomControl: false, attributionControl: false }"
      class="dark-map"
    >
      <l-tile-layer
        url="https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
        layer-type="base"
        name="CartoDB DarkMatter"
      ></l-tile-layer>

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
      heatLayer: null,
      viewerOpen: false,
      viewerPhotos: [],
      currentPhotoIndex: 0,
    };
  },
  methods: {
    async onMapReady(map) {
      this.map = map;
      await this.loadHeatmapData();
    },
    async loadHeatmapData() {
        const dataDir = await path.homeDir();
        try {
            const photosJson = await invoke("get_heatmap_data", { path: dataDir });
            this.photos = JSON.parse(photosJson);
            
            const heatData = this.photos.map(p => [p.latitude, p.longitude, 5]); // Intensity 5
            
            if (this.heatLayer) {
                this.map.removeLayer(this.heatLayer);
            }
            
            this.heatLayer = L.heatLayer(heatData, {
                radius: 25,
                blur: 15,
                maxZoom: 10,
            }).addTo(this.map);

            // Find location with most photos
            if (this.photos.length > 0) {
                const buckets = {};
                let maxCount = 0;
                let bestKey = null;

                this.photos.forEach(p => {
                    // Bucket by 1-degree grid
                    const key = `${Math.floor(p.latitude)},${Math.floor(p.longitude)}`;
                    if (!buckets[key]) buckets[key] = { count: 0, latSum: 0, lngSum: 0 };
                    buckets[key].count++;
                    buckets[key].latSum += p.latitude;
                    buckets[key].lngSum += p.longitude;
                    
                    if (buckets[key].count > maxCount) {
                        maxCount = buckets[key].count;
                        bestKey = key;
                    }
                });
                
                if (bestKey) {
                    const bucket = buckets[bestKey];
                    const center = [bucket.latSum / bucket.count, bucket.lngSum / bucket.count];
                    this.map.setView(center, 12);
                }
            }

            // Add click listener to map to find photos near click logic
            this.map.on('click', this.handleMapClick);
            
        } catch (e) {
            console.error("Failed to load map data", e);
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
    background: #09090b;
}

.dark-map :deep(.leaflet-tile-pane) {
    filter: brightness(0.6) invert(100%) contrast(90%) hue-rotate(180deg) brightness(0.9);
}

.map-empty-state {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(9, 9, 11, 0.7);
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
