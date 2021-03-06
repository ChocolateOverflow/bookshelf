# Bookshelf - a small and modular media manager

**Bookshelf** is made for managing media, mainly books. Modules are to be made by the user (or stolen from the internet) and used to scrape data from websites, thereby getting metadata and download entire books and other items.

This is something I'm making to learn Rust. ~~It's totally not because I wanted a way to scrape books like manga from online and read them later.~~ It's made mainly for managing books but can be used to manage any media.

## Usage

### Adding and Downloading a new item

```sh
# Add to index (but don't download) item using module `example_mod` with code `12345`
bookshelf add -m example_mod -c 12345

# Add to index and download item from URL
bookshelf download -u https://example.com/book/12345
```

### Search in index

```sh
# Search for items whose titles begin with "The", are made by "John Doe" have
# the "sci-fi" and "comedy" but not "horror" genre
bookshelf search -t "^The.*" -a "John Doe" -g "sci-fi,comedy" -b "horror"

# Search in favorites fir books by either "Jane Smith" or "Bob Ross" and of genre "romance" or "comedy"
bookshelf search -f -a "Jane Smith,Bob Ross" -g "romance,comedy" --broad_search
```

### Getting item information

```sh
# Get item handled by module `example_mod` with code `12345`
bookshelf info -m example_mod -c  12345
```

### Update items

```sh
# Toggle favorite for book handled by `example_mod` with id `1234`
bookshelf update -m example_mod -c 12345 -f

# Update book with new title, author and genre
bookshelf update -m example_mod -c 12345 -t "New title" -a "Alice" -g "comedy,horror"
```

### Download saved item

```sh
# Download everything saved in index
bookshelf pull

# Download items handled by `example_mod` and created by "Bob"
bookshelf pull -m example_mod -a "Bob"
```

### Remove items

```sh
# Remove item from index and favorites, as well as its downloaded files
bookshelf rm -m example_mod -c 12345
```

### List available modules

```sh
bookshelf modules
```

### Import/Exporting shelf

```sh
# Export current index to yaml file
bookshelf export -f index.yaml

# Import yaml index file to current index
bookshelf import -f index.yaml
```

## TUI mode

To launch **bookshelf** in TUI mode, simply run `bookshelf` without arguments.

![TUI](./img/tui.png)

- `Down`/`j` : move down
- `Up`/`k` : move up
- `ctrl-D` : move down 50%
- `ctrl-U` : move up 50%
- `Home`: go to top
- `End`: go to bottom
- `F` : toggle favorite (add/remove item to/from favorites)
- `y` : yank (copy) item (module and code) to clipboard
- `o` : open item
- `w` : **w**rite to index file
- `ft`/`fa`/`fg` : filter by title regex/authors/genres
- `et`/`ea`/`eg` : edit title/authors/genres
- `Esc` : cancel filter/edit

## Configuration

Currently, the following settings (and their default values) can be set in bookshelf `.yaml`:

```yaml
  "index_file": "~/.config/bookshelf/index"
  "modules_dir": "~/.config/bookshelf/modules"
  "data_dir": "~/.config/bookshelf/data"
```

Books and other items are stored in `data_dir` in their own directories.

## Making a new module

A module can be written in any language. It only needs to be made executable and placed in the modules directory to be used. The `metadata` and `download` are mostly handled by the module with little to no help from `bookshelf` because every site has its own ways to get metadata and download items, and it's a lot simpler to have the individual modules handle everything for that.

A module should be able to handle the following commands:

### `your_mod check $URL`

Given a URL, the module prints a `1` (with or without newlines) if the URL can be processed by the module, and `0` otherwise

### `your_mod code $URL`

Given a URL, the module prints the code identifying the item.

### `your_mod url $CODE`

Given a code, the module prints the URL for the item.

### `your_mod metadata $CODE`

Print title, authors, and genres for the item with the provided code. The 3 items are on separate lines, with authors and genres being comma-separated. For example:

```
Rust for noobs
John Doe,Jane Smith
education,programming
```

### `your_mod download $CODE`

The module downloads the book with the provided code. No out put is expected, and everything is done by the module in this case. Any and all output is ignored by `bookshelf`, so feel free to print anything.

### `your_mod media`

Print the media type of the item handled by the module, for example: `jpg`, `png`, `pdf`, `txt`, `mp3`, `mp4`. This is for the (to be implemented) feature of opening the downloaded files with other programs.

## Misc

License: GNU GPLv3
