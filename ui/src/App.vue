<template>
  <div id="app" class="is-max-height">
        <section class="section is-max-height">
            <div class="container is-max-height">
                <div class="columns is-max-height">
                    <div class="column is-one-third has-padding" v-if="!$root.$data.metadata.is_launcher">
                        <img src="./assets/light_mode_installer_logo.png" id="applicationIcon" alt="Application icon" />
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
                        <b-dropdown hoverable @change="selectLocale" aria-role="list" scrollable>
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
    const languages = window.navigator.languages
    if (languages) {
      // standard-compliant browsers
      for (let index = 0; index < languages.length; index++) {
        const lang = languages[index]
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

#applicationIcon {
    width:0px; height: 0px;
    padding: 50px 60% 0px 0px;
    background: url("./assets/light_mode_installer_logo.png") left top no-repeat;
    background-size: contain;
}

body.has-background-black-ter #applicationIcon {
    background: url("./assets/dark_mode_installer_logo.png") left top no-repeat;
    background-size: contain;
}

.package-icon {
    width: 3rem;
    height: 3rem;
    float: left;
    padding-right: 10px;
    padding-top: 10px;
}

.package-description {
    overflow: hidden;
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
    position: relative;
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

.tile.box.clickable-box {
    color: #4a4a4a;
}

/* Dark mode */
body.has-background-black-ter .subtitle, body.has-background-black-ter .column > div {
    color: hsl(0, 0%, 96%);
}

.ribbon {
    position: absolute;
    right: -5px; top: -5px;
    z-index: 1;
    overflow: hidden;
    width: 75px; height: 75px;
    text-align: right;
}
.ribbon span {
    font-size: 10px;
    font-weight: bold;
    color: #FFF;
    text-transform: uppercase;
    text-align: center;
    line-height: 20px;
    transform: rotate(45deg);
    -webkit-transform: rotate(45deg);
    width: 100px;
    display: block;
    background: #79A70A;
    background: linear-gradient(#FF3C28 0%, #FF3C28 100%);
    box-shadow: 0 3px 10px -5px rgba(0, 0, 0, 1);
    position: absolute;
    top: 19px; right: -21px;
}
.ribbon span::before {
    content: "";
    position: absolute; left: 0px; top: 100%;
    z-index: -1;
    border-left: 3px solid #FF3C28;
    border-right: 3px solid transparent;
    border-bottom: 3px solid transparent;
    border-top: 3px solid #FF3C28;
}
.ribbon span::after {
    content: "";
    position: absolute; right: 0px; top: 100%;
    z-index: -1;
    border-left: 3px solid transparent;
    border-right: 3px solid #FF3C28;
    border-bottom: 3px solid transparent;
    border-top: 3px solid #FF3C28;
}
a:hover {
    text-decoration: underline;
}
</style>
