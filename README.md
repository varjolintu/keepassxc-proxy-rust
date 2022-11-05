# keepassxc-proxy-rust
Application that works as a proxy between Native Messaging browser extension and KeePassXC

This is still under development. Installing the proxy needs manual changes to JSON scripts installed for Native Messaging.
See [this page](https://developer.chrome.com/extensions/nativeMessaging) for further information.

keepassxc-proxy listens stdin from keepassxc-browser extension and transfers the data to Unix domain socket `XDG_RUNTIME_DIR` or `/tmp/org.keepassxc.KeePassXC.BrowserServer` which KeePassXC listens.
With Windows this is a named pipe under `org.keepassxc.KeePassXC.BrowserServer\<username>`.


## Installing

### Alpine Linux

If you use Alpine Linux, you can install the proxy from the [keepassxc-proxy-static](https://pkgs.alpinelinux.org/packages?name=keepassxc-proxy-static) package.
It's built as a static binary, so it can be used with a browser installed from and running in Flatpak.
This package is available in Alpine Linux repositories since (upcoming) v3.17 and in Edge.

```bash
apk add keepassxc-proxy-static
```

You can then install the proxy and associated config into Firefox or Chromium using the `keepassxc-proxy-install` command.
Run `keepassxc-proxy-install -h` for more information.

## Building

The proxy can be built with:

```bash
cargo build --release
```

### Static library

To build a binary without dependencies (which is useful for running
inside of a flatpak), you'll have to install MUSL libc first:

```bash
rustup target add x86_64-unknown-linux-musl
```

Then build with

```bash
RUSTFLAGS='-C link-arg=-s' cargo build --release --target x86_64-unknown-linux-musl
```

(see [Stackoverflow](https://stackoverflow.com/a/59766875/487503))
## Copyright

```
Copyright (C) 2017-2022 Sami Vänttinen <sami.vanttinen@ahmala.org>
Copyright (C) 2017-2018 Andy Brandt <andy@brandt.tech>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
```
