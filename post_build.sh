#!/bin/bash
set -e

# Copy vendor assets
mkdir -p dist/vendor/ace
cp htom_web/vendor/ace/worker-html.js dist/vendor/ace
cp htom_web/vendor/ace/ace.js dist/vendor/ace
cp htom_web/vendor/ace/mode-html.js dist/vendor/ace
cp htom_web/vendor/ace/keybinding-vim.js dist/vendor/ace
cp htom_web/vendor/ace/keybinding-emacs.js dist/vendor/ace

# Generate html
cargo run -p htom_cli -- home_page > dist/index.html
