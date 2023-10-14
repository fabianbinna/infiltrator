# Infiltrator
A web server that bypasses web proxies to download files that would otherwise be blocked.

## Installation

Download a zip from Releases. Unzip and run the executable. The `data` folder needs to be created manually or set another directory in `Config.toml`. The default profile is `default`.

Folder structure:
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

## Usage

Start the infiltartor on a server available from the Internet. Put files to download in the `data` folder. Access the infiltrator over a browser on `http://<ip>:<port>`, enter the filename and click download. The file will then be downloaded in base64 chunks.

Hint: Open the dev tools of the browser to observe the download progress.
