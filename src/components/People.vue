<template>
  <v-container class="pa-6">
    <div class="d-flex align-center justify-space-between mb-8">
      <div>
        <div class="d-flex align-center mb-1">
          <v-icon color="#a1a1aa" size="28" class="mr-3">mdi-account-group-outline</v-icon>
          <h1 class="text-h4 font-weight-bold text-zinc-primary">People</h1>
        </div>
        <div class="text-subtitle-1 text-zinc-secondary">Manage and name identified people</div>
      </div>
    </div>

    <!-- Unified People & Faces Grid -->
    <v-row>
      <!-- Named People First -->
      <v-col cols="6" sm="4" md="3" lg="2" v-for="person in people" :key="person.id">
        <v-card class="border-subtle text-center pt-4 pb-2 h-100" variant="flat" @click="viewPerson(person)">
          <v-avatar size="100" class="mx-auto mb-3 border-subtle">
            <v-img :src="getFaceImageSrc(person.representative_crop, person.encoded)"></v-img>
          </v-avatar>
          <v-card-title class="text-subtitle-1 font-weight-bold text-zinc-primary px-2 pb-0">
            {{ person.name }}
          </v-card-title>
          <div class="text-caption text-zinc-muted pb-2">Identified</div>
        </v-card>
      </v-col>

      <!-- Unnamed Faces -->
      <v-col cols="6" sm="4" md="3" lg="2" v-for="face in unnamedFaces" :key="face.face_id">
        <v-card class="border-subtle h-100" variant="flat">
          <v-img :src="getFaceImageSrc(face.crop_path, face.encoded)" aspect-ratio="1" cover></v-img>
          <v-card-actions class="pa-2">
            <v-btn block size="small" variant="flat" color="white" class="text-none font-weight-bold" @click="promptName(face)">
              Who is this?
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>

      <!-- Empty State -->
      <v-col v-if="people.length === 0 && unnamedFaces.length === 0" cols="12" class="text-center py-12">
        <v-icon size="64" color="#71717a" class="mb-4 opacity-20">mdi-face-recognition</v-icon>
        <div class="text-zinc-muted">No faces detected yet. Try syncing your library!</div>
      </v-col>
    </v-row>

    <!-- Name Dialog -->
    <v-dialog v-model="nameDialog" max-width="400">
      <v-card class="border-subtle pa-4" color="background">
        <v-card-title class="text-zinc-primary px-0">Who is this?</v-card-title>
        <v-card-text class="px-0">
          <v-avatar size="120" class="mx-auto d-block mb-6 border-subtle">
            <v-img v-if="activeFace" :src="getFaceImageSrc(activeFace.crop_path, activeFace.crop_path_b64)"></v-img>
          </v-avatar>
          <v-text-field
            v-model="newName"
            label="Name"
            variant="solo-filled"
            bg-color="rgba(255,255,255,0.05)"
            hide-details
            autofocus
            @keyup.enter="saveName"
          ></v-text-field>
        </v-card-text>
        <v-card-actions class="px-0 pt-4">
          <v-spacer></v-spacer>
          <v-btn variant="text" color="grey" @click="nameDialog = false">Cancel</v-btn>
          <v-btn variant="flat" color="white" :disabled="!newName" @click="saveName">Save</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Person Gallery -->
    <v-dialog v-model="galleryDialog" fullscreen transition="dialog-bottom-transition">
      <v-card color="background" class="d-flex flex-column">
        <v-app-bar elevation="0" class="border-subtle" color="background">
          <v-btn icon="mdi-close" @click="galleryDialog = false" color="#71717a"></v-btn>
          <v-toolbar-title class="text-zinc-primary font-weight-bold">
            {{ activePerson?.name }}
          </v-toolbar-title>
        </v-app-bar>
        <v-main class="overflow-y-auto">
          <div class="pa-6">
            <Photos v-if="galleryDialog" :search-query="activePerson?.id" is-person-filter />
          </div>
        </v-main>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script>
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import Photos from "./Photos.vue";

export default {
  name: "People",
  components: { Photos },
  data: () => ({
    people: [],
    unnamedFaces: [],
    showUnnamed: false,
    nameDialog: false,
    galleryDialog: false,
    activeFace: null,
    activePerson: null,
    newName: "",
  }),
  async mounted() {
    this.fetchData();
  },
  methods: {
    async fetchData() {
      const peopleStr = await invoke("get_people");
      this.people = JSON.parse(peopleStr);

      const unnamedStr = await invoke("get_unnamed_faces");
      this.unnamedFaces = JSON.parse(unnamedStr);
    },
    getFaceImageSrc(crop_path, encoded) {
      if (encoded) return encoded;
      return '';
    },
    promptName(face) {
      this.activeFace = face;
      this.newName = "";
      this.nameDialog = true;
    },
    async saveName() {
      if (!this.newName || !this.activeFace) return;
      
      await invoke("assign_name_to_face", { 
        faceId: this.activeFace.face_id, 
        name: this.newName 
      });
      
      this.nameDialog = false;
      this.fetchData();
    },
    viewPerson(person) {
      this.activePerson = person;
      this.galleryDialog = true;
    }
  },
  watch: {
    showUnnamed(val) {
      this.fetchData();
    }
  }
};
</script>
