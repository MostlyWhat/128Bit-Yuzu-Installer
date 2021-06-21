<template>
    <div class="column has-padding">
            <h4 class="subtitle" v-if="$root.$data.metadata.is_launcher || is_update">Checking for updates...</h4>
            <h4 class="subtitle" v-else-if="is_uninstall">Uninstalling...</h4>
            <h4 class="subtitle" v-else-if="is_updater_update">Downloading self-update...</h4>
            <h4 class="subtitle" v-else>Installing...</h4>
            <div v-html="$root.$data.config.installing_message"></div>
            <br />

            <div v-html="progress_message"></div>
            <progress class="progress is-info is-medium" v-bind:value="progress" max="100">
                {{ progress }}%
            </progress>
    </div>
</template>

<script>
export default {
  name: 'InstallPackages',
  data: function () {
    return {
      progress: 0.0,
      progress_message: 'Please wait...',
      is_uninstall: false,
      is_updater_update: false,
      is_update: false,
      failed_with_error: false,
      packages_installed: 0
    }
  },
  created: function () {
    this.is_uninstall = this.$route.params.kind === 'uninstall'
    this.is_updater_update = this.$route.params.kind === 'updater'
    this.is_update = this.$route.params.kind === 'update'
    console.log('Installer kind: ' + this.$route.params.kind)
    this.install()
  },
  methods: {
    install: function () {
      var that = this
      var app = this.$root

      var results = {}

      for (var package_index = 0; package_index < app.config.packages.length; package_index++) {
        var current_package = app.config.packages[package_index]
        if (current_package.default != null) {
          results[current_package.name] = current_package.default
        }
      }

      results['path'] = app.install_location

      var targetUrl = '/api/start-install'
      if (this.is_uninstall) {
        targetUrl = '/api/uninstall'
      }
      if (this.is_updater_update) {
        targetUrl = '/api/update-updater'
      }

      this.$root.stream_ajax(targetUrl, function (line) {
        // On progress line received from server

        if (line.hasOwnProperty('Status')) {
          that.progress_message = line.Status[0]
          that.progress = line.Status[1] * 100
        }

        if (line.hasOwnProperty('PackageInstalled')) {
          that.packages_installed += 1
        }

        if (line.hasOwnProperty('Error')) {
          that.failed_with_error = true
          that.$router.replace({ name: 'showerr', params: { msg: line.Error } })
        }
      }, function (e) {
        // On request completed

        if (that.is_updater_update) {
          // Continue with what we were doing
          if (app.metadata.is_launcher) {
            that.$router.replace('/install/regular')
          } else {
            if (app.metadata.preexisting_install) {
              that.$router.replace('/modify')
            } else {
              that.$router.replace('/packages')
            }
          }
        } else {
          if (app.metadata.is_launcher) {
            app.exit()
          } else if (!that.failed_with_error) {
            if (that.is_uninstall) {
              that.$router.replace({ name: 'complete',
                params: {
                  uninstall: true,
                  update: that.is_update,
                  installed: that.packages_installed
                } })
            } else {
              that.$router.replace({ name: 'complete',
                params: {
                  uninstall: false,
                  update: that.is_update,
                  installed: that.packages_installed
                } })
            }
          }
        }
      }, undefined, results)
    }
  }
}
</script>
