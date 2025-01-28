<template>
  <v-row>
    <v-col offset-md="2" style="margin-top: 40px; padding-left: 20px" md="8">
      <div class="text-h5">Configuration</div>
      <div class="text-grey font-weight-thin">
        The configuration is being stored in {{ dataDir }}. <br />

        This is a
        <a href="https://www.sqlite.org" class="text-grey">SQLite</a> file that
        you can export.
      </div>
      <br />
      <v-sheet class="mx-auto">
        <v-form ref="form">
          <v-row>
            <v-col class="font-weight-thin">
              MediaSafe won't randomly scan folders for pictures. It only scans
              the folders you tell it to.
            </v-col>
          </v-row>
          <v-row>
            <v-col>
              <v-btn @click="select_directory" class="info"
                >Select folder(s)</v-btn
              >
            </v-col>
          </v-row>
          <v-row v-if="directories.length > 0">
            <v-col> You have selected the following directories: </v-col>
            <v-col>
              <v-list>
                <v-list-item
                  v-for="directory in directories"
                  :key="directory.title"
                  :title="directory.title"
                >
                  <template v-slot:append>
                    <v-btn
                      icon="mdi-delete"
                      size="small"
                      variant="text"
                      @click="remove_directory(directory.value)"
                    ></v-btn>
                  </template>
                </v-list-item>
              </v-list>
            </v-col>
          </v-row>
        </v-form>
      </v-sheet>
    </v-col>
  </v-row>
</template>
<script>
import * as path from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
export default {
  data: () => ({
    directories: [],
    dataDir: null,
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
    async select_directory() {
      const directory = await open({
        multiple: true,
        directory: true,
      });

      this.directories = directory.map((dir) => {
        return {
          title: dir,
          value: dir,
          props: {
            color: "primary",
            prependIcon: "mdi-folder",
            appendIcon: "mdi-close",
          },
        };
      });
    },
    remove_directory(path) {
      this.directories = this.directories.filter((dir) => dir.value !== path);
    },
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
  async mounted() {
    console.log("HELLOWORLD");
    this.dataDir = await this.path.homeDir();
  },
};
</script>
