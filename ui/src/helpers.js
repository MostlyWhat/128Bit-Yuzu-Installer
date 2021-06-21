/**
 * helpers.js
 *
 * Additional state-less helper methods.
 */

var request_id = 0

/**
 * Makes a AJAX request.
 *
 * @param path The path to connect to.
 * @param successCallback A callback with a JSON payload.
 * @param failCallback A fail callback. Optional.
 * @param data POST data. Optional.
 */
export function ajax (path, successCallback, failCallback, data) {
  if (failCallback === undefined) {
    failCallback = defaultFailHandler
  }

  console.log('Making HTTP request to ' + path)

  var req = new XMLHttpRequest()

  req.addEventListener('load', function () {
    // The server can sometimes return a string error. Make sure we handle this.
    if (this.status === 200 && this.getResponseHeader('Content-Type').indexOf('application/json') !== -1) {
      successCallback(JSON.parse(this.responseText))
    } else {
      failCallback(this.responseText)
    }
  })
  req.addEventListener('error', failCallback)

  req.open(data == null ? 'GET' : 'POST', path + '?nocache=' + request_id++, true)
  // Rocket only currently supports URL encoded forms.
  req.setRequestHeader('Content-Type', 'application/x-www-form-urlencoded')

  if (data != null) {
    var form = ''

    for (var key in data) {
      if (!data.hasOwnProperty(key)) {
        continue
      }

      if (form !== '') {
        form += '&'
      }

      form += encodeURIComponent(key) + '=' + encodeURIComponent(data[key])
    }

    req.send(form)
  } else {
    req.send()
  }
}

/**
 * Makes a AJAX request, streaming each line as it arrives. Type should be text/plain,
 * each line will be interpreted as JSON separately.
 *
 * @param path The path to connect to.
 * @param callback A callback with a JSON payload. Called for every line as it comes.
 * @param successCallback A callback with a raw text payload.
 * @param failCallback A fail callback. Optional.
 * @param data POST data. Optional.
 */
export function stream_ajax (path, callback, successCallback, failCallback, data) {
  var req = new XMLHttpRequest()

  console.log('Making streaming HTTP request to ' + path)

  req.addEventListener('load', function () {
    // The server can sometimes return a string error. Make sure we handle this.
    if (this.status === 200) {
      successCallback(this.responseText)
    } else {
      failCallback(this.responseText)
    }
  })

  var buffer = ''
  var seenBytes = 0

  req.onreadystatechange = function () {
    if (req.readyState > 2) {
      buffer += req.responseText.substr(seenBytes)

      var pointer
      while ((pointer = buffer.indexOf('\n')) >= 0) {
        var line = buffer.substring(0, pointer).trim()
        buffer = buffer.substring(pointer + 1)

        if (line.length === 0) {
          continue
        }

        var contents = JSON.parse(line)
        callback(contents)
      }

      seenBytes = req.responseText.length
    }
  }

  req.addEventListener('error', failCallback)

  req.open(data == null ? 'GET' : 'POST', path + '?nocache=' + request_id++, true)
  // Rocket only currently supports URL encoded forms.
  req.setRequestHeader('Content-Type', 'application/x-www-form-urlencoded')

  if (data != null) {
    var form = ''

    for (var key in data) {
      if (!data.hasOwnProperty(key)) {
        continue
      }

      if (form !== '') {
        form += '&'
      }

      form += encodeURIComponent(key) + '=' + encodeURIComponent(data[key])
    }

    req.send(form)
  } else {
    req.send()
  }
}

/**
 * The default handler if a AJAX request fails. Not to be used directly.
 *
 * @param e The XMLHttpRequest that failed.
 */
function defaultFailHandler (e) {
  console.error('A AJAX request failed, and was not caught:')
  console.error(e)
}
