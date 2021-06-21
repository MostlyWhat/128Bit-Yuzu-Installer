<template>
    <div class="column has-padding">
        <div v-if="was_migrate">
          <h4 class="subtitle">You have been moved to the new, single version of {{ $root.$data.attrs.name }}.</h4>

          <p>You can find your installed applications in your start menu - if you were in the middle of something, just reattempt.</p>

          <img src="../assets/how-to-open.png" alt="Where yuzu is installed"/>
        </div>
        <div v-else-if="was_update">
            <div v-if="has_installed">
                <h4 class="subtitle">{{ $root.$data.attrs.name }} has been updated.</h4>

                <p>You can find your installed applications in your start menu.</p>
            </div>
            <div v-else>
                <h4 class="subtitle">{{ $root.$data.attrs.name }} is already up to date!</h4>

                <p>You can find your installed applications in your start menu.</p>
            </div>
        </div>
        <div v-else-if="was_install">
            <h4 class="subtitle">Thanks for installing {{ $root.$data.attrs.name }}!</h4>

            <p>You can find your installed applications in your start menu.</p>
            <br>
            <img src="../assets/how-to-open.png" alt="Where yuzu is installed"/>
        </div>
        <div v-else>
            <h4 class="subtitle">{{ $root.$data.attrs.name }} has been uninstalled.</h4>
        </div>

        <div class="field is-grouped is-right-floating is-bottom-floating">
            <p class="control">
                <a class="button is-dark is-medium" v-on:click="exit">Exit</a>
            </p>
        </div>
    </div>
</template>

<script>
export default {
  name: 'CompleteView',
  data: function () {
    return {
      was_install: !this.$route.params.uninstall,
      was_update: this.$route.params.update,
      was_migrate: this.$route.params.migrate,
      has_installed: this.$route.params.packages_installed > 0
    }
  },
  methods: {
    exit: function () {
      this.$root.exit()
    }
  }
}
</script>
