// Overwrite loggers with the logging backend
if (window.external !== undefined && window.external.invoke !== undefined) {
    window.onerror = function(msg, url, line) {
        old_onerror(msg, url, line);
        window.external.invoke(JSON.stringify({
            Log: {
                kind: "error",
                msg: msg + " @ " + url + ":" + line
            }
        }));
    };

    // Borrowed from http://tobyho.com/2012/07/27/taking-over-console-log/
    function intercept(method){
        console[method] = function(){
            var message = Array.prototype.slice.apply(arguments).join(' ');
            window.external.invoke(JSON.stringify({
                Log: {
                    kind: method,
                    msg: message
                }
            }));
        }
    }

    var methods = ['log', 'warn', 'error'];
    for (var i = 0; i < methods.length; i++) {
        intercept(methods[i]);
    }
}

// Disable F5
function disable_shortcuts(e) {
    switch (e.keyCode) {
        case 116: // F5
            e.preventDefault();
            break;
    }
}

window.addEventListener("keydown", disable_shortcuts);

document.getElementById("window-title").innerText = base_attributes.name + " Installer";

function selectFileCallback(name) {
    app.install_location = name;
}

var app = new Vue({
    router: router,
    data: {
        attrs: base_attributes,
        config : {},
        install_location : "",
        // If the option to pick an install location should be provided
        show_install_location : true,
        metadata : {
            database : [],
            install_path : "",
            preexisting_install : false
        }
    },
    methods: {
        "exit": function() {
            ajax("/api/exit", function() {});
        }
    }
}).$mount("#app");
