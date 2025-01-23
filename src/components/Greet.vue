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
  <v-sheet style="margin-left: 20px; margin-top: 20px">
    <v-row>
      <H1> Hi there, </H1>
      <p>
        Thank you for using Mediasafe. In the next step we will go through the
        setup process. In the end you will have a backup of your photos and
        videos on your own devices. Without using a third party server.
      </p>
    </v-row>
    <v-row style="margin-top: 40px">
      <v-col offset="2">
        <h3>Are you setting up a new devices or joining a group?</h3>
      </v-col>
    </v-row>
    <v-row>
      <v-col offset="3" md="2">
        <v-btn color="grey" @click="$emit('new_device')"
          ><v-icon>mdi-plus</v-icon>New device</v-btn
        >
      </v-col>
      <v-col md="2">
        <v-btn @click="$emit('join_group')"
          ><v-icon>mdi-devices</v-icon>Join group</v-btn
        >
      </v-col>
    </v-row>
  </v-sheet>
</template>
