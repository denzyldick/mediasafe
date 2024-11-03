<template>
  <div>
    <v-row>
      <v-col>
        <v-toolbar flat permanent>
          <v-toolbar-title>All your devices</v-toolbar-title>
          <v-toolbar-subtitle
            v-if="device.name !== null && device.ip !== null"
            >{{ device.name }}</v-toolbar-subtitle
          >
          <v-spacer></v-spacer>
          <Connect />
        </v-toolbar>
      </v-col>
    </v-row>
    <v-row style="margin-top: 10px">
      <v-col>
        <v-list>
          <v-list-item
            v-for="file in devices"
            :key="file.title"
            :title="file.title"
            :subtitle="file.subtitle"
          >
            <template v-slot:prepend>
              <v-avatar :color="file.color">
                <v-icon color="white">{{ file.icon }}</v-icon>
              </v-avatar>
            </template>
            <template v-slot:append>
              <v-icon
                v-if="file.up_to_date && file.syncing == false"
                color="green-lighten-1"
                icon="mdi-check"
                variant="text"
              ></v-icon>

              <v-icon
                v-if="file.up_to_date == false && file.syncing == false"
                color="red-lighten-1"
                icon="mdi-sync-alert"
                variant="text"
              ></v-icon>

              <v-icon
                v-if="file.up_to_date == false && file.syncing == true"
                color="grey-lighten-1"
                icon="fa:fas mdi-sync fa-spin"
              ></v-icon>
            </template>
          </v-list-item>
        </v-list>
      </v-col>
    </v-row>
  </div>
</template>
<script>
import { invoke } from "@tauri-apps/api/core";
import Connect from "./Connect.vue";
export default {
  name: "DeviceList",
  components: {
    Connect,
  },
  data: function () {
    return {
      device: {
        name: null,
        ip: null,
      },
      devices: [
        {
          color: "red",
          icon: "mdi-android",
          title: "Samsung Galaxy ",
          subtitle: "Please turn on device. Last sync was 1 week ago.",
          up_to_date: false,
          syncing: false,
        },
        {
          color: "blue",
          icon: "mdi-apple",
          title: "Iphone 12",
          subtitle: "",
          up_to_date: true,
          syncing: false,
        },
        {
          color: "yellow",
          icon: "mdi-microsoft",
          title: "Windows desktop",
          subtitle: "Syncing in progress...",
          up_to_date: false,
          syncing: true,
        },
      ],
    };
  },
  methods: {
    join: function () {
      console.log("Joining");
      invoke("join_network", { ip: "192.168.68.117" }).then((response) => {
        console.log(response);
      });
    },
    listen_for_incomming_connect() {
      console.log("Listening for incomming connection");
      invoke("listen_for_incomming_connect").then((response) => {
        console.log(response);
      });
    },
  },
};
</script>
