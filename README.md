# Fastly-compute-relay

Consumes HTTP Requests on `/` and stores them in a `Fastly KV` based buffer which can be later retrived by specific hosts on `/getrequests` endpoint.

![flow diagram](https://raw.githubusercontent.com/flouthoc/fastly-compute-relay/main/assets/Untitled-2024-08-08-1011.png)

## Usage

* Install `fastly` CLI plugin.

* `fastly compute serve`

## Security issues

Please see [SECURITY.md](SECURITY.md) for guidance on reporting security-related issues.
