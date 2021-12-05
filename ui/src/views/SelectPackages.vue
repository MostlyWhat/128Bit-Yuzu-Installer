<template>
    <div class="column has-padding">
            <h4 class="subtitle" v-if="!repair">{{ $t('select_packages.title') }}</h4>
            <h4 class="subtitle" v-if="repair">{{ $t('select_packages.title_repair') }}</h4>

            <!-- Build options -->
            <div class="tile is-ancestor">
                <div class="tile is-parent" v-for="Lpackage in $root.$data.config.packages" :key="Lpackage.name" :index="Lpackage.name">
                    <div class="tile is-child">
                        <div class="box clickable-box" v-on:click.capture.stop="Lpackage.default = !Lpackage.default">
                            <label class="checkbox">
                                <b-checkbox v-model="Lpackage.default">
                                  {{ Lpackage.name }}
                                </b-checkbox>
                                <span v-if="Lpackage.installed"><i>{{ $t('select_packages.installed') }}</i></span>
                            </label>
                            <p>
                                {{ Lpackage.description }}
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            <div class="subtitle is-6" v-if="!$root.$data.metadata.preexisting_install && advanced">{{ $t('select_packages.location') }}</div>
            <div class="field has-addons" v-if="!$root.$data.metadata.preexisting_install && advanced">
                <div class="control is-expanded">
                    <input class="input" type="text" v-model="$root.$data.install_location"
                           :placeholder="$t('select_packages.location_placeholder')">
                </div>
                <div class="control">
                    <b-button class="is-dark" v-on:click="select_file">
                        {{ $t('select_packages.select') }}
                    </b-button>
                </div>
            </div>

            <div class="is-right-floating is-bottom-floating">
                <div class="field is-grouped">
                    <p class="control">
                        <b-button class="is-medium" v-if="!$root.$data.config.hide_advanced && !$root.$data.metadata.preexisting_install && !advanced"
                           v-on:click="advanced = true">{{ $t('select_packages.advanced') }}</b-button>
                    </p>
                    <p class="control">
                        <b-button class="is-dark is-medium" v-if="!$root.$data.metadata.preexisting_install"
                           v-on:click="install">{{ $t('select_packages.install') }}</b-button>
                    </p>
                    <p class="control">
                        <a class="button is-dark is-medium" v-if="$root.$data.metadata.preexisting_install"
                           v-on:click="install">{{ repair ? $t('select_packages.repair') : $t('select_packages.modify') }}</a>
                    </p>
                </div>
            </div>

            <div class="field is-grouped is-left-floating is-bottom-floating">
                <p class="control">
                    <b-button class="is-medium" v-if="$root.$data.metadata.preexisting_install"
                       v-on:click="go_back">{{ $t('back') }}</b-button>
                </p>
            </div>
        </div>
</template>

<script>
export default {
  name: 'SelectPackages',
  data: function () {
    return {
      advanced: false,
      repair: false
    }
  },
  mounted: function () {
    this.repair = this.$route.params.repair
  },
  methods: {
    select_file: function () {
      const that = this
      window.rpc.call('SelectInstallDir').then(function (name) {
        if (name) {
          that.$root.$data.install_location = name
        }
      })
    },
    show_overwrite_dialog: function (confirmCallback) {
      this.$buefy.dialog.confirm({
        title: this.$t('select_packages.overwriting'),
        message: this.$t('select_packages.overwriting_warning', { path: this.$root.$data.install_location }),
        confirmText: this.$t('continue'),
        cancelText: this.$t('cancel'),
        type: 'is-danger',
        hasIcon: true,
        onConfirm: confirmCallback
      })
    },
    show_nothing_picked_dialog: function () {
      this.$buefy.dialog.alert({
        title: this.$t('select_packages.nothing_picked'),
        message: this.$t('select_packages.nothing_picked_warning', { path: this.$root.$data.install_location }),
        confirmText: this.$t('cancel'),
        type: 'is-danger',
        hasIcon: true
      })
    },
    install: function () {
      if (!this.$root.config.packages.some(function (x) { return x.default })) {
        this.show_nothing_picked_dialog()
        return
      }
      // maintenance + repair
      if (this.repair) {
        this.$router.push('/install/repair')
        return
      }
      // maintenance + modify
      if (this.$root.$data.metadata.preexisting_install) {
        this.$router.push('/install/regular')
        return
      }
      const my = this
      this.$http.post('/api/verify-path', `path=${this.$root.$data.install_location}`).then(function (resp) {
        const data = resp.data || {}
        if (!data.exists) {
          my.$router.push('/install/regular')
        } else {
          my.show_overwrite_dialog(function () {
            my.$router.push('/install/repair')
          })
        }
      })
    },
    go_back: function () {
      this.$router.go(-1)
    }
  }
}
</script>
