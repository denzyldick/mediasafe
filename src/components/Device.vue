<template>
  <v-card flat max-width="344">
    <qrcode-svg v-if="value !== null" :value />
    <v-card-title>Arch-linux </v-card-title>
    <v-card-subtitle>This device</v-card-subtitle>
    <v-card-actions>
      <v-btn color="green-lighten-2" text="sync"></v-btn>
      <v-spacer></v-spacer>

      <v-btn
        :icon="show ? 'mdi-chevron-up' : 'mdi-chevron-down'"
        @click="show = !show"
      ></v-btn>
    </v-card-actions>

    <v-expand-transition>
      <div v-show="show">
        <v-divider></v-divider>

        <v-card-text>more information </v-card-text>
      </div>
    </v-expand-transition>
  </v-card>
</template>
<script>
import QrcodeVue, { QrcodeCanvas, QrcodeSvg } from "qrcode.vue";
export default {
  components: {
    QrcodeVue,
    QrcodeCanvas,
    QrcodeSvg,
  },
  data: () => ({
    show: false,
    value: null,
  }),
  async created() {
    let port = "9489";
    let ip = "192.168.68.115";

    let path = "/new-device";
    let message = "http://" + ip + ":" + port + path;
    this.value = message;
  },
};
</script>
