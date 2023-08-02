<template>
    <div class="column has-padding">
            <h4 class="subtitle" v-if="$root.$data.metadata.is_launcher || is_update">{{ $t('install_packages.check_for_update') }}</h4>
            <h4 class="subtitle" v-else-if="is_uninstall">{{ $t('install_packages.uninstall') }}</h4>
            <h4 class="subtitle" v-else-if="is_updater_update">{{ $t('install_packages.self_update') }}</h4>
            <h4 class="subtitle" v-else>{{ $t('install_packages.install') }}</h4>
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
      progress_message: this.$i18n.t('install_packages.please_wait'),
      is_uninstall: false,
      is_updater_update: false,
      is_update: false,
      is_repair: false,
      install_desktop_shortcut: false,
      failed_with_error: false,
      authorization_required: false,
      packages_installed: 0
    }
  },
  created: function () {
    this.is_uninstall = this.$route.params.kind === 'uninstall'
    this.is_updater_update = this.$route.params.kind === 'updater'
    this.is_update = this.$route.params.kind === 'update'
    this.install_desktop_shortcut = this.$route.params.desktop_shortcut === 'true'
    this.is_repair = this.$route.params.kind === 'repair'
    console.log('Installer kind: ' + this.$route.params.kind)
    console.log('Installing desktop shortcut: ' + this.$route.params.desktop_shortcut)
    this.install()
  },
  methods: {
    install: function () {
      const that = this
      const app = this.$root

      const results = {}

      for (let package_index = 0; package_index < app.config.packages.length; package_index++) {
        const current_package = app.config.packages[package_index]
        if (current_package.default != null) {
          results[current_package.name] = current_package.default
        }
      }

      results.path = app.install_location
      results.installDesktopShortcut = that.install_desktop_shortcut

      if (this.is_repair) {
        results.mode = 'force'
      }

      let targetUrl = '/api/start-install'
      if (this.is_uninstall) {
        targetUrl = '/api/uninstall'
      }
      if (this.is_updater_update) {
        targetUrl = '/api/update-updater'
      }

      this.$root.stream_ajax(targetUrl, function (line) {
        // On progress line received from server

        if (line.Status) {
          that.progress_message = line.Status[0]
          that.progress = line.Status[1] * 100
        }

        if (line.PackageInstalled) {
          that.packages_installed += 1
        }

        if (line.AuthorizationRequired) {
          that.authorization_required = true
        }

        if (line.Error) {
          that.failed_with_error = true
          that.$router.replace({ name: 'showerr', params: { msg: line.Error } })
        }
      }, function (e) {
        // On request completed

        if (that.is_updater_update) {
          // Continue with what we were doing
          if (app.metadata.is_launcher) {
            that.$router.replace('/install/regular/' + that.install_desktop_shortcut.toString())
          } else {
            if (app.metadata.preexisting_install) {
              that.$router.replace('/modify')
            } else {
              that.$router.replace('/packages')
            }
          }
        } else {
          if (that.authorization_required) {
            that.$router.push('/reauthenticate')
          } else if (app.metadata.is_launcher) {
            app.exit()
          } else if (!that.failed_with_error) {
            if (that.is_uninstall) {
              that.$router.replace({
                name: 'complete',
                params: {
                  uninstall: true,
                  update: that.is_update,
                  installed: that.packages_installed
                }
              })
            } else {
              that.$router.replace({
                name: 'complete',
                params: {
                  uninstall: false,
                  update: that.is_update,
                  installed: that.packages_installed
                }
              })
            }
          }
        }
      }, undefined, results)
    }
  }
}
</script>
