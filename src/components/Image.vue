<template>
  <div>
    <img :src="'data:image/jpeg;base64,' + path.encoded" />
  </div>
</template>

<script>
import { invoke } from "@tauri-apps/api/core";
export default {
  name: "Image",
  props: ["path"],
  created() {
    console.log("Loading image from: ", this.path.location);
    if (window.localStorage.getItem(this.path.location)) {
      this.path.encoded = window.localStorage.getItem(this.path.location);
    } else {
      invoke("get_thumbnail", { path: this.path.location }).then((base64) => {
        console.log("Done loading image from: ", this.path.location);
        this.path.encoded = base64;
        window.localStorage.setItem(this.path.location, base64);
      });
    }
  },
};
</script>
<style>
img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center;
}
</style>
