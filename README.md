# keepassxc-proxy-rust
Application that works as a proxy between Native Messaging browser extension and KeePassXC

This is still under development. Installing the proxy needs manual changes to JSON scripts installed for Native Messaging.
See [this page](https://developer.chrome.com/extensions/nativeMessaging) for further information.

keepassxc-proxy listens stdin from keepassxc-browser extension and transfers the data to Unix domain socket `/tmp/kpxc_server` which KeePassXC listens.

```
Copyright (C) 2017 Sami Vänttinen <sami.vanttinen@protonmail.com>
Copyright (C) 2017 Andy Brandt <andy@brandt.tech>

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
