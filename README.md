# suiro-rs
![suiro-rs-logo](https://github.com/noreplydev/suiro-rs/blob/assets/Screenshot%202023-10-18%20at%2021.29.19.png?raw=true)
suiro-rs is the [suiro](https://github.com/noreplydev/suiro) port to rust. Basically, it handles the same things as suiro, but in rust. 

## why?
Rust is very fast and reliable language. This caracteristics makes it perfect for a program like suiro that needs to handle different connections at the same time. Also, rust can be compiled to a target architecture and OS, so it can be used in any system without the need of installing rust.

## differences
- Architecture: Since rust supports multi-threading, suiro-rs is multi-threaded using tokio

- Connections management: In suiro we use alive-sessions package to manage connections and timeouts, but in suiro-rs we use a custom implementation of the alive connections.  

## Limitations
Suiro is still in development but we can predict some limitations that will be present in the first release.

#### · Url resolution
When a exposed service like a react app refers to a local resource (like an image) with relative routes it could not work. When a resource is requested using relative routes the browser will try to find the resource in the server. To be more clear, if the service endpoint is `https://52.23.234.23/8noiasdb238` and the resource is `/static/image.png` the browser will try to find the resource in `https://52.23.234.23/static/image.png` instead of `https://52.23.234.23/8noiasdb238/static/image.png` so to solve this, the tunneling server uses the referer header to make url resolution but this header is not required by the HTTP protocol so it can be missing in some cases. 

## install binary
```bash
cargo install https://github.com/noreplydev/suiro-rs --git 
```

## Issues 
If you find any issue, please report on github issues or email me at noreplycristiansanchez@gmail.com

made with ❤️ by @noreplydev