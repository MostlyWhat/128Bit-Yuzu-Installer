<template>
    <div class="column has-padding">
            <h4 class="subtitle">Performing migrations...</h4>
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
  name: 'MigrateView',
  data: function () {
    return {
      progress: 0.0,
      progress_message: 'Please wait...',
      failed_with_error: false,
      packages_installed: 0,
      next_stop: this.$route.params.next
    }
  },
  created: function () {
    // See if we need to migrate yuzu to mainline
    var need_migrate = false;
    for (var package_id in this.$root.metadata.database.packages) {
      var name = this.$root.metadata.database.packages[package_id].name
      if ((name.indexOf("Nightly") !== -1 || name.indexOf("Canary") !== -1)) {
        console.log("Migration needed (found \"" + name + "\", move to mainline)")

        // Migration step: deactivate this package
        if ( this.$root.config.packages[package_id] !== undefined) {
          this.$root.config.packages[package_id].default = false;
        }

        // Migration step: enable mainline
        for (var sub_package_id in this.$root.config.packages) {
          var name = this.$root.config.packages[sub_package_id].name
          if (name === "yuzu") {
            this.$root.config.packages[sub_package_id].default = true;
            break;
          }
        }

        need_migrate = true;
      }
    }

    console.log("Next stop: " + JSON.stringify(this.next_stop));
    if (need_migrate) {
      this.next_stop = "/complete/false/true/true/[]"
      this.install()
    } else {
      this.$router.replace(this.next_stop)
    }
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

      console.log("Install results: " + JSON.stringify(results));

      results['path'] = app.install_location

      var targetUrl = '/api/start-install'

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
        if (!that.failed_with_error) {
          that.$router.replace(that.next_stop)
        }
      }, undefined, results)
    }
  }
}
</script>
