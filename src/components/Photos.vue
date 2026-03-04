<template>
  <div class="photos-container">
    <div class="grid">
      <Image
        v-for="(image, index) in images"
        v-bind:key="image.id"
        :path="image"
        @click="openViewer(index)"
        @toggle-favorite="handleToggleFavorite"
      />
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
    paging: {
      offset: 0,
      limit: 50,
    },
    images: [],
    viewerOpen: false,
    currentPhotoIndex: 0,
    favoritesOnly: false,
    observer: null,
  }),
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
    }
  },
  async created() {
    // If mounted as "favorites view" via prop, set filter initially
    if (this.favorites) {
      this.favoritesOnly = true;
    }
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
    setupInfiniteScroll() {
      this.observer = new IntersectionObserver((entries) => {
        if (entries[0].isIntersecting && !this.loading && !this.allLoaded) {
          this.list_files();
        }
      }, { threshold: 0.1 });
      
      const sentinel = document.getElementById('scroll-sentinel');
      if (sentinel) this.observer.observe(sentinel);
    },
    list_files: async function () {
      if (this.loading) return;
      
      this.loading = true;
      console.log("Listing files. Offset:", this.paging.offset, "Query:", this.searchQuery, "PersonFilter:", this.isPersonFilter);
      
      try {
        let response;
        if (this.isPersonFilter && this.searchQuery) {
          response = await invoke("get_person_photos", { personId: this.searchQuery });
          this.allLoaded = true; // person filter returns all at once for now
        } else {
          response = await invoke("list_files", {
            offset: this.paging.offset,
            limit: this.paging.limit,
            query: this.searchQuery ?? "",
            scan: false,
            favoritesOnly: this.favoritesOnly,
          });
        }
        
        const new_images = JSON.parse(response);
        
        if (this.paging.offset === 0) {
          this.images = new_images;
        } else {
          this.images = this.images.concat(new_images);
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
          if (this.favoritesOnly && !isNowFavorite) {
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
  },
  watch: {
    searchQuery(val) {
      this.paging.offset = 0;
      this.allLoaded = false;
      this.list_files();
    },
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
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
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
