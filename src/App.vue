<script>
import { invoke } from "@tauri-apps/api/core";
import DeviceList from "./components/DeviceList.vue";
import Map from "./components/Map.vue";
import Photos from "./components/Photos.vue";
import Setting from "./components/Setting.vue";
export default {
  components: { DeviceList, Map, Photos, Setting },
  data: () => ({
    clean_install: true,
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
      {
        title: "Settings",
        value: "settings",
        icon: "mdi-wrench",
      },
    ],
  }),
  methods: {
    generate_offer: function () {
      console.log("Generating offer.");
      this.sdp = invoke("generate_offer").then(function (response) {
        return response;
      });
    },
    scan: async function () {
      this.scanning = !this.scanning;
      let data = pictureDirPath;
      let response = await invoke("scan_files", {
        directory: data,
        path: this.resourcePath,
      });
      this.scanning = true;
      return JSON.parse(response);
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
        <vue-qrcode
          value="Hello, World!"
          :options="{ width: 200 }"
        ></vue-qrcode>
        <Greet v-if="clean_install"></Greet>
        <v-app-bar elevation="1" v-if="current_page === 'home'">
          <v-row>
            <v-col md="3" sm="3" lg="3">
              <v-btn color="green" v-if="scanning">
                <v-icon>mdi-reload mdi-spin</v-icon>
                &nbsp;...scanning</v-btn
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
            <v-col md="1" sm="1" lg="1">
              <v-btn color="gray">
                <v-icon>mdi-ok</v-icon>
              </v-btn>
            </v-col>
          </v-row>
        </v-app-bar>

        <Photos v-if="current_page === 'home'" />
        <Map v-if="current_page === 'location'" />
        <DeviceList v-if="current_page === 'devices'" />
        <Setting v-if="current_page === 'settings'" />
      </v-main>
    </v-layout>
    <v-bottom-navigation mode="shift">
      <v-btn value="myphotos" @click="current_page = 'home'" flat>
        <v-icon>mdi-folder</v-icon>
        <span>My photos</span>
      </v-btn>

      <v-btn value="location" @click="current_page = 'location'" flat>
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

      <v-btn value="settings" @click="current_page = 'settings'" flat>
        <v-icon>mdi-wrench</v-icon>
        <span>Setting</span>
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
