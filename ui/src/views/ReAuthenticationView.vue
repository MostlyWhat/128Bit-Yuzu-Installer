<template>
  <div class="column has-padding">
    <b-message type="is-info">
      <h4 class="subtitle">Update Available!</h4>
      There is a new Early Access update available, but you are no longer in the early access reward tier.
      We'd love to have you check out this update, so <a v-on:click="go_authenticate">click here to refresh your access!</a>
    </b-message>
    <br>
    Alternatively, you can install the regular version of yuzu by clicking <strong>Back</strong> below.
    <br>
    Click <strong>Launch Old Version</strong> to continue using the old version of yuzu Early Access.

    <div class="is-left-floating is-bottom-floating">
      <p class="control">
        <a class="button is-medium" v-on:click="go_packages">Back</a>
      </p>
    </div>

    <div class="is-right-floating is-bottom-floating">
      <p class="control">
        <a class="button is-dark is-medium" v-on:click="launch_old_version">Launch Old Version</a>
      </p>
    </div>
  </div>

</template>

<script>
  export default {
    name: "ReAuthenticationView",
    methods: {
      go_authenticate: function() {
        this.$router.replace('/authentication')
      },
      launch_old_version: function () {
        this.$root.exit()
      },
      go_packages: function () {
        this.$router.push('/packages')
      }
    }
  }
</script>

<style scoped>

</style>