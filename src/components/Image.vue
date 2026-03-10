<template>
  <div
    class="image-item-container"
    :class="{ 'is-selected': selected, 'selection-active': selectionMode }"
    @click="handleClick"
  >
    <div class="image-wrapper shadow-sm">
      <img :src="imageSrc" loading="lazy" alt="Photo" class="photo-img" v-if="imageSrc" />
      <div class="scrim-overlay"></div>

      <!-- Video Indicator -->
      <div v-if="isVideo" class="video-indicator">
        <v-icon color="white" size="20">mdi-play</v-icon>
      </div>

      <!-- Selection Mode UI -->
      <div v-if="selectionMode" class="selection-indicator">
        <div class="check-circle" :class="{ 'checked': selected }">
          <v-icon v-if="selected" color="white" size="16">mdi-check</v-icon>
        </div>
      </div>

      <!-- Favorite Button -->
      <button
        v-if="!selectionMode"
        class="action-btn favorite-action"
        :class="{ 'is-fav': isFavorite }"
        @click.stop="toggleFavorite"
      >
        <v-icon size="18" :color="isFavorite ? '#ef4444' : 'white'">
          {{ isFavorite ? 'mdi-heart' : 'mdi-heart-outline' }}
        </v-icon>
      </button>

      <!-- AI Tags -->
      <div class="ai-tags-preview" v-if="tags.length > 0 && !selectionMode">
        <span v-for="tag in tags" :key="tag" class="tag-pill">{{ tag }}</span>
      </div>
    </div>
  </div>
</template>

<script>
import { convertFileSrc, invoke } from '@tauri-apps/api/core';

export default {
  name: "Image",
  props: {
    path: Object,
    selected: Boolean,
    selectionMode: Boolean
  },
  emits: ['toggle-favorite', 'click', 'select'],
  data: () => ({
    localThumb: null,
    mediaPort: null,
  }),
  async mounted() {
    try {
      this.mediaPort = await invoke("get_media_server_port");
      
      // If video and missing thumbnail, request from global worker
      if (this.isVideo && !this.path.encoded) {
        window.dispatchEvent(new CustomEvent('request-thumbnail', { 
          detail: { id: this.path.id, location: this.path.location, videoUrl: this.videoUrl } 
        }));
        
        window.addEventListener(`thumbnail-ready-${this.path.id}`, (e) => {
          this.localThumb = e.detail.b64;
        }, { once: true });
      }
    } catch (e) {}
  },
  computed: {
    videoUrl() {
      if (!this.path || !this.isVideo || !this.mediaPort) return '';
      let path = this.path.location.replace(/\\/g, '/');
      if (path.match(/^[a-zA-Z]:\//)) {
          path = path.substring(3);
      } else if (path.startsWith('/')) {
          path = path.substring(1);
      }
      const encoded = path.split('/').map(encodeURIComponent).join('/');
      return `http://127.0.0.1:${this.mediaPort}/media/${encoded}`;
    },
    imageSrc() {
      if (!this.path) return '';

      // 1. ALWAYS prioritize the generated base64 thumbnail (encoded)
      if (this.path.encoded && this.path.encoded.length > 100) {
          return this.path.encoded;
      }

      // 2. Fallback to local generated thumb (from video worker)
      if (this.localThumb) return this.localThumb;

      // 3. DO NOT load the full high-res file here as it slows down the UI
      // The high-res file will only be loaded in the PhotoViewer
      return null;
    },
    isFavorite() {
        return this.path.favorite === true;
    },
    isVideo() {
      if (!this.path || !this.path.location) return false;
      const ext = this.path.location.split('.').pop().toLowerCase();
      return ["mp4", "mkv", "mov", "avi", "webm"].includes(ext);
    },
    tags() {
      if (!this.path || !this.path.objects) return [];
      return Object.entries(this.path.objects)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 2)
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
.image-item-container {
  width: 100%;
  position: relative;
  cursor: pointer;
  transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  will-change: transform;
}

.image-wrapper {
  width: 100%;
  aspect-ratio: 1;
  overflow: hidden;
  border-radius: 16px;
  position: relative;
  background-color: #f4f4f5;
  border: 1px solid rgba(0,0,0,0.05);
}

.photo-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.6s cubic-bezier(0.16, 1, 0.3, 1);
}

.video-placeholder {
  width: 100%;
  height: 100%;
  background-color: #f4f4f5;
}

.scrim-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(to bottom, rgba(0,0,0,0.2) 0%, transparent 30%, transparent 70%, rgba(0,0,0,0.3) 100%);
  opacity: 0;
  transition: opacity 0.3s ease;
  z-index: 1;
}

.image-item-container:hover .scrim-overlay {
  opacity: 1;
}

.image-item-container:hover .photo-img {
  transform: scale(1.08);
}

.image-item-container:active {
  transform: scale(0.96);
}

/* Selection State */
.selection-active .image-wrapper {
  transform: scale(0.92);
}

.is-selected .image-wrapper {
  border: 4px solid #000000;
  transform: scale(0.92);
}

.selection-indicator {
  position: absolute;
  top: 12px;
  left: 12px;
  z-index: 10;
}

.check-circle {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid white;
  background: rgba(0,0,0,0.2);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.check-circle.checked {
  background: #000000;
  border-color: #000000;
}

/* Video Indicator */
.video-indicator {
  position: absolute;
  bottom: 12px;
  right: 12px;
  width: 32px;
  height: 32px;
  background: rgba(0,0,0,0.5);
  backdrop-filter: blur(8px);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 5;
}

/* Favorite Button */
.action-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 32px;
  height: 32px;
  border-radius: 10px;
  background: rgba(255,255,255,0.2);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  cursor: pointer;
  opacity: 0;
  transform: translateY(-4px);
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  z-index: 5;
}

.image-item-container:hover .action-btn,
.action-btn.is-fav {
  opacity: 1;
  transform: translateY(0);
}

.action-btn.is-fav {
  background: white;
}

/* AI Tags */
.ai-tags-preview {
  position: absolute;
  bottom: 12px;
  left: 12px;
  display: flex;
  gap: 4px;
  z-index: 5;
  opacity: 0;
  transform: translateY(4px);
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.image-item-container:hover .ai-tags-preview {
  opacity: 1;
  transform: translateY(0);
}

.tag-pill {
  font-size: 10px;
  font-weight: 700;
  color: white;
  background: rgba(0,0,0,0.5);
  backdrop-filter: blur(4px);
  padding: 2px 8px;
  border-radius: 6px;
  text-transform: capitalize;
}

.shadow-sm {
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
}
</style>
