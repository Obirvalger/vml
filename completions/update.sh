#!/bin/sh -eu

DIR="$(dirname $0)"
pushd "$DIR"

vml completion bash > bash
patch bash < bash.diff
mv bash ../files/completions/bash

vml completion zsh > zsh
patch zsh < zsh.diff
mv zsh ../files/completions/zsh
