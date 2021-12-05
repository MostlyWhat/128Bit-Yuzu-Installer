<template>
  <div id="app" class="is-max-height">
        <section class="section is-max-height">
            <div class="container is-max-height">
                <div class="columns is-max-height">
                    <div class="column is-one-third has-padding" v-if="!$root.$data.metadata.is_launcher">
                        <img src="./assets/logo.png" width="60%" alt="Application icon" />
                        <br />
                        <br />

                        <h2 class="subtitle" v-if="!$root.$data.metadata.preexisting_install">
                            {{ $t('app.installer_title', {'name': $root.$data.attrs.name}) }}
                        </h2>
                        <h2 class="subtitle" v-if="!$root.$data.metadata.preexisting_install">
                            {{ $t('app.installer_subtitle') }}
                        </h2>

                        <h2 class="subtitle" v-if="$root.$data.metadata.preexisting_install">
                            {{ $t('app.maintenance_title', {'name': $root.$data.attrs.name}) }}
                        </h2>
                        <b-dropdown hoverable @change="selectLocale" aria-role="list">
                            <button class="button" slot="trigger">
                                <span>{{ $t('locale') }}</span>
                                <b-icon icon="menu-down"></b-icon>
                            </button>

                            <b-dropdown-item v-for="(locale, index) in this.$i18n.messages" v-bind:key="index" :value="index" aria-role="listitem">{{locale.locale}}</b-dropdown-item>
                        </b-dropdown>
                    </div>

                    <router-view />
                </div>
            </div>
        </section>
    </div>
</template>

<script>
export default {
  mounted: function () {
    // detect languages
    var languages = window.navigator.languages
    if (languages) {
      // standard-compliant browsers
      for (var index = 0; index < languages.length; index++) {
        var lang = languages[index]
        // Find the most preferred language that we support
        if (Object.prototype.hasOwnProperty.call(this.$i18n.messages, lang)) {
          this.$i18n.locale = lang
          return
        }
      }
    }
    // IE9+ support
    this.$i18n.locale = window.navigator.browserLanguage
  },
  methods: {
    selectLocale: function (locale) {
      this.$i18n.locale = locale
    }
  }
}
</script>

<style>
/* roboto-regular - latin */
@font-face {
    font-family: 'Roboto';
    font-style: normal;
    font-weight: 400;
    src: url('./assets/fonts/roboto-v18-latin-regular.eot'); /* IE9 Compat Modes */
    src: local('Roboto'), local('Roboto-Regular'),
    url('./assets/fonts/roboto-v18-latin-regular.woff2') format('woff2'), /* Super Modern Browsers */
    url('./assets/fonts/roboto-v18-latin-regular.woff') format('woff');
}

html, body {
    overflow: hidden;
    height: 100%;
}

body, div, span, h1, h2, h3, h4, h5, h6 {
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
}

pre {
    -webkit-user-select: text;
    -moz-user-select: text;
    -ms-user-select: text;
    user-select: text;
    cursor: text;
}

.tile.is-child > .box {
    height: 100%;
}

.has-padding {
    padding: 2rem;
    position: relative;
}

.clickable-box {
    cursor: pointer;
}

.clickable-box label {
    pointer-events: none;
}

.is-max-height {
    height: 100%;
}

.is-bottom-floating {
    position: absolute;
    bottom: 0;
}

.is-top-floating {
    position: absolute;
    top: 0;
}

.is-right-floating {
    position: absolute;
    right: 0;
}

.has-padding .is-right-floating {
    right: 1rem;
}

.is-left-floating {
    position: absolute;
    left: 0;
}

.has-padding .is-left-floating {
    left: 1rem;
}

.fullscreen {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    width: 100%;
    height: 100%;
    z-index: 9999;
    padding: 20px;
    background: #fff;
}

/* Dark mode */
body.has-background-black-ter .subtitle, body.has-background-black-ter .column > div {
    color: hsl(0, 0%, 96%);
}
</style>
