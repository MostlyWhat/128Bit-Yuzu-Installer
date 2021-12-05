<template>
    <div class="column has-padding">
            <div v-if="was_update">
                <div v-if="has_installed">
                    <h4 class="subtitle">{{ $t('complete.updated', {'name': $root.$data.attrs.name}) }}</h4>

                    <p>{{ $t('complete.where_to_find') }}</p>
                </div>
                <div v-else>
                    <h4 class="subtitle">{{ $t('complete.up_to_date', {'name': $root.$data.attrs.name}) }}</h4>

                    <p>{{ $t('complete.where_to_find') }}</p>
                </div>
            </div>
            <div v-else-if="was_install">
                <h4 class="subtitle">{{ $t('complete.thanks', {'name': $root.$data.attrs.name}) }}</h4>

                <p>{{ $t('complete.where_to_find') }}</p>
            </div>
            <div v-else>
                <h4 class="subtitle">{{ $t('complete.uninstalled', {'name': $root.$data.attrs.name}) }}</h4>
            </div>

            <div class="field is-grouped is-right-floating is-bottom-floating">
                <p class="control">
                    <b-button class="is-dark is-medium" v-on:click="exit">{{ $t('exit') }}</b-button>
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
