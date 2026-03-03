<template>
  <div class="photos-container">
    <div class="d-flex justify-end mb-4" v-if="false"> <!-- Hidden for now, maybe move to App Bar later -->
      <v-btn
        icon
        :color="favoritesOnly ? 'red' : 'grey'"
        class="elevation-2"
        @click="toggleFavoritesFilter"
      >
        <v-icon>mdi-heart</v-icon>
      </v-btn>
    </div>

    <div class="grid">
      <Image
        v-for="(image, index) in images"
        v-bind:key="image.id"
        :path="image"
        @click="openViewer(index)"
        @toggle-favorite="handleToggleFavorite"
      />
    </div>

    <!-- Loading State -->
    <v-row class="mt-4">
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
          v-if="loading === false"
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
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import Image from "./Image.vue";
import PhotoViewer from "./PhotoViewer.vue";
import * as path from "@tauri-apps/api/path";

export default {
  name: "Photos",
  components: { Image, PhotoViewer },
  data: () => ({
    resourcePath: null,
    loading: false,
    paging: {
      offset: 0,
      limit: 50,
    },
    images: [],
    scan: false,
    viewerOpen: false,
    currentPhotoIndex: 0,
    favoritesOnly: false,
  }),
  props: {
    favorites: {
      type: Boolean,
      default: false,
    },
    searchQuery: {
      type: String,
      default: "",
    }
  },
  async created() {
    this.resourcePath = await path.homeDir();
    // If mounted as "favorites view" via prop, set filter initially
    if (this.favorites) {
      this.favoritesOnly = true;
    }
    
    this.list_files();
    window.onscroll = function () {
      if (
        this.loading === false &&
        window.innerHeight + Math.ceil(window.pageYOffset) >=
          document.body.offsetHeight
      ) {
        this.list_files();
      }
    }.bind(this);
  },
  mounted() {},
  methods: {
    list_files: async function () {
      this.loading = true;
      if (this.images.length > 0 && (this.paging.offset + this.paging.limit <= this.images.length)) {
        this.paging.offset = this.paging.offset + this.paging.limit;
      }
      
      console.log("Listing files. Query:", this.searchQuery);
      invoke("list_files", {
        offset: this.paging.offset,
        limit: this.paging.limit,
        query: this.searchQuery ?? "",
        scan: this.scan,
        favoritesOnly: this.favoritesOnly,
      }).then(
        function (response) {
          let new_images = JSON.parse(response);
          if (this.paging.offset === 0) {
            this.images = [];
          }
          this.images = this.images.concat(new_images);
          this.loading = false;
        }.bind(this),
      );
    },
    async handleToggleFavorite(id) {
      const isNowFavorite = await invoke("toggle_favorite", { id: id });

      // Update local state
      const photo = this.images.find((p) => p.id === id);
      if (photo) {
        photo.favorite = isNowFavorite;

        // If in favorites-only mode and it's no longer favorite, remove it
        if (this.favoritesOnly && !isNowFavorite) {
          this.images = this.images.filter((p) => p.id !== id);
        }
      }
    },
    toggleFavoritesFilter() {
      this.favoritesOnly = !this.favoritesOnly;
      this.images = [];
      this.paging.offset = 0;
      this.list_files();
    },
    get_thumbnail: async function (key, path) {
      if (window.localStorage.get(path)) {
        this.images[key].encoded = window.localStorage.get(path);
      } else {
        invoke("get_thumbnail", { path: path }).then((result) => {
          this.images[key].encoded = "data:image/jpeg;base64," + result;
        });
      }
    },
    openViewer(index) {
      this.currentPhotoIndex = index;
      this.viewerOpen = true;
    },
  },
  watch: {
    searchQuery(val) {
      console.log("Photos component: search query changed to", val);
      this.paging.offset = 0;
      this.list_files();
    },
  },
};
</script>
<style scoped>
.photos-container {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
  padding-bottom: 80px; /* Space for FAB or loading */
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
<style scoped>
.photos-container {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.search-bar {
  margin-bottom: 24px;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
  padding-bottom: 80px; /* Space for FAB or loading */
}

/* Scrollbar styling for webkit */
::-webkit-scrollbar {
  width: 8px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: #ccc;
  border-radius: 4px;
}

.faces-scroller {
  /* Hide scrollbar for aesthetics but allow touch/trackpad scroll */
  scrollbar-width: thin;
}
.face-avatar {
  border: 2px solid transparent;
  transition: border-color 0.2s;
}
.face-avatar:hover {
  border-color: #1867c0;
}
</style>
