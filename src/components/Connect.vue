<template>
  <v-row justify="center">
    <v-dialog v-model="dialog" persistent width="auto">
      <template v-slot:activator="{ props }">
        <v-btn icon v-bind="props">
          <v-icon>mdi-plus</v-icon>
        </v-btn>
      </template>
      <v-card>
        <v-card-title class="text-h5">
          Connect a new device.
        </v-card-title>
        <v-card-text>
          Install and open this app on your other device.
          <v-progress-linear indeterminate :height="12" v-if="devices.length == 0"></v-progress-linear>

          <v-list>
            <v-list-item v-for="device in devices" :key="device.ip" :subtitle="device.ip" :title="device.name">

              <template v-slot:append>
                <v-btn color="green-lighten-1" icon="mdi-check" variant="text"></v-btn>
              </template>
            </v-list-item>
          </v-list>

        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="green-darken-1" variant="text" @click="dialog = false">
            Cancel
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-row>
</template>
<script>
import { invoke } from "@tauri-apps/api/core";

export default {
  name: "Connect",
  data: function () {
    return {
      interval: null,
      devices: [],
      dialog: false,
    }
  },
  watch: {
    dialog: function (newV, oldV) {
      if (newV) {
        this.listen();
      }
    }
  },
  methods: {
    listen: function () {
      console.log("Listeing for new connection");
      let interval = setInterval(function () {
        invoke("list_devices").then(function (response) {
          let devices = JSON.parse(response);
          if (this.devices.filter(function (device) {
            let s = devices.filter((d) => d.ip == device.ip);
            return s.length == 0;
          }).length == 0) {
            devices.forEach((d) => {
              this.devices.push(d);
            })
            this.devices.push(devices);
            if (devices.length > 0) {
              // clearInterval(interval)
            }
          }
        }.bind(this));
      }.bind(this), 1000)

      console.log("Listening for incomming connection");
      invoke("listen_for_incomming_connect").then((response) => {
        console.log(response);
      })
    }
  }
}

</script>
