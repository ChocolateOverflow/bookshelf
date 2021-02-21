#!/usr/bin/python3

import re
import requests
import sys

def is_url_valid(url: str) -> bool:
    return bool(re.match(r"https://example.com/book/[0-9]+/$", url))

def get_code(url: str) -> str:
    return url.split("/")[-1].strip()

def get_url(code: str) -> str:
    return f"https://example.com/book/{code}/"

def get_metadata(url: str) -> tuple[str, set[str], set[str]]:
    txt = requests.get(url).text
    title = "CHANGE THIS"
    authors = {"CHANGE", "THIS"}
    tags = {"CHANGE-THIS"}
    return (title, authors, tags)

def download_item(url: str, out_dir: str):
    pass

# main
if len(sys.argv) == 2:
    if sys.argv[1] == "media":
        print("jpg", end="")

elif len(sys.argv) == 3:
    if sys.argv[1] == "check":
        if is_url_valid(sys.argv[2]):
            print(1, end="")
        else:
            print(0, end="")

    elif sys.argv[1] == "code":
        print(get_code(sys.argv[2]), end="")

    elif sys.argv[1] == "url":
        print(get_url(sys.argv[2]), end="")

    elif sys.argv[1] == "metadata":
        metadata = get_metadata(get_url(sys.argv[2]))
        title = metadata[0]
        authors = ",".join(metadata[1])
        tags = ",".join(metadata[2])
        print(f"{title}\n{authors}\n{tags}", end="")

elif len(sys.argv) == 4:
    if sys.argv[1] == "download":
        download_item(get_url(sys.argv[2]), sys.argv[3])
