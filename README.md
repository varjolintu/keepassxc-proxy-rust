# keepassxc-proxy-rust
Application that works as a proxy between Native Messaging browser extension and KeePassXC

This is still under development. Installing the proxy needs manual changes to JSON scripts installed for Native Messaging.
See [this page](https://developer.chrome.com/extensions/nativeMessaging) for further information.

keepassxc-proxy listens stdin from keepassxc-browser extension and transfers the data to Unix domain socket `/tmp/kpxc_server` which KeePassXC listens.
