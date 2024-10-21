<template>
  <v-row>
    <v-col offset-md="2" style="margin-top: 40px" md="8">
      <v-sheet class="mx-auto">
        <v-form ref="form">
          <v-row>
            <v-col>
              <v-file-input
                v-model="name"
                :counter="10"
                :rules="nameRules"
                required
                label="Which folder to backup?"
                placeholder="Make sure your photos are in this folder."
                prepend-icon="mdi-camera"
                webkitdirectory
              ></v-file-input>
            </v-col>
          </v-row>
          <v-row>
            <v-col>
              <v-select
                v-model="select"
                :items="items"
                :rules="[(v) => !!v || 'Item is required']"
                label="Which hardware to use?"
                required
              ></v-select>
            </v-col>
          </v-row>

          <v-row>
            <v-col>
              <v-checkbox
                v-model="checkbox"
                :rules="[(v) => !!v || 'You must agree to continue!']"
                label="Detect objects in your photos?"
                required
              ></v-checkbox>
            </v-col>
          </v-row>
          <v-row>
            <v-col>
              <v-checkbox
                v-model="checkbox"
                :rules="[(v) => !!v || 'You must agree to continue!']"
                label="Enable on startup"
                required
              ></v-checkbox>
            </v-col>
          </v-row>

          <v-row>
            <v-col offset-md="8" md="2">
              <v-btn class="mt-4" color="info" block @click="resetValidation">
                <v-icon>mdi-trash-can</v-icon> Default
              </v-btn>
            </v-col>
            <v-col md="2">
              <v-btn class="mt-4" color="success" block @click="validate">
                <v-icon> mdi-floppy </v-icon> Save
              </v-btn>
            </v-col>
          </v-row>
        </v-form>
      </v-sheet>
    </v-col>
  </v-row>
</template>
<script>
export default {
  data: () => ({
    name: "",
    nameRules: [
      (v) => !!v || "Path is required",
      (v) => (v && v.length <= 10) || "Name must be 10 characters or less",
    ],
    select: null,
    items: ["Gpu", "Cpu"],
    checkbox: false,
  }),

  methods: {
    async validate() {
      const { valid } = await this.$refs.form.validate();

      if (valid) alert("Form is valid");
    },
    reset() {
      this.$refs.form.reset();
    },
    resetValidation() {
      this.$refs.form.resetValidation();
    },
  },
};
</script>
