<template>
  <v-dialog v-model="visible" fullscreen transition="dialog-bottom-transition">
    <v-card color="black" class="d-flex align-center justify-center fill-height">
      <v-btn icon="mdi-close" variant="text" color="white" style="position: absolute; top: 20px; right: 20px; z-index: 2000" @click="close"></v-btn>

      <v-btn icon="mdi-chevron-left" variant="text" color="white" size="x-large" @click="prev" style="position: absolute; left: 20px; z-index: 10"></v-btn>

      <div class="d-flex align-center justify-center" style="width: 100%; height: 100%"
           v-touch="{ left: () => next(), right: () => prev() }">
        <img v-if="currentPhoto" :src="currentPhotoSrc" style="max-width: 90%; max-height: 90%; object-fit: contain" />
      </div>

      <v-btn icon="mdi-chevron-right" variant="text" color="white" size="x-large" @click="next" style="position: absolute; right: 20px; z-index: 10"></v-btn>
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
      // Prefer encoded path if available, otherwise location
      // In current backend implementation, encoded is path string for real photos
      const src = this.currentPhoto.encoded || this.currentPhoto.location;
      
      if (src) {
          // If it looks like base64, return as is (future proofing)
          if (src.startsWith('data:image')) return src;
          
          // Try convertFileSrc
          const converted = convertFileSrc(src);

          // If convertFileSrc returned the raw path (starts with /), manually construct asset URL
          if (converted === src && src.startsWith('/')) {
                console.log("PhotoViewer: Manual asset URL construction for:", src);
                return `http://asset.localhost${encodeURI(src)}`;
          }
          
          return converted;
      }
      return '';
    },
  },
  methods: {
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
</style>
