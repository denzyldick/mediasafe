<script>
export default {
  name: "Greet",
  data() {
    return {
      active: true,
    };
  },
};
</script>

<template>
  <v-dialog max-width="500">
    <template>
      <v-card title="Hello, there!">
        <v-card-text>
          Thank you, for using Medisafe.<br />In this steps we will configure
          MediaSafe. <br />

          In the end you will have a multiple devices with access to your
          photos. Without third-party in between.
        </v-card-text>

        <v-card-actions>
          <v-spacer></v-spacer>

          <v-btn text="Next" @click="isActive.value = false"></v-btn>
        </v-card-actions>
      </v-card>
    </template>
  </v-dialog>
</template>
