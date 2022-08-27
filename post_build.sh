#!/bin/bash
set -e

env="$1"
event="$2"
target="debug"


if [[ "$env" == "release" ]]; then
    target="release"
fi

if [[ "$event" == "before_asset_hash" ]]; then
    # Copy vendor assets
    mkdir -p dist/vendor/ace
    cp htom_web/vendor/ace/worker-html.js dist/vendor/ace
    cp htom_web/vendor/ace/ace.js dist/vendor/ace
    cp htom_web/vendor/ace/mode-html.js dist/vendor/ace
    cp htom_web/vendor/ace/keybinding-vim.js dist/vendor/ace
    cp htom_web/vendor/ace/keybinding-emacs.js dist/vendor/ace
fi

if [[ "$event" == "after_asset_hash" || "$env" == "dev" ]]; then
    # Generate html
    ./target/$target/htom_cli home_page > dist/index.html
fi
