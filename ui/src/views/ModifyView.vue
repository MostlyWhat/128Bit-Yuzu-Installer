<template>
    <div class="column has-padding">
            <h4 class="subtitle">{{ $t('modify.title') }}</h4>

            <b-button icon-left="update" type="is-dark-green" size="is-medium" @click="update">
                {{ $t('modify.update') }}
            </b-button>
            <br />
            <br />

            <b-button icon-left="pencil" type="is-info" size="is-medium" @click="modify_packages">
                {{ $t('modify.modify') }}
            </b-button>
            <br />
            <br />

            <b-button icon-left="wrench" type="is-info" size="is-medium" @click="repair_packages">
                {{ $t('modify.repair') }}
            </b-button>
            <br />
            <br />

            <b-button icon-left="delete" type="is-danger" size="is-medium" @click="prepare_uninstall">
                {{ $t('modify.uninstall') }}
            </b-button>
            <br />
            <br />

            <b-button icon-left="file-find" type="is-link" size="is-medium" @click="view_files">
                {{ $t('modify.view_local_files') }}
            </b-button>
    </div>
</template>

<script>
export default {
  name: 'ModifyView',
  data: function () {
    return {}
  },
  methods: {
    update: function () {
      this.$router.push('/install/update')
    },
    modify_packages: function () {
      this.$router.push('/packages')
    },
    repair_packages: function () {
      this.$router.push({ name: 'packages', params: { repair: true } })
    },
    prepare_uninstall: function () {
      this.$buefy.dialog.confirm({
        title: this.$t('modify.uninstall'),
        message: this.$t('modify.prompt', { name: this.$root.$data.attrs.name }),
        cancelText: this.$t('cancel'),
        confirmText: this.$t('modify.prompt_confirm', { name: this.$root.$data.attrs.name }),
        type: 'is-danger',
        hasIcon: true,
        onConfirm: this.uninstall
      })
    },
    uninstall: function () {
      this.$router.push('/install/uninstall')
    },
    view_files: function () {
      this.$http.get('/api/view-local-folder')
    }
  }
}
</script>

<style>
span {
  cursor: unset !important;
}
.button.is-dark-green {
  background-color: #00B245;
  border-color: transparent;
  color: #fff;
}
.button.is-dark-green:hover, .button.is-dark-green.is-hovered, .button.is-dark-green:focus {
  background-color: #00a53f;
  border-color: transparent;
  color: #fff;
}
</style>
