# Infiltrator
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/fabianbinna/infiltrator/build.yml) ![GitHub release (with filter)](https://img.shields.io/github/v/release/fabianbinna/infiltrator)

A web server that bypasses web proxies to download files that would otherwise be blocked.

## :building_construction: Installation

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

## :key: TLS

> :warning: **It is highly recommended to use TLS when transmitting sensitive data!**

To enable TLS communication, the following configs need to be done in the `Config.toml`:
```
[default.tls]
key = "cert/key.pem"
certs = "cert/cert.pem"
```

## :rocket: Usage

> :information_source: Open the dev tools of the browser to observe the download progress.

1. Start the infiltartor on a server available from the Internet.
2. Put files to download in the `data` folder.
3. Access the infiltrator over a browser on `http://<ip>:<port>`, enter the filename and click download.
4. The file will then be downloaded in base64 chunks.


