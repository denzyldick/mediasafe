<script>
import { invoke } from "@tauri-apps/api/core";
import DeviceList from "./components/DeviceList.vue";
import Map from "./components/Map.vue";
import Photos from "./components/Photos.vue";
export default {
  components: { DeviceList, Map, Photos },
  data: () => ({
    scanning: false,
    search: null,
    current_page: "photos",
    group: null,
    items: [
      {
        title: "Devices",
        value: "devices",
        icon: "mdi-laptop",
      },
      {
        title: "Folders",
        value: "bar",
        icon: "mdi-folder",
      },
    ],
  }),
  methods: {
    generate_offer: function () {
      console.log("Generating offer.");
      this.sdp = invoke("generate_offer").then(function (response) {
        console.log(response);
        return response;
      });
    },
    scan: function () {
      this.scanning = !this.scanning;
    },
  },
  watch: {
    group() {
      this.drawer = false;
    },
  },
};
</script>

<template>
  <v-app>
    <v-layout>
      <v-main>
        <v-app-bar elevation="1" v-if="current_page === 'home'">
          <v-row>
            <v-col>
              <v-btn color="green" v-if="scanning">
                <v-icon>mdi-reload mdi-spin</v-icon> &nbsp;...scanning</v-btn
              >
              <v-btn
                flat
                color="grey"
                v-if="scanning === false"
                @click="scan()"
              >
                <v-icon>mdi-ok</v-icon> last scan 10s ago
              </v-btn>
            </v-col>
          </v-row>
        </v-app-bar>

        <Photos v-if="current_page === 'home'" />
        <Map v-if="current_page === 'location'" />
        <DeviceList v-if="current_page === 'devices'" />
      </v-main>
    </v-layout>
    <v-bottom-navigation mode="shift">
      <v-btn value="myphotos" @click="current_page = 'home'" flat>
        <v-icon>mdi-folder</v-icon>
        <span>My photos</span>
      </v-btn>

      <v-btn value="locations" @click="current_page = 'locations'" flat>
        <v-icon>mdi-map</v-icon>
        <span>My locations</span>
      </v-btn>

      <v-btn value="devices" @click="current_page = 'devices'" flat>
        <v-icon>mdi-laptop</v-icon>
        <span>Devices</span>
      </v-btn>

      <v-btn value="starred" @click="current_page = 'starred'" flat>
        <v-icon>mdi-heart</v-icon>
        <span>Favorites</span>
      </v-btn>
    </v-bottom-navigation>
  </v-app>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
