# kurl
Simple CLI HTTP client focused on security research.

`kurl` has features such as:
- Display status code
- Display content-type
- Check if the body is valid json
- Guess the json format
- Display the content-length

# Install Kurl

```bash
cargo install kurl
```
_For this to work you need to have [Rust installed](https://rustup.rs/)_

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

# Usage

```
kurl --help
```

Will show the command line usage.


```console
Simple CLI HTTP client focused on security research

Usage: kurl [OPTIONS] <URL_OR_FILE>

Arguments:
  <URL_OR_FILE>  URL or file with URLs to send the request

Options:
  -p <NWORKERS>
          Number of parallel threads to send the requests [default: 4]
  -X <VERB>
          [default: GET] [possible values: POST, GET, HEAD]
  -b, --body

  -d, --data <DATA>
          Data to be sent in the request body
      --verbosity-level <VERBOSITY_LEVEL>
          [default: 0]
  -o <OUTPUT>
          File to write the results
      --fext <FILTER_EXTENSIONS>
          Extensions to be ignored [default: jpeg,png,jpg,gif,wof,ttf,otf,eot,swf,ico,svg,css,woff,woff2]
      --fstatus <FILTER_STATUS>
          Status codes to be ignored [default: 404]
  -h, --help
          Print help
  -V, --version
          Print version
```
