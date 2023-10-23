# suiro-rs
![suiro-rs-logo](https://github.com/noreplydev/suiro-rs/blob/assets/Screenshot%202023-10-18%20at%2021.29.19.png?raw=true)
suiro-rs is the [suiro](https://github.com/noreplydev/suiro) port to rust. Basically, it handles the same things as suiro, but in rust. 

## explanation
suiro-rs borns as a project to the `BCNRust` meetup group. So a full explanation of the codebase can be found on google presentation made for the meetup. [link](https://docs.google.com/presentation/d/1_E6IuHBWGSFeSWzCKA6qL2ul4Uy1GIk84CLSzTye6Rg/edit?usp=sharing)



## why?
Rust is very fast and reliable language. This caracteristics makes it perfect for a program like suiro that needs to handle different connections at the same time. Also, rust can be compiled to a target architecture and OS, so it can be used in any system without the need of installing rust.

## differences
- Architecture: Since rust supports multi-threading, suiro-rs is multi-threaded using each thread to manage each server.

- Connections management: In suiro we use alive-sessions package to manage connections and timeouts, but in suiro-rs we use a custom implementation of the sessions.  

## install binary
```bash
cargo install suiro-rs --git 
```

## Issues 
If you find any issue, please report on github issues or email me at noreplycristiansanchez@gmail.com

made with ❤️ by @noreplydev