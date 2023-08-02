const path = require('path')
const fs = require('fs')

const input_file = process.argv[2]
const output_file = process.argv[3]

const mappings = {
  select: 'select_packages',
  install: 'install_packages',
  download: 'download_packages'
}

console.info(`Fixing ${input_file} ...`)
const lang = path.basename(input_file).replace('.json', '').replace('_', '-')
const translations = require(path.resolve(input_file))

translations[lang] = translations.en
delete translations.en

translations[lang].modify.modify = translations[lang].select['modify en'].modify.modify
delete translations[lang].select['modify en']
translations[lang].modify.repair = translations[lang].select['repair en'].modify.repair
delete translations[lang].select['repair en']

for (const i of Object.keys(mappings)) {
  translations[lang][mappings[i]] = translations[lang][i]
  delete translations[lang][i]
}

fs.writeFileSync(output_file, JSON.stringify(translations, null, 2))
