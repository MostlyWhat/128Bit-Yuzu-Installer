import Vue from 'vue'
import App from './App.vue'
import router from './router'
import { ajax, stream_ajax } from './helpers'
import Buefy from 'buefy'
import 'buefy/dist/buefy.css'

Vue.config.productionTip = false
Vue.use(Buefy)

// Borrowed from http://tobyho.com/2012/07/27/taking-over-console-log/
function intercept (method) {
  console[method] = function () {
    var message = Array.prototype.slice.apply(arguments).join(' ')
    window.external.invoke(
      JSON.stringify({
        Log: {
          kind: method,
          msg: message
        }
      })
    )
  }
}

// See if we have access to the JSON interface
var has_external_interface = false;
try {
  window.external.invoke(JSON.stringify({
    Test: {}
  }))
  has_external_interface = true;
} catch (e) {
  console.warn("Running without JSON interface - unexpected behaviour may occur!")
}

// Overwrite loggers with the logging backend
if (has_external_interface) {
  window.onerror = function (msg, url, line) {
    window.external.invoke(
      JSON.stringify({
        Log: {
          kind: 'error',
          msg: msg + ' @ ' + url + ':' + line
        }
      })
    )
  }

  var methods = ['log', 'warn', 'error']
  for (var i = 0; i < methods.length; i++) {
    intercept(methods[i])
  }
}

// Disable F5
function disable_shortcuts (e) {
  switch (e.keyCode) {
    case 116: // F5
      e.preventDefault()
      break
  }
}

// Check to see if we need to enable dark mode
ajax('/api/dark-mode', function (enable) {
  if (enable) {
    document.body.classList.add('has-background-black-ter')
  }
})

window.addEventListener('keydown', disable_shortcuts)

document.getElementById('window-title').innerText =
  base_attributes.name + ' Installer'

function selectFileCallback (name) {
  app.install_location = name
}

var app = new Vue({
  router: router,
  data: {
    attrs: base_attributes,
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
  methods: {
    exit: function () {
      ajax(
        '/api/exit',
        function () {},
        function (msg) {
          var search_location = app.metadata.install_path.length > 0 ? app.metadata.install_path :
            "the location where this installer is";

          app.$router.replace({ name: 'showerr', params: { msg: msg +
                '\n\nPlease upload the log file (in ' + search_location + ') to ' +
                'the ' + app.attrs.name + ' team'
          }});
        }
      )
    },
    check_authentication: function (success, error) {
      var that = this;
      var app = this.$root;

      app.ajax('/api/check-auth', function (auth) {
        app.$data.username = auth.username;
        app.$data.token = auth.token;
        that.jwt_token = auth.jwt_token;
        that.is_authenticated = Object.keys(that.jwt_token).length !== 0 && that.jwt_token.constructor === Object;
        if (that.is_authenticated) {
          // Give all permissions to vip roles
          that.is_linked = that.jwt_token.isPatreonAccountLinked;
          that.is_subscribed = that.jwt_token.isPatreonSubscriptionActive;
          that.has_reward_tier = that.jwt_token.releaseChannels.indexOf("early-access") > -1;
        }
        if (success) {
          success();
        }
      }, function (e) {
        if (error) {
          error();
        }
      }, {
        "username": app.$data.username,
        "token": app.$data.token
      })
    },

    ajax: ajax,
    stream_ajax: stream_ajax
  }
}).$mount('#app')

console.log("Vue started")
