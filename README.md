
<h1 align="center">suiro</h1>

![suiro-logo](https://raw.githubusercontent.com/noreplydev/suiro/assets/suiro_logo.png)
Suiro is a NAT traversal for HTTP protocol based services. It allows you to expose your local services to the internet without having to configure your router or firewall. 

<h2 align="center">Installation</h2>

```bash
cargo install --git https://github.com/noreplydev/suiro
```

<h2 align="center">Known issues</h2>

<h4 align="center">url resolution</h4>
When a exposed service like a react app refers to a local resource (like an image) with relative routes it could not work. When a resource is requested using relative routes the browser will try to find the resource in the server. To be more clear, if the service endpoint is `https://52.23.234.23/8noiasdb238` and the resource is `/static/image.png` the browser will try to find the resource in `https://52.23.234.23/static/image.png` instead of `https://52.23.234.23/8noiasdb238/static/image.png` so to solve this, the tunneling server uses the referer header to make url resolution but this header is not required by the HTTP protocol so it can be missing in some cases. 

<h2 align="center">features roadmap (the order could change)</h2>

- [x]️ Basic tunneling
- [_]️ Tcp connection buffer
- [_] Encrypt data between agent and server
- [_] Port suiro agent to rust (currently in nodejs)


<p align="center">made with ❤️ by @noreplydev</p>