# Daemon framing v1

Frame: uint32 little-endian byte length followed by 1–65536 UTF-8 bytes. Requests
are fixed command names with newline-separated arguments. No shell evaluates
request text. Responses are JSON with ok/data or ok/error. All administrative
commands authenticate SO_PEERCRED and the current Manager package/APK pin.

Module upload sends `install\n<size>`, waits for `{"ready":true}`, then transfers
exactly size raw bytes (maximum 256 MiB). Final success is sent only after archive
validation, extraction, installer completion and update staging. Ordinary apps
may request `authorize-root` only; the actual credential grant uses kernel UAPI.
