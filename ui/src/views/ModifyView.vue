<template>
    <div class="column has-padding">
            <h4 class="subtitle">Choose an option:</h4>

            <a class="button is-dark is-medium" v-on:click="update">
                Update
            </a>
            <br />
            <br />

            <a class="button is-dark is-medium" v-on:click="modify_packages">
                Modify
            </a>
            <br />
            <br />

            <a class="button is-dark is-medium" v-on:click="prepare_uninstall">
                Uninstall
            </a>

            <div class="modal is-active" v-if="show_uninstall">
                <div class="modal-background"></div>
                <div class="modal-card">
                    <header class="modal-card-head">
                        <p class="modal-card-title">Are you sure you want to uninstall {{ $root.$data.attrs.name }}?</p>
                    </header>
                    <footer class="modal-card-foot">
                        <button class="button is-danger" v-on:click="uninstall">Yes</button>
                        <button class="button" v-on:click="cancel_uninstall">No</button>
                    </footer>
                </div>
            </div>
    </div>
</template>

<script>
export default {
  name: 'ModifyView',
  data: function () {
    return {
      show_uninstall: false
    }
  },
  methods: {
    update: function () {
      this.$router.push('/install/update')
    },
    modify_packages: function () {
      this.$router.push('/packages')
    },
    prepare_uninstall: function () {
      this.show_uninstall = true
    },
    cancel_uninstall: function () {
      this.show_uninstall = false
    },
    uninstall: function () {
      this.$router.push('/install/uninstall')
    }
  }
}
</script>
