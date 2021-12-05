#!/bin/env node
const fs = require('fs')
const merge = require('deepmerge')
const glob = require('glob')

glob('src/locales/!(messages).json', {}, (e, files) => {
  let messages = []
  for (const file of files) {
    console.log(`Loading ${file}...`)
    const locale_messages = require(`./${file}`)
    messages.push(locale_messages)
  }
  console.log('Merging messages...')
  if (messages && messages.length > 1) {
    messages = merge.all(messages)
  } else {
    messages = messages[0] // single locale mode
  }
  fs.writeFileSync('src/locales/messages.json', JSON.stringify(messages), {})
})
