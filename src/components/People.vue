<template>
  <v-container class="pa-6 bg-siegu-main min-h-100">
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
            <v-btn variant="flat" class="siegu-btn px-4" size="small" @click="openManageDialog(person)">
              <div class="d-flex align-center">
                <div class="siegu-icon-circle siegu-icon-circle-sm mr-2">
                  <v-icon size="10" color="white">mdi-pencil-outline</v-icon>
                </div>
                <span class="text-white">Rename</span>
              </div>
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>

      <!-- Unnamed Faces -->
      <v-col cols="6" sm="4" md="3" lg="2" v-for="group in groupedUnnamedFaces" :key="group.representative.face_id">
        <v-card class="border-subtle h-100 unnamed-card" variant="flat" color="#ffffff" rounded="xl">
          <div class="pos-rel">
            <v-img :src="getFaceImageSrc(group.representative.crop_path, group.representative.encoded)" aspect-ratio="1" cover class="rounded-t-xl"></v-img>
            <v-chip
              v-if="group.faces.length > 1"
              size="x-small"
              color="#18181b"
              variant="flat"
              class="font-weight-bold text-white pos-tr-8"
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
          <v-btn variant="flat" class="siegu-btn px-6 mr-2" @click="nameDialog = false">
             <div class="d-flex align-center">
               <div class="siegu-icon-circle siegu-icon-circle-sm mr-2">
                 <v-icon size="12" color="white">mdi-close</v-icon>
               </div>
               <span class="text-white">Cancel</span>
             </div>
          </v-btn>
          <v-btn variant="flat" class="siegu-btn px-6" :disabled="!newName" @click="saveName">
             <div class="d-flex align-center">
               <div class="siegu-icon-circle siegu-icon-circle-sm mr-2">
                 <v-icon size="12" color="white">mdi-check</v-icon>
               </div>
               <span class="text-white">Save</span>
             </div>
          </v-btn>
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
                hide-details
                flat
                rounded="lg"
                class="siegu-field"
                @keyup.enter="renamePerson"
              ></v-text-field>
              <div class="mt-4">
                <v-btn block variant="flat" class="siegu-btn py-6" @click="renamePerson">
                  <div class="d-flex align-center">
                    <div class="siegu-icon-circle mr-3">
                      <v-icon size="14" color="white">mdi-pencil-outline</v-icon>
                    </div>
                    <span class="text-white">Update Name</span>
                  </div>
                </v-btn>
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
                hide-details
                flat
                rounded="lg"
                class="siegu-field"
              ></v-select>
              <div class="mt-4">
                <v-btn block variant="flat" :disabled="!mergeTargetId" @click="mergePerson" class="siegu-btn py-6">
                  <div class="d-flex align-center">
                    <div class="siegu-icon-circle mr-3">
                      <v-icon size="14" color="white">mdi-merge</v-icon>
                    </div>
                    <span class="text-white">Confirm Merge</span>
                  </div>
                </v-btn>
              </div>
            </v-window-item>
          </v-window>
        </v-card-text>
        <v-card-actions class="px-0 pt-4">
          <v-spacer></v-spacer>
          <v-btn variant="flat" class="siegu-btn px-6" @click="manageDialog = false">
             <div class="d-flex align-center">
               <div class="siegu-icon-circle siegu-icon-circle-sm mr-2">
                 <v-icon size="12" color="white">mdi-close</v-icon>
               </div>
               <span class="text-white">Close</span>
             </div>
          </v-btn>
        </v-card-actions>
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
      this.$emit("search-person", person);
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
