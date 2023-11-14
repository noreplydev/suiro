# suiro
![suiro-logo](https://github.com/noreplydev/suiro-rs/blob/assets/Screenshot%202023-10-18%20at%2021.29.19.png?raw=true)
Suiro is a NAT traversal for HTTP protocol based services. It allows you to expose your local services to the internet without having to configure your router or firewall. 

## limitations
Suiro is still in development but we can predict some limitations that will be present in the first release.

#### · url resolution
When a exposed service like a react app refers to a local resource (like an image) with relative routes it could not work. When a resource is requested using relative routes the browser will try to find the resource in the server. To be more clear, if the service endpoint is `https://52.23.234.23/8noiasdb238` and the resource is `/static/image.png` the browser will try to find the resource in `https://52.23.234.23/static/image.png` instead of `https://52.23.234.23/8noiasdb238/static/image.png` so to solve this, the tunneling server uses the referer header to make url resolution but this header is not required by the HTTP protocol so it can be missing in some cases. 

## install binary
```bash
cargo install https://github.com/noreplydev/suiro --git 
```

## features roadmap (the order could change)
- [x]️ Basic tunneling
- [_]️ Tcp connection buffer
- [_] Encrypt data between agent and server
- [_] Port suiro agent to rust (currently in nodejs)

made with ❤️ by @noreplydev