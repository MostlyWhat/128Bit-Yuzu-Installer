'use strict'

const express = require('express')
const app = express()
const port = 3000

function progressSimulation (res) {
  var progress = 0.0
  var timer = setInterval(() => {
    var resp = JSON.stringify({ Status: ['Processing...', progress] }) + '\n'
    progress += 0.1
    res.write(resp)
    if (progress >= 1) {
      res.status(200).end()
      clearInterval(timer)
    }
  }, 1500)
}

function returnConfig (res) {
  res.json({
    installing_message:
      'Test Banner <strong>Bold</strong>&nbsp;<pre>Code block</pre>&nbsp;<i>Italic</i>&nbsp;<del>Strike</del>',
    new_tool: null,
    packages: [
      {
        name: 'Test 1',
        description: 'LiftInstall GUI Test 1',
        default: true,
        source: {
          name: 'github',
          match: '^test$',
          config: { repo: 'j-selby/liftinstall' }
        },
        shortcuts: []
      },
      {
        name: 'Test 2',
        description:
          'Different Banner <strong>Bold</strong>&nbsp;<pre>Code block</pre>&nbsp;<i>Italic</i>&nbsp;<del>Strike</del>',
        default: null,
        source: {
          name: 'github',
          match: '^test2$',
          config: { repo: 'j-selby/liftinstall' }
        },
        shortcuts: []
      }
    ],
    hide_advanced: false
  })
}

app.get('/api/attrs', (req, res) => {
  res.send(
    `var base_attributes = {"name":"yuzu","target_url":"https://raw.githubusercontent.com/j-selby/test-installer/master/config.linux.v2.toml"};`
  )
})

app.get('/api/dark-mode', (req, res) => {
  res.json(false)
})

app.get('/api/installation-status', (req, res) => {
  res.json({
    database: { packages: [], shortcuts: [] },
    install_path: null,
    preexisting_install: false,
    is_launcher: false,
    launcher_path: null
  })
})

app.get('/api/default-path', (req, res) => {
  res.json({ path: '/tmp/test/' })
})

app.get('/api/config', (req, res) => {
  setTimeout(() => {
    returnConfig(res)
  }, 3000)
})

app.post('/api/start-install', (req, res) => {
  console.log(`-- Install: ${req.body}`)
  progressSimulation(res)
})

app.get('/api/exit', (req, res) => {
  console.log('-- Exit')
  res.status(204)
})

console.log(`Listening on ${port}...`)
app.listen(port)
