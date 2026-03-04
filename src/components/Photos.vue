<template>
  <div class="photos-container">
    <!-- View Controls -->
    <div class="d-flex align-center justify-end mb-6" v-if="images.length > 0">
      <v-btn-toggle v-model="viewMode" mandatory variant="outlined" density="compact" class="border-subtle rounded-lg overflow-hidden">
        <v-btn value="grid" size="small" icon="mdi-grid" class="text-none"></v-btn>
        <v-btn value="monthly" size="small" icon="mdi-calendar-month" class="text-none"></v-btn>
      </v-btn-toggle>
    </div>

    <!-- Bulk Actions Toolbar -->
    <v-fade-transition>
      <div v-if="selectedIds.length > 0" class="bulk-toolbar-container">
        <v-sheet class="bulk-toolbar border-subtle d-flex align-center px-4 py-2 rounded-pill shadow-lg">
          <v-btn icon="mdi-close" variant="text" density="comfortable" color="#a1a1aa" @click="clearSelection"></v-btn>
          <span class="text-body-2 font-weight-bold text-zinc-primary ml-2">{{ selectedIds.length }} selected</span>
          <v-spacer></v-spacer>
          <v-btn
            prepend-icon="mdi-heart"
            variant="flat"
            color="white"
            size="small"
            class="text-none mr-2"
            @click="bulkFavorite"
          >
            Favorite
          </v-btn>
          <v-btn
            prepend-icon="mdi-delete-outline"
            variant="text"
            color="error"
            size="small"
            class="text-none"
            @click="bulkRemove"
          >
            Remove
          </v-btn>
        </v-sheet>
      </div>
    </v-fade-transition>

    <!-- Standard Grid View -->
    <div v-if="images.length > 0 && viewMode === 'grid'">
      <div class="grid">
        <Image
          v-for="(image, index) in images"
          v-bind:key="image.id"
          :path="image"
          :selected="selectedIds.includes(image.id)"
          :selection-mode="selectedIds.length > 0"
          @click="openViewer(index)"
          @select="toggleSelection"
          @toggle-favorite="handleToggleFavorite"
        />
      </div>
    </div>

    <!-- Monthly Grouped View -->
    <div v-if="images.length > 0 && viewMode === 'monthly'">
      <div v-for="(group, month) in groupedImages" :key="month" class="mb-10">
        <div class="text-h6 font-weight-bold text-zinc-primary mb-4 sticky-header px-2">
          {{ month }}
          <span class="text-caption text-zinc-muted ml-2">{{ group.length }} photos</span>
        </div>
        <div class="grid">
          <Image
            v-for="image in group"
            v-bind:key="image.id"
            :path="image"
            :selected="selectedIds.includes(image.id)"
            :selection-mode="selectedIds.length > 0"
            @click="openViewerByPhoto(image)"
            @select="toggleSelection"
            @toggle-favorite="handleToggleFavorite"
          />
        </div>
      </div>
    </div>

    <!-- Empty States -->
    <div v-else-if="!loading" class="d-flex flex-column align-center justify-center py-16 text-center animate-fade-in">
      <template v-if="searchQuery">
        <v-icon size="64" color="#3f3f46" class="mb-4">mdi-text-search-variant</v-icon>
        <div class="text-h6 text-zinc-secondary font-weight-bold">No results found</div>
        <p class="text-body-2 text-zinc-muted mt-1">We couldn't find any photos matching "{{ searchQuery }}"</p>
        <v-btn variant="text" color="white" class="mt-4 text-none" @click="$emit('clear-search')">Clear search</v-btn>
      </template>
      <template v-else-if="favoritesOnly">
        <v-icon size="64" color="#3f3f46" class="mb-4">mdi-heart-outline</v-icon>
        <div class="text-h6 text-zinc-secondary font-weight-bold">No favorites yet</div>
        <p class="text-body-2 text-zinc-muted mt-1">Tap the heart on any photo to add it to your favorites</p>
      </template>
      <template v-else>
        <v-icon size="64" color="#3f3f46" class="mb-4">mdi-image-plus-outline</v-icon>
        <div class="text-h6 text-zinc-secondary font-weight-bold">Your library is empty</div>
        <p class="text-body-2 text-zinc-muted mt-1">Add a folder in settings to start indexing your memories</p>
      </template>
    </div>

    <!-- Sentinel for infinite scroll -->
    <div id="scroll-sentinel" style="height: 20px;"></div>

    <!-- Loading State -->
    <v-row class="mt-4 pb-16">
      <v-col class="d-flex justify-center align-center">
        <v-progress-circular
          indeterminate
          color="#71717a"
          v-if="loading === true"
        ></v-progress-circular>
        <v-btn
          @click="list_files"
          variant="text"
          color="#a1a1aa"
          v-if="loading === false && !allLoaded"
          class="text-none"
        >
          Load more
        </v-btn>
      </v-col>
    </v-row>

    <PhotoViewer
      v-model="viewerOpen"
      :photos="images"
      v-model:index="currentPhotoIndex"
    />
  </div>
</template>
<script>
import { invoke } from "@tauri-apps/api/core";
import Image from "./Image.vue";
import PhotoViewer from "./PhotoViewer.vue";

export default {
  name: "Photos",
  components: { Image, PhotoViewer },
  data: () => ({
    loading: false,
    allLoaded: false,
    viewMode: 'monthly', // Default to monthly gallery view
    paging: {
      offset: 0,
      limit: 50,
    },
    images: [],
    selectedIds: [],
    viewerOpen: false,
    currentPhotoIndex: 0,
    favoritesOnly: false,
    observer: null,
  }),
  computed: {
    groupedImages() {
      const groups = {};
      this.images.forEach(image => {
        let date;
        if (image.created) {
          // EXIF format is usually YYYY:MM:DD HH:MM:SS
          const parts = image.created.split(' ');
          const dateParts = parts[0].split(':');
          if (dateParts.length === 3) {
            date = new Date(dateParts[0], dateParts[1] - 1, dateParts[2]);
          } else {
            date = new Date(image.created);
          }
        } else {
          date = new Date(); // Fallback
        }

        const month = date.toLocaleString('default', { month: 'long', year: 'numeric' });
        if (!groups[month]) {
          groups[month] = [];
        }
        groups[month].push(image);
      });
      return groups;
    }
  },
  props: {
    favorites: {
      type: Boolean,
      default: false,
    },
    searchQuery: {
      type: String,
      default: "",
    },
    isPersonFilter: {
      type: Boolean,
      default: false,
    },
    filters: {
      type: Object,
      default: () => ({
        favoritesOnly: false,
        dateRange: 'all',
        folder: null,
      })
    }
  },
  async created() {
    this.list_files();
  },
  mounted() {
    if (!this.isPersonFilter) {
      this.setupInfiniteScroll();
    }
  },
  beforeUnmount() {
    if (this.observer) {
      this.observer.disconnect();
    }
  },
  methods: {
    toggleSelection(id) {
      const index = this.selectedIds.indexOf(id);
      if (index === -1) {
        this.selectedIds.push(id);
      } else {
        this.selectedIds.splice(index, 1);
      }
    },
    clearSelection() {
      this.selectedIds = [];
    },
    async bulkFavorite() {
      const ids = [...this.selectedIds];
      for (const id of ids) {
        await this.handleToggleFavorite(id);
      }
      this.clearSelection();
    },
    async bulkRemove() {
      this.clearSelection();
    },
    setupInfiniteScroll() {
      this.observer = new IntersectionObserver((entries) => {
        if (entries[0].isIntersecting && !this.loading && !this.allLoaded) {
          this.list_files();
        }
      }, { 
        threshold: 0.01,
        rootMargin: '400px' // Load when sentinel is within 400px of viewport
      });
      
      const sentinel = document.getElementById('scroll-sentinel');
      if (sentinel) this.observer.observe(sentinel);
    },
    list_files: async function () {
      if (this.loading) return;
      
      this.loading = true;
      console.log("Listing files. Offset:", this.paging.offset, "Query:", this.searchQuery, "Filters:", this.filters);
      
      try {
        let response;
        if (this.isPersonFilter && this.searchQuery) {
          response = await invoke("get_person_photos", { personId: this.searchQuery });
          this.allLoaded = true;
        } else {
          response = await invoke("list_files", {
            offset: this.paging.offset,
            limit: this.paging.limit,
            query: this.searchQuery ?? "",
            scan: false,
            favoritesOnly: this.filters.favoritesOnly,
          });
        }
        
        const new_images = JSON.parse(response);
        
        if (this.paging.offset === 0) {
          this.images = new_images;
        } else {
          this.images.push(...new_images);
        }
        
        if (!this.isPersonFilter) {
          if (new_images.length < this.paging.limit) {
            this.allLoaded = true;
          } else {
            this.paging.offset += this.paging.limit;
          }
        }
      } catch (err) {
        console.error("Failed to list files:", err);
      } finally {
        this.loading = false;
      }
    },
    async handleToggleFavorite(id) {
      try {
        const isNowFavorite = await invoke("toggle_favorite", { id: id });
        const photo = this.images.find((p) => p.id === id);
        if (photo) {
          photo.favorite = isNowFavorite;
          if (this.filters.favoritesOnly && !isNowFavorite) {
            this.images = this.images.filter((p) => p.id !== id);
          }
        }
      } catch (err) {
        console.error("Failed to toggle favorite:", err);
      }
    },
    openViewer(index) {
      this.currentPhotoIndex = index;
      this.viewerOpen = true;
    },
    openViewerByPhoto(photo) {
      const index = this.images.findIndex(p => p.id === photo.id);
      if (index !== -1) {
        this.openViewer(index);
      }
    },
  },
  watch: {
    searchQuery(val) {
      this.paging.offset = 0;
      this.allLoaded = false;
      this.list_files();
    },
    filters: {
      deep: true,
      handler() {
        this.paging.offset = 0;
        this.allLoaded = false;
        this.list_files();
      }
    }
  },
};
</script>
<style scoped>
.photos-container {
  padding: 20px;
  height: 100%;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 4px; /* Reduced gap for a tighter gallery look */
}

.sticky-header {
  position: sticky;
  top: 0;
  background: rgba(9, 9, 11, 0.9);
  backdrop-filter: blur(8px);
  z-index: 5;
  margin-left: -20px;
  margin-right: -20px;
  padding-left: 20px;
  padding-top: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.bulk-toolbar-container {
  position: fixed;
  bottom: 100px;
  left: 0;
  right: 0;
  display: flex;
  justify-content: center;
  z-index: 1000;
  padding: 0 20px;
}

.bulk-toolbar {
  background: #18181b !important;
  width: 100%;
  max-width: 500px;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
}

.animate-fade-in {
  animation: fadeIn 0.4s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* Scrollbar styling for webkit */
::-webkit-scrollbar {
  width: 8px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: #27272a;
  border-radius: 4px;
}
</style>
