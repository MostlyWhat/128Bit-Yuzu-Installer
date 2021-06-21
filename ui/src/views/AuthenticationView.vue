<template>
  <div class="column has-padding">
    <b-message type="is-info" :active.sync="browser_opened">
      Page opened! Check your default browser for the page, and follow the instructions there to link your patreon account.
      When you are done, enter the token below.
    </b-message>
    <b-message type="is-info" :active.sync="show_header">
      The <strong>Early Access</strong> release channel lets you try out the latest experimental features and fixes, before they are merged into yuzu. This channel includes all regular yuzu daily updates, plus these exclusive features.

      To be an Early Access member, you must be a Patreon Early Access Subscriber.
    </b-message>
    <div>
    If you are a subscriber, <a v-on:click="launch_browser('https://profile.yuzu-emu.org/')">click here to link your yuzu-emu.org account</a>
    <br>
    If you are not already a subscriber, <a v-on:click="launch_browser('https://www.patreon.com/join/yuzuteam/checkout?rid=2822069')">click here to become one</a>
    </div>
    <br>

    <div class="control">
      <label for="token">Token</label>
      <input class="input" type="text" v-model="combined_token" placeholder="Token" id="token">
    </div>

    <br>

    <b-message type="is-danger" :active.sync="invalid_token">
      Login failed!
      Double check that your token is correct and try again
    </b-message>

    <b-message type="is-danger" :active.sync="invalid_login">
      Login failed!
      Double check that your token is correct and try again
    </b-message>

    <b-message type="is-danger" :active.sync="unlinked_patreon">
      Your credentials are valid, but you still need to link your patreon!
      If this is an error, then <a v-on:click="launch_browser('https://profile.yuzu-emu.org/')">click here to link your yuzu-emu.org account</a>
    </b-message>

    <b-message type="is-danger" :active.sync="no_subscription">
      Your patreon is linked, but you are not a current subscriber!
      <a v-on:click="launch_browser('https://www.patreon.com/join/yuzuteam/checkout?rid=2822069')">Log into your patreon account</a> and support the project!
    </b-message>

    <b-message type="is-danger" :active.sync="tier_not_selected">
      Your patreon is linked, and you are supporting the project, but you must first join the Early Access reward tier!
      <a v-on:click="launch_browser('https://www.patreon.com/join/yuzuteam/checkout?rid=2822069')">Log into your patreon account</a> and choose to back the Early Access reward tier.
    </b-message>

    <div class="is-left-floating is-bottom-floating">
      <p class="control">
        <a class="button is-medium" v-on:click="go_back">Back</a>
      </p>
    </div>

    <div class="is-right-floating is-bottom-floating">
      <p class="control">
        <a class="button is-dark is-medium" v-on:click="verify_token">Verify Token</a>
      </p>
    </div>
  </div>
</template>

<script>
export default {
  name: 'AuthenticationView',
  created: function() {
    // If they are already authenticated when this page is loaded,
    // then we can asssume they are "clicking here for more details" and should show the appropriate error message
    if (this.$root.is_authenticated) {
      this.verification_opened = true;
    }
  },
  data: function() {
    return {
      browser_opened: false,
      verification_opened: false,
      invalid_token: false,
    }
  },
  computed: {
    show_header: function() {
      return !this.browser_opened && !this.verification_opened && !this.invalid_token;
    },
    invalid_login: function() {
      return this.verification_opened && !this.$root.is_authenticated;
    },
    unlinked_patreon: function() {
      return this.verification_opened && this.$root.is_authenticated && !this.$root.is_linked;
    },
    no_subscription: function() {
      return this.verification_opened && this.$root.is_linked && !this.$root.is_subscribed;
    },
    tier_not_selected: function() {
      return this.verification_opened && this.$root.is_linked && this.$root.is_subscribed && !this.$root.has_reward_tier;
    },
    combined_token: {
      // getter
      get: function () {
        if (this.$root.$data.username && this.$root.$data.token) {
          return btoa(this.$root.$data.username + ":" + this.$root.$data.token)
        }
        return "";
      },
      // setter
      set: function (newValue) {
        try {
          var split = atob(newValue).split(':')
          this.$root.$data.username = split[0];
          this.$root.$data.token = split[1];
          this.invalid_token = false;
        } catch (e) {
          this.invalid_token = true;
        }
      }
    }
  },
  methods: {
    go_back: function () {
      this.$router.go(-1)
    },
    launch_browser: function(url) {
      const that = this;
      let app = this.$root;
      app.ajax('/api/open-browser', function (e) {
        // only open the browser opened message if there isn't an error message currently
        if (!that.verification_opened) {
          that.browser_opened = true;
        }
      }, function (e) {}, {
        "url": url,
      });
    },
    verify_token: function() {
      this.browser_opened = false;
      this.$root.check_authentication(this.success, this.error);
    },
    success: function() {
      // if they are eligible, go back to the select package page
      if (this.$root.has_reward_tier) {
        this.$router.go(-1);
        return;
      }
      // They aren't currently eligible for the release, so display the error message
      this.verification_opened = true;
    },
    error: function() {
      this.verification_opened = true;
    }
  }
}
</script>
