# kurl
HTTP Requests with some status

# Installation

```bash
git clone https://github.com/gbrls/kurl
# To install it with cargo you need to have the Rust toolchain installed
cd kurl && cargo install --path .
```

# Usage

```
Usage: kurl [OPTIONS] <URL>

Arguments:
  <URL>  URL to send the request

Options:
  -c, --status-code
  -s, --size
  -j, --valid-json
  -t, --content-type
  -n, --no-body
  -k, --keys          Try to guess the JSON's format
      --all           Display all status
  -h, --help          Print help
  -V, --version       Print version
```
