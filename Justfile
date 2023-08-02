ui-build:
    yarn --cwd {{ justfile_directory() }}/ui/ install
    yarn --cwd {{ justfile_directory() }}/ui/ build

ui-test:
    cd {{ justfile_directory() }}/ui/ && node mock-server.js &
    yarn --cwd {{ justfile_directory() }}/ui/ serve

update-i18n:
    #!/bin/bash -e
    [ -z "${TX_PULL}" ] || tx pull -a --minimum-perc 85
    for i in {{ justfile_directory() }}/ui/translations/*.po; do
        TARGET_FILE="$(basename $i)"
        TARGET_LANG="${TARGET_FILE/.po/}"
        OUTPUT="{{ justfile_directory() }}/ui/src/locales/${TARGET_LANG}.json"
        i18next-conv -l en -s "$i" -t "$OUTPUT" -K
        node {{ justfile_directory() }}/ui/unbreak-translations.js "$OUTPUT" "$OUTPUT"
    done
