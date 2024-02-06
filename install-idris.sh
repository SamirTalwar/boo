#!/usr/bin/env bash

bash -c "$(curl -fsSL https://raw.githubusercontent.com/stefan-hoeck/idris2-pack/main/install.bash)"

pack install-app idris2-lsp
