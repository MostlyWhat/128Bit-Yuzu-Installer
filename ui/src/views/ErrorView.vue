<template>
  <div class="column" v-bind:class="{ 'has-padding': !$root.$data.metadata.is_launcher }">
    <b-message :title="$t('error.title')" type="is-danger" :closable="false">
      <div id="error_msg" v-html="msg"></div>
    </b-message>
    <div class="field is-grouped is-right-floating" v-bind:class="{ 'is-bottom-floating': !$root.$data.metadata.is_launcher, 'is-top-floating': $root.$data.metadata.is_launcher }">
      <p class="control">
        <b-button class="is-primary is-medium" v-if="remaining && !$root.$data.metadata.is_launcher" v-on:click="go_back">{{ $t('back') }}</b-button>
        <b-button class="is-primary is-medium" v-if="$root.$data.metadata.is_launcher" v-on:click="exit">{{ $t('exit') }}</b-button>
      </p>
    </div>
  </div>
</template>

<style>
.pre-wrap {
  /* https://css-tricks.com/snippets/css/make-pre-text-wrap/ */
  white-space: pre-wrap; /* css-3 */
  white-space: -moz-pre-wrap; /* Mozilla, since 1999 */
  white-space: -pre-wrap; /* Opera 4-6 */
  white-space: -o-pre-wrap; /* Opera 7 */
  word-wrap: break-word; /* Internet Explorer 5.5+ */
}

#error_msg {
    -webkit-user-select: text;
    -moz-user-select: text;
    -ms-user-select: text;
    user-select: text;
    cursor: text;
}
</style>

<script>
export default {
  name: 'ErrorView',
  data: function () {
    return {
      // https://stackoverflow.com/questions/6234773/can-i-escape-html-special-chars-in-javascript
      msg: this.$route.params.msg
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;')
        .replace(/\n/g, '<br />'),
      remaining: window.history.length > 1
    }
  },
  methods: {
    go_back: function () {
      this.$router.go(-1)
    },
    exit: function () {
      this.$root.exit()
    }
  }
}
</script>
