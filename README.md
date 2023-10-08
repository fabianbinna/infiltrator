# Infiltrator
A web server that bypasses web proxies to download files that would otherwise be blocked

## Build

Run `cargo build --relase`

Get the executable from `target/release/infiltrator[.exe]`

## Installation

Currently, there is no packaging script. The following files and directory structure needs to be created manually:

```
- data/
    - [files to download]
- static/
    - index.html
    - index.js
    - 404.html
- infiltrator[.exe]
- Config.toml
```

Edit the Config.toml to make it fit your needs.

## Usage

Start the infiltartor on a server available from the Internet. Put files to download in the `data` folder. Access the infiltrator over a browser on `http://<ip>:<port>`, enter the filename and click download. The file will then be downloaded in base64 chunks.

Hint: Open the dev tools of the browser to observe the download progress.
