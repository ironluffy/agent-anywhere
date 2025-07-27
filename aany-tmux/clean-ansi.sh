#!/bin/bash
# Clean ANSI escape codes from text

# More comprehensive ANSI code removal
sed -E 's/\x1b\[[0-9;]*m//g' | \
sed -E 's/\x1b\[[0-9]+;[0-9]+;[0-9]+;[0-9]+;[0-9]+m//g' | \
sed -E 's/\x1b\[[0-9]+;[0-9]+H//g' | \
sed -E 's/\x1b\[K//g' | \
sed -E 's/\x1b\[[0-9]*[A-Za-z]//g' | \
sed -E 's/\x1b\]0;[^\x07]*\x07//g' | \
sed -E 's/\[\?[0-9]+[hl]//g' | \
sed -E 's/\[[0-9]+m//g'