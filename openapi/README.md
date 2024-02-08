openapi
=======

This directory contains tooling for fetching and transforming the OpenAPI specs
used in generating the Rust client. They work around various limitations
that impede the generation of the unmodified specifications.

To (re)generate the specifications, update the `versions` file, then run `make`.
