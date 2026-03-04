<template>
  <div 
    class="image-container" 
    :class="{ 'is-selected': selected, 'in-selection-mode': selectionMode }"
    @click="handleClick"
  >
    <img :src="imageSrc" loading="lazy" alt="Photo" />
    
    <!-- Selection Checkbox -->
    <div v-if="selectionMode" class="selection-overlay">
      <v-icon :color="selected ? 'white' : 'white'" size="24">
        {{ selected ? 'mdi-checkbox-marked-circle' : 'mdi-checkbox-blank-circle-outline' }}
      </v-icon>
    </div>

    <!-- Favorite Button (Hidden in selection mode) -->
    <v-btn
      v-if="!selectionMode"
      icon
      variant="text"
      size="small"
      class="favorite-btn"
      :class="{ 'is-fav': isFavorite }"
      :color="isFavorite ? 'red' : 'white'"
      @click.stop="toggleFavorite"
    >
      <v-icon>{{ isFavorite ? 'mdi-heart' : 'mdi-heart-outline' }}</v-icon>
    </v-btn>

    <div class="tags-container" v-if="tags.length > 0 && !selectionMode">
      <v-chip
        v-for="tag in tags"
        :key="tag"
        size="x-small"
        color="primary"
        variant="flat"
        class="ma-1"
      >
        {{ tag }}
      </v-chip>
    </div>
  </div>
</template>

<script>
import { convertFileSrc } from '@tauri-apps/api/core';

export default {
  name: "Image",
  props: {
    path: Object,
    selected: Boolean,
    selectionMode: Boolean
  },
  emits: ['toggle-favorite', 'click', 'select'],
  computed: {
    imageSrc() {
      if (!this.path || !this.path.encoded) return '';
      
      const src = this.path.encoded;
      if (src.startsWith('data:image')) return src;

      const converted = convertFileSrc(src);
      if (converted === src && src.startsWith('/')) {
            return `http://asset.localhost${encodeURIComponent(src)}`;
      }
      return converted;
    },
    isFavorite() {
        return this.path.favorite === true;
    },
    tags() {
      if (!this.path || !this.path.objects) return [];
      return Object.entries(this.path.objects)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 3)
        .map(entry => entry[0]);
    }
  },
  methods: {
      toggleFavorite() {
          this.$emit('toggle-favorite', this.path.id);
      },
      handleClick() {
        if (this.selectionMode) {
          this.$emit('select', this.path.id);
        } else {
          this.$emit('click');
        }
      }
  }
};
</script>

<style scoped>
.image-container {
  width: 100%;
  aspect-ratio: 1;
  overflow: hidden;
  background-color: #09090b;
  cursor: pointer;
  position: relative;
  transition: all 0.2s ease;
  border: 2px solid transparent;
  will-change: transform, border-color;
}

.in-selection-mode {
  transform: scale(0.95);
}

.is-selected {
  border-color: #ffffff !important;
  transform: scale(0.95);
}

.selection-overlay {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 3;
  text-shadow: 0 0 4px rgba(0,0,0,0.5);
}

img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center;
  transition: transform 0.3s ease;
  will-change: transform;
}

.image-container:not(.in-selection-mode):hover img {
  transform: scale(1.05);
}

.favorite-btn {
    position: absolute;
    top: 5px;
    right: 5px;
    z-index: 2;
    background-color: rgba(0,0,0,0.2);
    backdrop-filter: blur(4px);
    opacity: 0;
    transition: opacity 0.2s ease;
}

.image-container:hover .favorite-btn,
.favorite-btn.is-fav {
    opacity: 1;
}

.tags-container {
  position: absolute;
  bottom: 4px;
  left: 4px;
  display: flex;
  flex-wrap: wrap;
  z-index: 2;
  gap: 2px;
}
</style>
