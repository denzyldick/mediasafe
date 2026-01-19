<template>
  <div class="image-container">
    <img :src="imageSrc" loading="lazy" alt="Photo" />
    <v-btn
      icon
      variant="text"
      size="small"
      class="favorite-btn"
      :color="isFavorite ? 'red' : 'white'"
      @click.stop="toggleFavorite"
    >
      <v-icon>{{ isFavorite ? 'mdi-heart' : 'mdi-heart-outline' }}</v-icon>
    </v-btn>
  </div>
</template>

<script>
import { convertFileSrc } from '@tauri-apps/api/core';

export default {
  name: "Image",
  props: ["path"],
  emits: ['toggle-favorite'],
  computed: {
    imageSrc() {
      if (!this.path || !this.path.encoded) return '';
      
      const src = this.path.encoded;
      
      // If it's already a data URI, return as is
      if (src.startsWith('data:image')) return src;

      // Try convertFileSrc
      const converted = convertFileSrc(src);
      
      // If convertFileSrc returned the raw path (starts with /), manually construct asset URL
      if (converted === src && src.startsWith('/')) {
            console.log("Manual asset URL construction for:", src);
            return `http://asset.localhost${encodeURI(src)}`;
      }
      
      return converted;
    },
    isFavorite() {
        return this.path.favorite === true;
    }
  },
  methods: {
      toggleFavorite() {
          this.$emit('toggle-favorite', this.path.id);
      }
  }
};
</script>

<style>
.image-container {
  width: 100%;
  aspect-ratio: 1;
  overflow: hidden;
  border-radius: 12px;
  background-color: #f0f0f0;
  cursor: pointer;
  position: relative;
}

img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center;
  transition: transform 0.3s ease;
}

.image-container:hover img {
  transform: scale(1.05);
}

.favorite-btn {
    position: absolute;
    top: 5px;
    right: 5px;
    z-index: 2;
    background-color: rgba(0,0,0,0.3);
}
</style>
