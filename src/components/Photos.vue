<template>
  <div class="photos-container">
    <div class="search-bar d-flex align-center gap-2">
      <v-autocomplete
        v-model="search"
        v-model:search="query"
        :items="objects"
        prepend-inner-icon="mdi-magnify"
        variant="solo"
        density="comfortable"
        placeholder="Search photos..."
        hide-details
        class="rounded-lg elevation-2 flex-grow-1"
      >
        <template v-slot:prepend-item>
          <div v-if="faces.length > 0" class="faces-scroller pa-2 d-flex flex-nowrap" style="overflow-x: auto; gap: 8px;">
            <v-avatar
              v-for="face in faces"
              :key="face.face_id"
              size="48"
              class="cursor-pointer face-avatar"
              @click="addFaceToSearch(face)"
              color="grey-lighten-2"
            >
              <v-img :src="getFaceImageSrc(face.crop_path)"></v-img>
            </v-avatar>
          </div>
          <v-divider v-if="faces.length > 0" class="my-1"></v-divider>
        </template>
      </v-autocomplete>

      <v-btn
        icon
        :color="favoritesOnly ? 'red' : 'grey'"
        class="ml-2 elevation-2"
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
          color="primary"
          v-if="loading === true"
        ></v-progress-circular>
        <v-btn
          @click="list_files"
          variant="text"
          color="primary"
          v-if="loading === false"
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
    search: null,
    query: null,
    loading: false,
    paging: {
      offset: 0,
      limit: 50,
    },
    objects: [],
    images: [],
    faces: [],
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
  },
  async created() {
    this.resourcePath = await path.homeDir();
    // If mounted as "favorites view" via prop, set filter initially
    if (this.favorites) {
      this.favoritesOnly = true;
    }
    
    // Fetch faces
    invoke("get_faces").then(response => {
      this.faces = JSON.parse(response);
    });

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
      if (this.images.length > 0) {
        this.paging.offset = this.paging.offset + this.paging.limit;
      }
      let s = this.search ?? "";
      if (s.length > 0) {
        this.paging.offset = 0;
      }
      console.log("Listing files");
      invoke("list_files", {
        offset: this.paging.offset,
        limit: this.paging.limit,
        query: this.search ?? "",
        scan: this.scan,
        favoritesOnly: this.favoritesOnly,
      }).then(
        function (response) {
          let new_images = JSON.parse(response);
          console.log(response);
          if (s.length > 0) {
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
    list_objects: function (val) {
      if (val.length > 0) {
        invoke("list_objects", { query: val }).then(
          function (response) {
            this.objects = JSON.parse(response);
          }.bind(this),
        );
      }
    },
    search_by_object: function (tag) {
      let result = invoke("search_by_object").then((result) => {
        console.log(result);
      });
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
    getFaceImageSrc(crop_path) {
      if (!crop_path) return '';
      const converted = convertFileSrc(crop_path);
      if (converted === crop_path && crop_path.startsWith('/')) {
         return `http://asset.localhost${encodeURI(crop_path)}`;
      }
      return converted;
    },
    addFaceToSearch(face) {
      this.search = (this.search || '') ? this.search + ' ' + face.photo_id : face.photo_id;
      // Triggers watcher to list_files automatically
    },
  },
  watch: {
    query(val) {
      this.list_objects(val);
    },
    search(val) {
      console.log("Searching ");
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
