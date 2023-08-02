import Vue from 'vue'
import App from './App.vue'
import router from './router'
import axios from 'axios'
import VueAxios from 'vue-axios'
import VueI18n from 'vue-i18n'
import { stream_ajax as streamAjax } from './helpers'
import Buefy from 'buefy'
import messages from './locales/messages.json'
import 'buefy/dist/buefy.css'
import '@mdi/font/css/materialdesignicons.min.css'

// HACK: disables context menu
document.addEventListener('contextmenu', event => event.preventDefault())

Vue.config.productionTip = false
Vue.use(Buefy)
Vue.use(VueI18n)
Vue.use(VueAxios, axios)

export const i18n = new VueI18n({
  locale: 'en', // set locale
  fallbackLocale: 'en',
  messages // set locale messages
})

// Borrowed from http://tobyho.com/2012/07/27/taking-over-console-log/
function intercept (method) {
  console[method] = function () {
    const message = Array.prototype.slice.apply(arguments).join(' ')
    window.rpc.notify(
      'Log', method, message
    )
  }
}

// See if we have access to the JSON interface
let hasExternalInterface = false
try {
  window.rpc.notify('Test')
  hasExternalInterface = true
} catch (e) {
  console.warn('Running without JSON interface - unexpected behaviour may occur!')
}

// Overwrite loggers with the logging backend
if (hasExternalInterface) {
  window.onerror = function (msg, url, line) {
    window.rpc.notify(
      'Log', 'error', msg + ' @ ' + url + ':' + line
    )
  }

  const methods = ['log', 'warn', 'error']
  for (let i = 0; i < methods.length; i++) {
    intercept(methods[i])
  }
}

// Disable F5
function disableShortcuts (e) {
  switch (e.keyCode) {
    case 116: // F5
      e.preventDefault()
      break
  }
}

// Check to see if we need to enable dark mode
axios.get('/api/dark-mode').then(function (resp) {
  if (resp.data === true) {
    document.body.classList.add('has-background-black-ter')
  }
})

window.addEventListener('keydown', disableShortcuts)

axios.get('/api/attrs').then(function (resp) {
  document.getElementById('window-title').innerText =
    i18n.t('app.window_title', { name: resp.data.name })
}).catch(function (err) {
  console.error(err)
})

const app = new Vue({
  i18n: i18n,
  router: router,
  data: {
    attrs: {},
    config: {},
    install_location: '',
    username: '',
    token: '',
    jwt_token: {},
    is_authenticated: false,
    is_linked: false,
    is_subscribed: false,
    has_reward_tier: false,
    // If the option to pick an install location should be provided
    show_install_location: true,
    metadata: {
      database: [],
      install_path: '',
      preexisting_install: false
    }
  },
  render: function (caller) {
    return caller(App)
  },
  mounted: function () {
    axios.get('/api/attrs').then(function (resp) {
      app.attrs = resp.data
    }).catch(function (err) {
      console.error(err)
    })
  },
  methods: {
    exit: function () {
      axios.get('/api/exit').catch(function (msg) {
        const searchLocation = (app.metadata.install_path && app.metadata.install_path.length > 0)
          ? app.metadata.install_path
          : i18n.t('error.location_unknown')

        app.$router.replace({
          name: 'showerr',
          params: {
            msg: i18n.t('error.exit_error', {
              name: app.attrs.name,
              path: searchLocation,
              msg: msg
            })
          }
        })
      })
    },
    check_authentication: function (success, error) {
      const that = this
      const app = this.$root

      axios.post('/api/check-auth', {
        username: app.$data.username,
        token: app.$data.token
      }).then(function (resp) {
        app.$data.username = resp.data.username
        app.$data.token = resp.data.token
        that.jwt_token = resp.data.jwt_token
        that.is_authenticated = Object.keys(that.jwt_token).length !== 0 && that.jwt_token.constructor === Object
        if (that.is_authenticated) {
          // Give all permissions to vip roles
          that.is_linked = that.jwt_token.isPatreonAccountLinked
          that.is_subscribed = that.jwt_token.isPatreonSubscriptionActive
          that.has_reward_tier = that.jwt_token.releaseChannels.indexOf('early-access') > -1
        }
        if (success) {
          success()
        }
      }).catch(function () {
        if (error) {
          error()
        }
      })
    },
    stream_ajax: streamAjax
  }
}).$mount('#app')

console.log('Vue started')
