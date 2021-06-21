<template>
    <div class="column has-padding">
            <h4 class="subtitle">Downloading config...</h4>

            <br />
            <progress class="progress is-info is-medium" max="100">
                0%
            </progress>
    </div>
</template>

<script>
export default {
  name: 'DownloadConfig',
  created: function () {
    this.download_install_status()
  },
  methods: {
    download_install_status: function () {
      var that = this
      this.$root.ajax('/api/installation-status', function (e) {
        that.$root.metadata = e

        that.download_config()
      })
    },
    download_config: function () {
      var that = this
      this.$root.ajax('/api/config', function (e) {
        that.$root.config = e

        // Update the updater if needed
        if (that.$root.config.new_tool) {
          this.$router.push('/install/updater')
          return
        }

        that.$root.check_authentication(that.choose_next_state, that.choose_next_state)
      }, function (e) {
        console.error('Got error while downloading config: ' + e)

        if (that.$root.metadata.is_launcher) {
          // Just launch the target application
          that.$root.exit()
        } else {
          that.$router.replace({ name: 'showerr',
            params: { msg: 'Got error while downloading config: ' + e } })
        }
      })
    },
    choose_next_state: function () {
      var app = this.$root
      if (app.metadata.preexisting_install) {
        app.install_location = app.metadata.install_path

        // Copy over installed packages
        for (var x = 0; x < app.config.packages.length; x++) {
          app.config.packages[x].default = false
          app.config.packages[x].installed = false
        }

        for (var i = 0; i < app.metadata.database.packages.length; i++) {
          // Find this config package
          for (var x = 0; x < app.config.packages.length; x++) {
            if (app.config.packages[x].name === app.metadata.database.packages[i].name) {
              app.config.packages[x].default = true
              app.config.packages[x].installed = true
            }
          }
        }

        this.$router.replace({ name: 'migrate',
          params: { next: app.metadata.is_launcher ? '/install/regular' : '/modify' } })
      } else {
        for (var x = 0; x < app.config.packages.length; x++) {
          app.config.packages[x].installed = false
        }

        // Need to do a bit more digging to get at the
        // install location.
        this.$root.ajax('/api/default-path', function (e) {
          if (e.path != null) {
            app.install_location = e.path
          }
        })

        this.$router.replace({ name: 'migrate',
          params: { next: '/packages' } })
      }
    }
  }
}
</script>
