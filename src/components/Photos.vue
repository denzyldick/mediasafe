<template>
  <div style="margin-left: 20px; margin-top: 10px">
    <v-row>
      <v-col md="10" lg="10">
        <v-autocomplete
          v-model="search"
          v-model:search="query"
          :items="objects"
        ></v-autocomplete>
      </v-col>
    </v-row>
    <v-row>
      <v-col cols="12" md="10" lg="10">
        <v-row>
          <v-col>
            <div class="grid">
              <Image
                v-for="image in images"
                v-bind:key="image.id"
                :path="image"
              />
            </div>
          </v-col>
        </v-row>
      </v-col>
    </v-row>
    <v-row>
      <v-col class="d-flex justify-center align-center">
        <v-progress-circular
          indeterminate
          v-if="loading === true"
        ></v-progress-circular>
        <v-btn @click="list_files" flat v-if="loading === false"
          >Load more</v-btn
        >
      </v-col>
    </v-row>
  </div>
</template>
<script>
import { invoke } from "@tauri-apps/api/core";
import Image from "./Image.vue";
export default {
  name: "Photos",
  components: { Image },
  data: () => ({
    resourcePath: "/home/denzyl/Pictures",
    search: null,
    query: null,
    loading: false,
    paging: {
      offset: 0,
      limit: 30,
    },
    objects: [],
    images: [],
    scan: true,
  }),
  mounted() {
    this.scan_folder();
  },
  created() {
    this.list_files();
    window.onscroll = function () {
      if (
        window.innerHeight + Math.ceil(window.pageYOffset) >=
        document.body.offsetHeight
      ) {
        this.list_files();
      }
    }.bind(this);
  },
  methods: {
    scan_folder: async function () {
      let data = pictureDirPath;
      invoke("scan_files", { directory: data, path: this.resourcePath }).then(
        function (response) {
          console.log(response);
          return JSON.parse(response);
        },
      );
    },
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
        path: this.resourcePath,
        offset: this.paging.offset,
        limit: this.paging.limit,
        query: this.search ?? "",
        scan: this.scan,
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
      invoke("get_thumbnail", { path: path }).then((result) => {
        this.images[key].encoded = "data:image/jpeg;base64," + result;
      });
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
<style>
/* input:before { */
/*   box-shadow: none !important; */
/* } */
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  grid-template-rows: repeat(auto-fit, minmax(0, 1fr));
  grid-gap: 10px;
}

img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center;
}
</style>
