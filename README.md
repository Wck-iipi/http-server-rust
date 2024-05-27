# http-server-rust
This is a web server written in rust. It implements the following: \
1. Support for get and post with 200 and 404 error codes.
2. Support for commands like echo, user-agent
3. Has GZip compression support

## How to run:
1. Install this with `cargo build`
2. Run this server with `./your_server.sh` in one window.
3. Run `curl -i http://localhost:4221/` to run and see output of server.
4. Add commands supported to this server. For example echo will have: `curl -i http://localhost:4221/echo/abc`
<img width="1168" alt="image" src="https://github.com/Wck-iipi/http-server-rust/assets/110763795/b3a9d7ca-2aa2-42ce-9e1d-1c09bdb669f0">
