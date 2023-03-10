# kurl
Simple CLI HTTP client focused on security research.

`kurl` has features such as:
- Display status code
- Display content-type
- Check if the body is valid json
- Guess the json format
- Display the content-length

# Installation

```bash
# To install it with cargo you need to have the Rust toolchain installed
cargo install kurl
```
### Example
```bash
~ ❯ kurl ipinfo.io/8.8.8.8 --all
200 304 get json "anycast city country hostname ip loc org postal readme region timezone" "application/json; charset =utf-8" ipinfo.io/8.8.8.8 
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
Simple CLI HTTP client focused on security research

Usage: kurl [OPTIONS] <URL>

Arguments:
  <URL>  URL to send the request

Options:
  -c, --status-code        
  -s, --size               
  -j, --valid-json         
  -t, --content-type       
  -n, --no-body            
  -k, --keys               Try to guess the JSON's format
  -u, --show-url           Display the URL
      --all                Display all status
      --scripts <SCRIPTS>  
  -X <VERB>                [default: GET] [possible values: POST, GET, HEAD]
  -d, --data <DATA>        
  -h, --help               Print help
  -V, --version            Print version
```
