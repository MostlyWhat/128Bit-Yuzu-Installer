<template>
    <div class="column has-padding">
            <h4 class="subtitle">Select which packages you want to install:</h4>

            <!-- Build options -->
            <div class="tile is-ancestor">
                <div class="tile is-parent" v-for="Lpackage in $root.$data.config.packages" :key="Lpackage.name" :index="Lpackage.name">
                    <div class="tile is-child">
                        <div class="box clickable-box" v-on:click.capture.stop="Lpackage.default = !Lpackage.default">
                            <label class="checkbox">
                                <b-checkbox v-model="Lpackage.default">
                                  {{ Lpackage.name }}
                                </b-checkbox>
                                <span v-if="Lpackage.installed"><i>(installed)</i></span>
                            </label>
                            <p>
                                {{ Lpackage.description }}
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            <div class="subtitle is-6" v-if="!$root.$data.metadata.preexisting_install && advanced">Install Location</div>
            <div class="field has-addons" v-if="!$root.$data.metadata.preexisting_install && advanced">
                <div class="control is-expanded">
                    <input class="input" type="text" v-model="$root.$data.install_location"
                           placeholder="Enter a install path here">
                </div>
                <div class="control">
                    <a class="button is-dark" v-on:click="select_file">
                        Select
                    </a>
                </div>
            </div>

            <div class="is-right-floating is-bottom-floating">
                <div class="field is-grouped">
                    <p class="control">
                        <a class="button is-medium" v-if="!$root.$data.config.hide_advanced && !$root.$data.metadata.preexisting_install && !advanced"
                           v-on:click="advanced = true">Advanced...</a>
                    </p>
                    <p class="control">
                        <a class="button is-dark is-medium" v-if="!$root.$data.metadata.preexisting_install"
                           v-on:click="install">Install</a>
                    </p>
                    <p class="control">
                        <a class="button is-dark is-medium" v-if="$root.$data.metadata.preexisting_install"
                           v-on:click="install">Modify</a>
                    </p>
                </div>
            </div>

            <div class="field is-grouped is-left-floating is-bottom-floating">
                <p class="control">
                    <a class="button is-medium" v-if="$root.$data.metadata.preexisting_install"
                       v-on:click="go_back">Back</a>
                </p>
            </div>
        </div>
</template>

<script>
export default {
  name: 'SelectPackages',
  data: function () {
    return {
      advanced: false
    }
  },
  methods: {
    select_file: function () {
      window.external.invoke(JSON.stringify({
        SelectInstallDir: {
          callback_name: 'selectFileCallback'
        }
      }))
    },
    install: function () {
      this.$router.push('/install/regular')
    },
    go_back: function () {
      this.$router.go(-1)
    }
  }
}
</script>
