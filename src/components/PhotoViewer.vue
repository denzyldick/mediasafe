<template>
  <v-dialog v-model="visible" fullscreen transition="dialog-bottom-transition">
    <v-card rounded="0" color="black" class="fill-height" style="overflow: hidden;">
      <!-- Main Viewer Area -->
      <div class="position-relative d-flex align-center justify-center h-100"
           v-touch="{ left: () => next(), right: () => prev() }"
           :style="{ marginRight: (showInfo && !$vuetify.display.mobile) ? '350px' : '0' }">
        
        <v-btn icon="mdi-close" variant="text" color="white" style="position: absolute; top: 20px; right: 20px; z-index: 2000" @click="close"></v-btn>

        <v-btn icon="mdi-chevron-left" variant="text" color="white" size="x-large" @click="prev" style="position: absolute; left: 20px; z-index: 10"></v-btn>

        <img v-if="currentPhoto" :src="currentPhotoSrc" class="viewer-image" />

        <v-btn icon="mdi-chevron-right" variant="text" color="white" size="x-large" @click="next" style="position: absolute; right: 20px; z-index: 10"></v-btn>
        
        <!-- Toggle Info Panel Button -->
        <v-btn
          icon="mdi-information-outline"
          variant="text"
          :color="showInfo ? '#e4e4e7' : 'white'"
          style="position: absolute; bottom: 20px; right: 20px; z-index: 2000"
          @click="showInfo = !showInfo"
        ></v-btn>
      </div>

      <!-- Info Drawer -->
      <v-navigation-drawer
        v-model="showInfo"
        location="right"
        width="350"
        temporary
        color="#09090b"
        class="border-s border-subtle"
        style="border-left: 1px solid rgba(255,255,255,0.1) !important;"
      >
        <v-toolbar color="transparent" density="compact">
          <v-toolbar-title class="text-zinc-primary text-subtitle-1 font-weight-bold">Metadata</v-toolbar-title>
          <v-spacer></v-spacer>
          <v-btn icon="mdi-close" variant="text" size="small" color="#a1a1aa" @click="showInfo = false"></v-btn>
        </v-toolbar>

        <v-divider class="opacity-10"></v-divider>

        <v-list class="bg-transparent px-4">
          <!-- File Info -->
          <div class="mb-6 pt-4">
            <div class="text-caption text-zinc-muted mb-1 text-uppercase tracking-widest">File Details</div>
            <div class="d-flex align-start mb-2">
              <v-icon size="small" color="#a1a1aa" class="mr-2 mt-1">mdi-file-document-outline</v-icon>
              <div class="text-body-2 text-zinc-secondary" style="word-break: break-all;">
                {{ currentPhoto.location }}
              </div>
            </div>
          </div>

          <v-divider class="opacity-10 mb-4"></v-divider>

          <!-- Camera / EXIF Info -->
          <div class="mb-6" v-if="hasExif">
            <div class="text-caption text-zinc-muted mb-3 text-uppercase tracking-widest">Camera Settings</div>
            
            <div class="d-flex align-center mb-4" v-if="exifData.make || exifData.model">
              <v-icon size="small" color="#a1a1aa" class="mr-2">mdi-camera</v-icon>
              <span class="text-body-2 text-zinc-secondary">{{ exifData.make }} {{ exifData.model }}</span>
            </div>

            <v-row dense>
              <v-col cols="6" v-if="exifData.date" class="mb-3">
                <div class="text-caption text-zinc-muted">Date Taken</div>
                <div class="text-body-2 text-zinc-secondary">{{ exifData.date }}</div>
              </v-col>
              <v-col cols="6" v-if="exifData.dimensions" class="mb-3">
                <div class="text-caption text-zinc-muted">Resolution</div>
                <div class="text-body-2 text-zinc-secondary">{{ exifData.dimensions }}</div>
              </v-col>
              <v-col cols="6" v-if="exifData.iso" class="mb-3">
                <div class="text-caption text-zinc-muted">ISO</div>
                <div class="text-body-2 text-zinc-secondary">{{ exifData.iso }}</div>
              </v-col>
              <v-col cols="6" v-if="exifData.shutter" class="mb-3">
                <div class="text-caption text-zinc-muted">Shutter</div>
                <div class="text-body-2 text-zinc-secondary">{{ exifData.shutter }}</div>
              </v-col>
              <v-col cols="6" v-if="exifData.aperture" class="mb-3">
                <div class="text-caption text-zinc-muted">Aperture</div>
                <div class="text-body-2 text-zinc-secondary">{{ exifData.aperture }}</div>
              </v-col>
            </v-row>
          </div>

          <v-divider class="opacity-10 mb-4" v-if="hasExif"></v-divider>

          <!-- AI Insights -->
          <div class="mb-6">
            <div class="text-caption text-zinc-muted mb-3 text-uppercase tracking-widest">AI Insights</div>
            
            <div v-if="aiTags.length === 0" class="text-body-2 text-zinc-muted font-italic">
              No AI insights generated yet.
            </div>
            
            <div v-for="tag in aiTags" :key="tag.name" class="mb-4">
              <div class="d-flex align-center justify-space-between w-100">
                <span class="text-body-2 text-zinc-secondary text-capitalize">{{ tag.name }}</span>
                <span class="text-caption text-zinc-muted">{{ tag.percent }}%</span>
              </div>
              <v-progress-linear
                :model-value="tag.percent"
                color="#e4e4e7"
                height="2"
                rounded
                class="mt-1 opacity-50"
              ></v-progress-linear>
            </div>
          </div>
        </v-list>
      </v-navigation-drawer>
    </v-card>
  </v-dialog>
</template>

<script>
import { convertFileSrc } from '@tauri-apps/api/core';

export default {
  name: "PhotoViewer",
  props: {
    modelValue: Boolean,
    photos: {
      type: Array,
      default: () => []
    },
    index: {
      type: Number,
      default: 0
    }
  },
  emits: ['update:modelValue', 'update:index'],
  data: () => ({
    showInfo: false,
    fullPhotoB64: '',
  }),
  computed: {
    visible: {
      get() { return this.modelValue; },
      set(val) { this.$emit('update:modelValue', val); }
    },
    currentPhoto() {
      if (!this.photos || this.photos.length === 0) return null;
      return this.photos[this.index];
    },
    currentPhotoSrc() {
      if (!this.currentPhoto) return '';
      // Use full base64 if fetched, otherwise fallback to thumbnail
      return this.fullPhotoB64 || this.currentPhoto.encoded;
    },
    exifData() {
      if (!this.currentPhoto || !this.currentPhoto.properties) return {};
      const props = this.currentPhoto.properties;
      
      // Attempt to format common EXIF fields
      let dimensions = null;
      if (props.PixelXDimension && props.PixelYDimension) {
        dimensions = `${props.PixelXDimension} x ${props.PixelYDimension}`;
      } else if (props.ImageWidth && props.ImageLength) {
        dimensions = `${props.ImageWidth} x ${props.ImageLength}`;
      }

      return {
        make: props.Make,
        model: props.Model,
        date: props.DateTimeOriginal || props.DateTime,
        dimensions,
        iso: props.PhotographicSensitivity || props.ISOSpeedRatings,
        shutter: props.ExposureTime,
        aperture: props.FNumber
      };
    },
    hasExif() {
      return Object.values(this.exifData).some(val => val !== undefined && val !== null);
    },
    aiTags() {
      if (!this.currentPhoto || !this.currentPhoto.objects) return [];
      
      return Object.entries(this.currentPhoto.objects)
        .map(([name, score]) => ({
          name,
          percent: Math.round(score * 100)
        }))
        .sort((a, b) => b.percent - a.percent);
    }
  },
  methods: {
    async fetchFullPhoto() {
      if (this.currentPhoto && this.currentPhoto.location) {
        try {
          this.fullPhotoB64 = await invoke("get_raw_photo", { path: this.currentPhoto.location });
        } catch (e) {
          console.error("Failed to fetch full photo", e);
          this.fullPhotoB64 = '';
        }
      } else {
        this.fullPhotoB64 = '';
      }
    },
    close() { this.visible = false; },
    next() {
        if (this.photos.length === 0) return;
        const newIndex = (this.index + 1) % this.photos.length;
        this.$emit('update:index', newIndex);
    },
    prev() {
        if (this.photos.length === 0) return;
        const newIndex = (this.index - 1 + this.photos.length) % this.photos.length;
        this.$emit('update:index', newIndex);
    },
    handleKeydown(e) {
        if (!this.visible) return;
        if (e.key === 'ArrowRight') this.next();
        if (e.key === 'ArrowLeft') this.prev();
        if (e.key === 'Escape') this.close();
        if (e.key === 'i') this.showInfo = !this.showInfo;
    }
  },
  watch: {
    index() {
      this.fetchFullPhoto();
    },
    visible(val) {
      if (val) {
        this.fetchFullPhoto();
      } else {
        this.fullPhotoB64 = '';
      }
    }
  },
  mounted() {
      window.addEventListener('keydown', this.handleKeydown);
  },
  beforeUnmount() {
      window.removeEventListener('keydown', this.handleKeydown);
  }
}
</script>

<style scoped>
.viewer-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  transition: opacity 0.2s ease-in-out;
  user-select: none;
  -webkit-user-drag: none;
}

.tracking-widest {
  letter-spacing: 0.1em;
}
.min-h-0 {
  min-height: 0 !important;
}
</style>
