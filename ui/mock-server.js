'use strict'

const express = require('express')
const app = express()
const port = 3000

let showError = false
let showConfigError = false
let maintenance = false
let launcher = false
let fileExists = false
let darkMode = false
let recoveryMode = false

function progressSimulation (res) {
  if (showError) {
    const resp = JSON.stringify({ Error: 'Simulated error.' }) + '\n'
    res.write(resp)
    res.status(200).end()
    return
  }
  let progress = 0.0
  const timer = setInterval(() => {
    const resp = JSON.stringify({ Status: ['Processing...', progress] }) + '\n'
    progress += 0.1
    res.write(resp)
    if (progress >= 1) {
      res.status(200).end()
      clearInterval(timer)
    }
  }, 500)
}

function returnConfig (res) {
  if (showConfigError) {
    res.status(500).json({})
    return
  }
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
  console.log('-- Get attrs')
  res.send(
    { name: 'yuzu', recovery: recoveryMode, target_url: 'https://raw.githubusercontent.com/j-selby/test-installer/master/config.linux.v2.toml' }
  )
})

app.get('/api/dark-mode', (req, res) => {
  res.json(darkMode)
})

app.get('/api/installation-status', (req, res) => {
  res.json({
    database: { packages: [], shortcuts: [] },
    install_path: null,
    preexisting_install: maintenance,
    is_launcher: launcher,
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
  console.log('-- Install:')
  console.log(req.body)
  progressSimulation(res)
})

app.get('/api/exit', (req, res) => {
  console.log('-- Exit')
  if (showError) {
    res.status(500).send('Simulated error: Nothing to see here.')
    return
  }
  res.status(204).send()
})

app.post('/api/verify-path', (req, res) => {
  console.log('-- Verify Path')
  res.send({
    exists: fileExists
  })
})

app.post('/api/check-auth', (req, res) => {
  console.log('-- Check Authorization')
  res.send({
    username: 'test1',
    token: 'token',
    jwt_token: {
      isPatreonAccountLinked: true,
      isPatreonSubscriptionActive: true,
      releaseChannels: ['early-access']
    }
  })
})

process.argv.forEach((val, index) => {
  switch (val) {
    case 'maintenance':
      maintenance = true
      console.log('Simulating maintenance mode')
      break
    case 'launcher':
      maintenance = true
      launcher = true
      console.log('Simulating launcher mode')
      break
    case 'exists':
      fileExists = true
      console.log('Simulating file exists situation')
      break
    case 'dark':
      darkMode = true
      console.log('Simulating dark mode')
      break
    case 'config-error':
      showConfigError = true
      console.log('Simulating configuration errors')
      break
    case 'error':
      showError = true
      console.log('Simulating errors')
      break
    case 'recovery':
      recoveryMode = true
      console.log('Simulating recovery mode')
      break
  }
})

console.log(`Listening on ${port}...`)
app.listen(port)
