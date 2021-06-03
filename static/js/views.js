const DownloadConfig = {
    template: `
        <div class="column has-padding">
            <h4 class="subtitle">Downloading config...</h4>

            <br />
            <progress class="progress is-info is-medium" value="0" max="100">
                0%
            </progress>
        </div>
    `,
    created: function() {
        this.download_install_status();
    },
    methods: {
        download_install_status: function() {
            var that = this; // IE workaround

            ajax("/api/installation-status", function(e) {
                app.metadata = e;

                that.download_config();
            });
        },
        download_config: function() {
            var that = this; // IE workaround

            ajax("/api/config", function(e) {
                app.config = e;

                that.choose_next_state();

            }, function(e) {
                console.error("Got error while downloading config: "
                    + e);

                if (app.metadata.is_launcher) {
                    // Just launch the target application
                    app.exit();
                } else {
                    router.replace({name: 'showerr', params: {msg: "Got error while downloading config: "
                                + e}});
                }
            });
        },
        choose_next_state: function() {
            // Update the updater if needed
            if (app.config.new_tool) {
                router.push("/install/updater");
                return;
            }

            if (app.metadata.preexisting_install) {
                app.install_location = app.metadata.install_path;

                // Copy over installed packages
                for (var x = 0; x < app.config.packages.length; x++) {
                    app.config.packages[x].default = false;
                    app.config.packages[x].installed = false;
                }

                for (var i = 0; i < app.metadata.database.packages.length; i++) {
                    // Find this config package
                    for (var x = 0; x < app.config.packages.length; x++) {
                        if (app.config.packages[x].name === app.metadata.database.packages[i].name) {
                            app.config.packages[x].default = true;
                            app.config.packages[x].installed = true;
                        }
                    }
                }

                if (app.metadata.is_launcher) {
                    router.replace("/install/regular");
                } else {
                    router.replace("/modify");
                }
            } else {
                for (var x = 0; x < app.config.packages.length; x++) {
                    app.config.packages[x].installed = false;
                }

                // Need to do a bit more digging to get at the
                // install location.
                ajax("/api/default-path", function(e) {
                    if (e.path != null) {
                        app.install_location = e.path;
                    }
                });

                router.replace("/packages");
            }
        }
    }
};

const SelectPackages = {
    template: `
        <div class="column has-padding">
            <h4 class="subtitle">Select which packages you want to install:</h4>

            <!-- Build options -->
            <div class="tile is-ancestor">
                <div class="tile is-parent" v-for="package in $root.$data.config.packages" :index="package.name">
                    <div class="tile is-child">
                        <div class="box clickable-box" v-on:click.capture.stop="package.default = !package.default">
                            <label class="checkbox">
                                <input type="checkbox" v-model="package.default" />
                                {{ package.name }}
                                <span v-if="package.installed"><i>(installed)</i></span>
                            </label>
                            <p>
                                {{ package.description }}
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            <div class="subtitle is-6" v-if="!$root.$data.metadata.preexisting_install && advanced">Install Location</div>
            <div class="field has-addons" v-if="!$root.$data.metadata.preexisting_install && advanced">
                <div class="control is-expanded">
                    <input class="input" type="text" v-model="$root.$data.install_location"
                           placeholder="Enter a install path here">
                </div>
                <div class="control">
                    <a class="button is-dark" v-on:click="select_file">
                        Select
                    </a>
                </div>
            </div>

            <div class="is-right-floating is-bottom-floating">
                <div class="field is-grouped">
                    <p class="control">
                        <a class="button is-medium" v-if="!$root.$data.config.hide_advanced && !$root.$data.metadata.preexisting_install && !advanced" 
                           v-on:click="advanced = true">Advanced...</a>
                    </p>
                    <p class="control">
                        <a class="button is-dark is-medium" v-if="!$root.$data.metadata.preexisting_install" 
                           v-on:click="install">Install</a>
                    </p>
                    <p class="control">
                        <a class="button is-dark is-medium" v-if="$root.$data.metadata.preexisting_install"
                           v-on:click="install">Modify</a>
                    </p>
                </div>
            </div>
            
            <div class="field is-grouped is-left-floating is-bottom-floating">
                <p class="control">
                    <a class="button is-medium" v-if="$root.$data.metadata.preexisting_install"
                       v-on:click="go_back">Back</a>
                </p>
            </div>
        </div>
    `,
    data: function() {
        return {
            advanced: false
        }
    },
    methods: {
        select_file: function() {
            window.external.invoke(JSON.stringify({
                SelectInstallDir: {
                    callback_name: "selectFileCallback"
                }
            }));
        },
        install: function() {
            router.push("/install/regular");
        },
        go_back: function() {
            router.go(-1);
        }
    }
};

const InstallPackages = {
    template: `
        <div class="column has-padding">
            <h4 class="subtitle" v-if="$root.$data.metadata.is_launcher || is_update">Checking for updates...</h4>
            <h4 class="subtitle" v-else-if="is_uninstall">Uninstalling...</h4>
            <h4 class="subtitle" v-else-if="is_updater_update">Downloading self-update...</h4>
            <h4 class="subtitle" v-else>Installing...</h4>
            <div v-html="$root.$data.config.installing_message"></div>
            <br />

            <div v-html="progress_message"></div>
            <progress class="progress is-info is-medium" v-bind:value="progress" max="100">
                {{ progress }}%
            </progress>
        </div>
    `,
    data: function() {
        return {
            progress: 0.0,
            progress_message: "Please wait...",
            is_uninstall: false,
            is_updater_update: false,
            is_update: false,
            failed_with_error: false,
            packages_installed: 0
        }
    },
    created: function() {
        this.is_uninstall = this.$route.params.kind === "uninstall";
        this.is_updater_update = this.$route.params.kind === "updater";
        this.is_update = this.$route.params.kind === "update";
        console.log("Installer kind: " + this.$route.params.kind);
        this.install();
    },
    methods: {
        install: function() {
            var results = {};

            for (var package_index = 0; package_index < app.config.packages.length; package_index++) {
                var current_package = app.config.packages[package_index];
                if (current_package.default != null) {
                    results[current_package.name] = current_package.default;
                }
            }

            results["path"] = app.install_location;

            var that = this; // IE workaround

            var targetUrl = "/api/start-install";
            if (this.is_uninstall) {
                targetUrl = "/api/uninstall";
            }
            if (this.is_updater_update) {
                targetUrl = "/api/update-updater";
            }

            stream_ajax(targetUrl, function(line) {
                if (line.hasOwnProperty("Status")) {
                    that.progress_message = line.Status[0];
                    that.progress = line.Status[1] * 100;
                }

                if (line.hasOwnProperty("PackageInstalled")) {
                    that.packages_installed += 1;
                }

                if (line.hasOwnProperty("Error")) {
                    if (app.metadata.is_launcher) {
                        app.exit();
                    } else {
                        that.failed_with_error = true;
                        router.replace({name: 'showerr', params: {msg: line.Error}});
                    }
                }
            }, function(e) {
                if (that.is_updater_update) {
                    // Continue with what we were doing
                    if (app.metadata.is_launcher) {
                        router.replace("/install/regular");
                    } else {
                        if (app.metadata.preexisting_install) {
                            router.replace("/modify");
                        } else {
                            router.replace("/packages");
                        }
                    }
                } else {
                    if (app.metadata.is_launcher) {
                        app.exit();
                    } else if (!that.failed_with_error) {
                        if (that.is_uninstall) {
                            router.replace({name: 'complete', params: {
                                uninstall: true,
                                update: that.is_update,
                                installed: that.packages_installed
                            }});
                        } else {
                            router.replace({name: 'complete', params: {
                                uninstall: false,
                                update: that.is_update,
                                installed: that.packages_installed
                            }});
                        }
                    }
                }
            }, undefined, results);
        }
    }
};

const ErrorView = {
    template: `
        <div class="column has-padding">
            <h4 class="subtitle">An error occurred:</h4>

            <pre>{{ msg }}</pre>

            <div class="field is-grouped is-right-floating is-bottom-floating">
                <p class="control">
                    <a class="button is-primary is-medium" v-if="remaining" v-on:click="go_back">Back</a>
                </p>
            </div>
        </div>
    `,
    data: function() {
        return {
            msg: this.$route.params.msg,
            remaining: window.history.length > 1
        }
    },
    methods: {
        go_back: function() {
            router.go(-1);
        }
    }
};

const CompleteView = {
    template: `
        <div class="column has-padding">
            <div v-if="was_update">
                <div v-if="has_installed">
                    <h4 class="subtitle">{{ $root.$data.attrs.name }} has been updated.</h4>
    
                    <p>You can find your installed applications in your start menu.</p>
                </div>
                <div v-else>
                    <h4 class="subtitle">{{ $root.$data.attrs.name }} is already up to date!</h4>
    
                    <p>You can find your installed applications in your start menu.</p>
                </div>
            </div>
            <div v-else-if="was_install">
                <h4 class="subtitle">Thanks for installing {{ $root.$data.attrs.name }}!</h4>

                <p>You can find your installed applications in your start menu.</p>
            </div>
            <div v-else>
                <h4 class="subtitle">{{ $root.$data.attrs.name }} has been uninstalled.</h4>
            </div>

            <div class="field is-grouped is-right-floating is-bottom-floating">
                <p class="control">
                    <a class="button is-dark is-medium" v-on:click="exit">Exit</a>
                </p>
            </div>
        </div>
    `,
    data: function() {
        return {
            was_install: !this.$route.params.uninstall,
            was_update: this.$route.params.update,
            has_installed: this.$route.params.packages_installed > 0
        }
    },
    methods: {
        exit: function() {
            app.exit();
        }
    }
};

const ModifyView = {
    template: `
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
    `,
    data: function() {
        return {
            show_uninstall: false
        }
    },
    methods: {
        update: function() {
            router.push("/install/update");
        },
        modify_packages: function() {
            router.push("/packages");
        },
        prepare_uninstall: function() {
            this.show_uninstall = true;
        },
        cancel_uninstall: function() {
            this.show_uninstall = false;
        },
        uninstall: function() {
            router.push("/install/uninstall");
        },
    }
};

const router = new VueRouter({
    routes: [
        {
            path: '/config',
            name: 'config',
            component: DownloadConfig
        },
        {
            path: '/packages',
            name: 'packages',
            component: SelectPackages
        },
        {
            path: '/install/:kind',
            name: 'install',
            component: InstallPackages
        },
        {
            path: '/showerr',
            name: 'showerr',
            component: ErrorView
        },
        {
            path: '/complete/:uninstall/:update/:packages_installed',
            name: 'complete',
            component: CompleteView
        },
        {
            path: '/modify',
            name: 'modify',
            component: ModifyView
        },
        {
            path: '/',
            redirect: '/config'
        }
    ]
});
