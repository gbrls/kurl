# kurl
HTTP Requests with some status

# Installation

```bash
git clone https://github.com/gbrls/kurl
# To install it with cargo you need to have the Rust toolchain installed
cd kurl && cargo install --path .
```
### Example
```bash
❯ kurl ipinfo.io/8.8.8.8 --all
200 308 isjson=true "city country hostname ip loc org postal readme region timezone" "application/json; charset=utf-8"
{
  "ip": "8.8.8.8",
  "hostname": "dns.google",
  "anycast": true,
  "city": "Mountain View",
  "region": "California",
  "country": "US",
  "loc": "37.4056,-122.0775",
  "org": "AS15169 Google LLC",
  "postal": "94043",
  "timezone": "America/Los_Angeles",
  "readme": "https://ipinfo.io/missingauth"
}
```

# Help

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
