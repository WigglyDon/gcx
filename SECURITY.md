# Security and private material

Report vulnerabilities without attaching private games, firmware, cards, saves,
credentials or device identifiers. Use GitHub private vulnerability reporting if
available; otherwise contact the maintainer through an established private route.
Do not publish a working exploit against another user or private data in issues.

Treat game/container metadata, manifests, cards, checkpoints, launch URIs and
external file grants as untrusted input. Bound parsing and allocation, validate
opaque IDs, refuse arbitrary native-core or save destinations, and preserve user
files on errors. Do not disable platform security to accommodate the engine.

The foundation itself does not configure a security contact or enable private
reporting. The Master must verify the actual available reporting route before
claiming one. Source review/build success is not a security audit.
