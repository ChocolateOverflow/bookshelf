#!/bin/bash

get_metadata(){
cat <<< \
"""\
Test title
John Doe,Jane Smith
romance,comedy\
"""
}

download(){
    echo "Downloading book from $1 (code) to $2 (dir)"
}

case $1 in
    'check') echo "$2" | grep -E '^https?://example.com/.*' &>/dev/null && echo -n 1 || echo -n 0 ;;
    'code') echo -n "$2" | rev | cut -d'/' -f1 | rev ;;
    'url') echo -n "https://example.com/book/$2" ;;
    'metadata') get_metadata "$2" ;;
    'download') download "$2" "$3";;
    'media') echo -n "jpg";;
esac
