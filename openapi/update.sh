#!/usr/bin/env bash

set -eu -o pipefail

set +a
source versions
set -a

WORKDIR=$(mktemp -d)
SCHEMATOOLS=$(realpath .)/bin/schematools-cli

info() {
    echo "[-] $@"
}

# Use schematools to dereference schema: Progenitor can't handle references.
install_schematools() {
    info "\`cargo install\` for schema-tools ..."

    cargo install \
        --git https://github.com/kstasik/schema-tools.git \
        --tag v0.18.1 \
        --bin schematools-cli \
        --root "." 2>/dev/null # --root "$WORKDIR"
}

# Bend the schema to our steadfast, unwavering will.
transform_schema() {
    "$SCHEMATOOLS" process dereference --skip-root-internal-references "$1" | # tee /dev/stderr |
    jq \
    '# Recursively iterate over all objects in the file.
    walk(if type == "object" then
        with_entries(
            # Convert 200 to 2XX to work around response type issues.
            # https://github.com/oxidecomputer/progenitor/issues/344
            if .key | test("^2\\d\\d$") then
                .key = "2XX"
            # Delete enumerated error code cases per above.
            elif .key | test("^4\\d\\d$") then
                empty
            # Convert base64-encoded objects to bare strings, as the former
            # is not technically in the OpenAPI spec.
            # https://spec.openapis.org/registry/format/byte.html
            elif .key == "attestation" and .value.type? == "object" then
                .value = {"type": "string"}
            # Change the catch-all error case to a more generic type now that
            # we are deleting the individual error cases.
            elif try(.value.default."$ref" != null) catch false then
                .value.default = {
                    "description": "An issue occurred while processing the request.",
                    "schema": {"$ref": "#/definitions/Error"}
                }
        end)
    else . end)'
}

# The Rekor and Fulcio OpenAPI specs are written with the 2.0 spec in mind, but Progenitor only
# supports OpenAPI 3.0. Converting between 2 and 3 locally seems non-trivial (i.e. involves a JVM),
# so we call a web service here that does it for us.
convert_openapi() {
    curl --silent \
        -H "Accept: application/json" \
        -H "Content-Type: application/json" \
        -d "@/dev/stdin" \
        "https://converter.swagger.io/api/convert"
}

# Download and transform the specs.
download_spec() {
    git clone --depth 1 -b "$3" "$2" "$1" 2>/dev/null
    pushd "$1" >/dev/null

    # XX: This ordering is important! The Swagger Converter service can't handle
    # the raw schema since it contains references.
    transform_schema "$4" |
        convert_openapi |
        tee "$(dirs -l +2)/$service.openapi.json" |
        openssl dgst -sha256 -binary |
        xxd -p -c 32

    popd >/dev/null
}

install_schematools

mkdir -p "$WORKDIR"
pushd "$WORKDIR"

for service in FULCIO REKOR; do
    service_ref_var="${service}_REF"
    service_repo_var="${service}_REPO"
    service_path_var="${service}_PATH"
    service_hash_var="${service}_HASH"

    service=$(echo "$service" | tr '[A-Z]' '[a-z]')
    info "working on schema for $service ..."

    actual_hash=$(download_spec \
        "$service" \
        "${!service_repo_var}" "${!service_ref_var}" \
        "${!service_path_var}")
    expect_hash="${!service_hash_var}"

    # Compare hashes to make sure nothing fishy is going on with the converter API.
    if [ "$expect_hash" != "$actual_hash" ]; then
        info "hash for $service doesn't match; please verify the generated schema and update the versions file"
        info "expect=$expect_hash"
        info "actual=$actual_hash"
    fi
done

popd
rm -rf "$WORKDIR"
