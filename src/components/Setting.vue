<template>
  <v-container class="fill-height" fluid style="background-color: #f5f5f5">
    <v-row justify="center">
      <v-col cols="12" md="8" lg="6">
        <div class="d-flex align-center mb-6">
          <v-icon color="primary" size="x-large" class="mr-3">mdi-cog</v-icon>
          <h1 class="text-h4 font-weight-bold text-grey-darken-3">Settings</h1>
        </div>

        <!-- Storage Config Card -->
        <v-card class="mb-6" elevation="2" rounded="lg">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon color="secondary" size="large">mdi-database</v-icon>
            </template>
            <v-card-title class="text-h6">Storage Configuration</v-card-title>
            <v-card-subtitle>Where your media database is stored</v-card-subtitle>
          </v-card-item>

          <v-card-text>
            <v-alert
              color="info"
              variant="tonal"
              icon="mdi-information"
              class="mb-2"
            >
              Your configuration database is located at:
              <div class="font-weight-bold mt-1 text-body-2">{{ dataDir }}/database.sql</div>
            </v-alert>
            <div class="text-caption text-grey">
              This is a standard <a href="https://www.sqlite.org" class="text-decoration-none font-weight-bold text-primary">SQLite</a> file. You can back it up or export it at any time.
            </div>
          </v-card-text>
        </v-card>

        <!-- Library Config Card -->
        <v-card elevation="2" rounded="lg">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon color="secondary" size="large">mdi-image-multiple</v-icon>
            </template>
            <v-card-title class="text-h6">Media Library</v-card-title>
            <v-card-subtitle>Manage which folders MediaSafe can access</v-card-subtitle>
          </v-card-item>

          <v-card-text>
            <v-alert
              type="success"
              variant="tonal"
              class="mb-4 text-body-2"
              icon="mdi-shield-check"
            >
              <strong>Privacy First:</strong> MediaSafe only scans the specific folders you add below. It will never access other parts of your system.
            </v-alert>

            <v-row class="align-center mb-2">
              <v-col>
                <div class="text-subtitle-1 font-weight-medium">Watched Folders</div>
              </v-col>
              <v-col cols="auto">
                 <v-btn
                  color="primary"
                  prepend-icon="mdi-folder-plus"
                  variant="flat"
                  @click="select_directory"
                >
                  Add Folder
                </v-btn>
              </v-col>
            </v-row>

            <v-expand-transition>
              <div v-if="directories.length > 0">
                 <v-list lines="one" class="bg-grey-lighten-5 rounded-lg border">
                  <v-list-item
                    v-for="(directory, index) in directories"
                    :key="directory.value"
                    :title="directory.title"
                  >
                    <template v-slot:prepend>
                       <v-icon color="amber-darken-2">mdi-folder</v-icon>
                    </template>
                    
                    <template v-slot:append>
                      <v-btn
                        icon="mdi-delete"
                        variant="text"
                        color="error"
                        size="small"
                        @click="remove_directory(directory.value)"
                        title="Remove folder"
                      ></v-btn>
                    </template>
                     <v-divider v-if="index < directories.length - 1"></v-divider>
                  </v-list-item>
                </v-list>
              </div>
              <div v-else class="text-center py-8 text-grey-darken-1 border border-dashed rounded-lg">
                <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-folder-open-outline</v-icon>
                <div>No folders added yet.</div>
                <div class="text-caption">Click "Add Folder" to get started.</div>
              </div>
            </v-expand-transition>
          </v-card-text>
          
          <v-divider></v-divider>
          
          <v-card-actions class="pa-4">
             <v-spacer></v-spacer>
             <v-btn
              color="primary"
              size="large"
              variant="elevated"
              :disabled="directories.length === 0"
              @click="$emit('done')"
              append-icon="mdi-arrow-right"
            >
              Continue to Library
            </v-btn>
          </v-card-actions>
        </v-card>

      </v-col>
    </v-row>
  </v-container>
</template>
<script>
import { invoke } from "@tauri-apps/api/core";
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
        invoke("add_directory", { path: dir, configPath: this.dataDir }).then(() => { 
          console.log("Added directory:", dir);
          this.list_directories();
          invoke("scan_files");
        }).catch(err => {
          console.error("Failed to add directory:", err);
        });
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
      invoke("remove_directory", { path: path, configPath: this.dataDir }).then(() => {
        this.list_directories();
      });
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
    list_directories() {
      console.log("Listing directories. Config path:", this.dataDir);
      invoke("list_directories", { path: this.dataDir }).then((response) => {
        console.log("Raw response from list_directories:", response);
        const dirs = JSON.parse(response);
        console.log("Parsed directories:", dirs);
        this.directories = dirs.map(dir => ({
          title: dir,
          value: dir,
          props: {
            color: "primary",
            prependIcon: "mdi-folder",
            appendIcon: "mdi-close",
          }
        }));
      }).catch(err => {
        console.error("Failed to list directories:", err);
      });
    },
  },
  async mounted() {
    this.dataDir = await path.homeDir();
    this.list_directories();
  },
};
</script>
