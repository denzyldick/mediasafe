<template>
  <div style="height: 100%; width: 100%; position: relative">
    <div style="position: absolute; top: 10px; right: 10px; z-index: 1000">
        <v-btn @click="generateDummy" color="secondary" size="small" class="mr-2">Generate Dummy Data</v-btn>
    </div>
    
    <l-map ref="map" v-model:zoom="zoom" :center="[0, 0]" @ready="onMapReady" :minZoom="2">
      <l-tile-layer
        url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        layer-type="base"
        name="OpenStreetMap"
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
                    this.map.flyTo(center, 12);
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
    async generateDummy() {
        const dataDir = await path.homeDir();
        await invoke("generate_dummy_data", { path: dataDir });
        await this.loadHeatmapData();
    }
  }
};
</script>

<style scoped>
/* Ensure map takes full height of container */
:deep(.leaflet-container) {
    height: 100%;
}
</style>
