<template>
  <v-container class="pa-6" style="background-color: #fafafa !important; min-height: 100%;">
    <div class="d-flex align-center justify-space-between mb-8">
      <div>
        <div class="d-flex align-center mb-1">
          <v-icon color="#18181b" size="28" class="mr-3">mdi-account-group-outline</v-icon>
          <h1 class="text-h4 font-weight-bold text-zinc-primary">People</h1>
        </div>
        <div class="text-subtitle-1 text-zinc-secondary">Manage and name identified people</div>
      </div>
    </div>

    <!-- Unified People & Faces Grid -->
    <v-row>
      <!-- Named People First -->
      <v-col cols="6" sm="4" md="3" lg="2" v-for="person in people" :key="person.id">
        <v-card class="border-subtle text-center pt-4 pb-2 h-100 person-card" variant="flat" color="#ffffff" rounded="xl">
          <div @click="viewPerson(person)" class="cursor-pointer">
            <v-avatar size="100" class="mx-auto mb-3 border-subtle">
              <v-img :src="getFaceImageSrc(person.representative_crop, person.encoded)"></v-img>
            </v-avatar>
            <v-card-title class="text-subtitle-1 font-weight-bold text-zinc-primary px-2 pb-0">
              {{ person.name }}
            </v-card-title>
          </div>
          <v-card-actions class="justify-center">
            <v-btn icon="mdi-pencil-outline" size="x-small" color="#71717a" @click="openManageDialog(person)"></v-btn>
          </v-card-actions>
        </v-card>
      </v-col>

      <!-- Unnamed Faces -->
      <v-col cols="6" sm="4" md="3" lg="2" v-for="group in groupedUnnamedFaces" :key="group.representative.face_id">
        <v-card class="border-subtle h-100 unnamed-card" variant="flat" color="#ffffff" rounded="xl">
          <div class="position-relative">
            <v-img :src="getFaceImageSrc(group.representative.crop_path, group.representative.encoded)" aspect-ratio="1" cover class="rounded-t-xl"></v-img>
            <v-chip
              v-if="group.faces.length > 1"
              size="x-small"
              color="#18181b"
              variant="flat"
              class="position-absolute font-weight-bold text-white"
              style="top: 8px; right: 8px; z-index: 2;"
            >
              {{ group.faces.length }}
            </v-chip>
          </div>
          <v-card-actions class="pa-2 bg-zinc-50">
            <v-btn block size="small" variant="flat" class="siegu-btn" @click="promptName(group)">
              Who is this?
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>

      <!-- Empty State -->
      <v-col v-if="people.length === 0 && unnamedFaces.length === 0" cols="12" class="text-center py-12">
        <v-icon size="64" color="#d4d4d8" class="mb-4">mdi-face-recognition</v-icon>
        <div class="text-zinc-muted">No faces detected yet. Try scanning your library!</div>
      </v-col>
    </v-row>

    <!-- Name Dialog -->
    <v-dialog v-model="nameDialog" max-width="400">
      <v-card class="border-subtle pa-4" color="#ffffff" rounded="xl">
        <v-card-title class="text-zinc-primary px-0">Who is this?</v-card-title>
        <v-card-text class="px-0">
          <v-avatar size="120" class="mx-auto d-block mb-6 border-subtle">
            <v-img v-if="activeFace" :src="getFaceImageSrc(activeFace.crop_path, activeFace.encoded)"></v-img>
          </v-avatar>
          <v-text-field
            v-model="newName"
            label="Name"
            variant="solo-filled"
            bg-color="#f4f4f5"
            hide-details
            flat
            rounded="lg"
            class="siegu-field"
            autofocus
            @keyup.enter="saveName"
          ></v-text-field>
        </v-card-text>
        <v-card-actions class="px-0 pt-4">
          <v-spacer></v-spacer>
          <v-btn variant="text" color="#71717a" @click="nameDialog = false">Cancel</v-btn>
          <v-btn variant="flat" class="siegu-btn" :disabled="!newName" @click="saveName">Save</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Manage Person Dialog (Rename/Merge) -->
    <v-dialog v-model="manageDialog" max-width="450">
      <v-card class="border-subtle pa-4" color="#ffffff" rounded="xl">
        <v-card-title class="text-zinc-primary px-0">Manage {{ activePerson?.name }}</v-card-title>
        <v-card-text class="px-0">
          <v-tabs v-model="manageTab" bg-color="transparent" color="#18181b" grow class="mb-4 border-subtle rounded-lg overflow-hidden">
            <v-tab value="rename" class="text-none">Rename</v-tab>
            <v-tab value="merge" class="text-none">Merge</v-tab>
          </v-tabs>

          <v-window v-model="manageTab">
            <v-window-item value="rename">
              <v-text-field
                v-model="newName"
                label="New Name"
                variant="solo-filled"
                bg-color="#f4f4f5"
                hide-details
                flat
                rounded="lg"
                class="siegu-field"
                @keyup.enter="renamePerson"
              ></v-text-field>
              <div class="mt-4">
                <v-btn block variant="flat" class="siegu-btn" @click="renamePerson">Update Name</v-btn>
              </div>
            </v-window-item>

            <v-window-item value="merge">
              <div class="text-body-2 text-zinc-secondary mb-4">
                Combine all photos of <b>{{ activePerson?.name }}</b> with another person.
              </div>
              <v-select
                v-model="mergeTargetId"
                :items="otherPeople"
                item-title="name"
                item-value="id"
                label="Select Target Person"
                variant="solo-filled"
                bg-color="#f4f4f5"
                flat
                rounded="lg"
                class="siegu-field"
              ></v-select>
              <div class="mt-4">
                <v-btn block color="error" variant="flat" :disabled="!mergeTargetId" @click="mergePerson" class="rounded-lg text-none font-weight-bold">Confirm Merge</v-btn>
              </div>
            </v-window-item>
          </v-window>
        </v-card-text>
        <v-card-actions class="px-0 pt-4">
          <v-spacer></v-spacer>
          <v-btn variant="text" color="#71717a" @click="manageDialog = false">Close</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Person Gallery -->
    <v-dialog v-model="galleryDialog" fullscreen transition="dialog-bottom-transition">
      <v-card color="#fafafa" class="d-flex flex-column">
        <v-app-bar elevation="0" class="border-bottom-subtle" color="#ffffff">
          <v-btn icon="mdi-close" @click="galleryDialog = false" color="#18181b"></v-btn>
          <v-toolbar-title class="text-zinc-primary font-weight-bold">
            {{ activePerson?.name }}
          </v-toolbar-title>
        </v-app-bar>
        <v-main class="overflow-y-auto" style="background-color: #fafafa !important;">
          <div class="pa-6">
            <Photos v-if="galleryDialog" :search-query="activePerson?.id" is-person-filter />
          </div>
        </v-main>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<style scoped>
.person-card, .unnamed-card {
  transition: all 0.2s ease;
  border: 1px solid rgba(0, 0, 0, 0.05) !important;
}

.person-card:hover, .unnamed-card:hover {
  background: #ffffff !important;
  border-color: rgba(0, 0, 0, 0.1) !important;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.05) !important;
}

.bg-zinc-50 {
  background-color: #f9fafb !important;
}

.siegu-field :deep(.v-field) {
  background: #f4f4f5 !important;
  border: 1px solid rgba(0, 0, 0, 0.05) !important;
  border-radius: 12px !important;
  color: #18181b !important;
}
</style>

<script>
import { invoke } from "@tauri-apps/api/core";
import Photos from "./Photos.vue";

export default {
  name: "People",
  components: { Photos },
  data: () => ({
    people: [],
    unnamedFaces: [],
    nameDialog: false,
    manageDialog: false,
    galleryDialog: false,
    activeFace: null,
    activeGroup: null,
    activePerson: null,
    newName: "",
    manageTab: "rename",
    mergeTargetId: null,
  }),
  computed: {
    otherPeople() {
      return this.people.filter(p => p.id !== this.activePerson?.id);
    },
    groupedUnnamedFaces() {
      if (this.unnamedFaces.length === 0) return [];
      const groups = [];
      const SIMILARITY_THRESHOLD = 0.85;
      this.unnamedFaces.forEach(face => {
        if (!face.embedding || face.embedding.length === 0) {
          groups.push({ representative: face, faces: [face] });
          return;
        }
        let addedToGroup = false;
        for (const group of groups) {
          if (!group.representative.embedding) continue;
          let dotProduct = 0;
          for (let i = 0; i < face.embedding.length; i++) {
            dotProduct += face.embedding[i] * group.representative.embedding[i];
          }
          if (dotProduct > SIMILARITY_THRESHOLD) {
            group.faces.push(face);
            addedToGroup = true;
            break;
          }
        }
        if (!addedToGroup) groups.push({ representative: face, faces: [face] });
      });
      return groups;
    }
  },
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
      return encoded || '';
    },
    promptName(group) {
      this.activeGroup = group;
      this.activeFace = group.representative;
      this.newName = "";
      this.nameDialog = true;
    },
    async saveName() {
      if (!this.newName || !this.activeGroup) return;
      const promises = this.activeGroup.faces.map(face => 
        invoke("assign_name_to_face", { faceId: face.face_id, name: this.newName })
      );
      await Promise.all(promises);
      this.nameDialog = false;
      this.activeGroup = null;
      this.fetchData();
    },
    viewPerson(person) {
      this.activePerson = person;
      this.galleryDialog = true;
    },
    openManageDialog(person) {
      this.activePerson = person;
      this.newName = person.name;
      this.manageTab = "rename";
      this.mergeTargetId = null;
      this.manageDialog = true;
    },
    async renamePerson() {
      if (!this.activePerson || !this.newName) return;
      await invoke("rename_person", { id: this.activePerson.id, newName: this.newName });
      this.manageDialog = false;
      this.fetchData();
    },
    async mergePerson() {
      if (!this.activePerson || !this.mergeTargetId) return;
      await invoke("merge_people", { fromId: this.activePerson.id, toId: this.mergeTargetId });
      this.manageDialog = false;
      this.fetchData();
    }
  }
};
</script>
