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

# Use schematools to dereference schema: Progenitor can't handle external
# references.
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
    cat $1 | if ! echo $1 | grep -iq "json$"; then
    perl -MJSON -M"YAML::PP" -e "print encode_json(YAML::PP::LoadFile('/dev/stdin'))" ; else cat; fi |
    jq -S \
    '
    .definitions += ([
        .. | try(."$ref" | select(test("json$"))) | {"$ref": .}
    ] | with_entries(.key = (.key | tostring) + "_tmp")) |
    # Recursively iterate over all objects in the file.
    walk(if type == "object" then
        with_entries(
            # Convert 200 to 2XX to work around response type issues.
            # https://github.com/oxidecomputer/progenitor/issues/344
            if .key | test("^2\\d\\d$") then
                .key = "2XX"
            # Delete enumerated error code cases per above.
            elif .key | test("^4\\d\\d$") then
                empty
            # LogEntry.body is incorrectly marked as an object in the current
            # schema; adjust to a string. Technically the same condition as
            # below, but separate for easy removal when fixed upstream.
            # https://github.com/sigstore/rekor/pull/2091
            elif .key == "body" and .value.type? == "object" then
                .value = {"type": "string"}
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

    # XX: This ordering is important! The swagger converter service can't handle
    # the raw schema since it contains references.
    transform_schema "$4" > "work.json"
    cp "work.json" ~/Documents/sw/sigstore-apis/

    # XX: This technically shouldn't work: schema-tools is designed for OpenAPI 3.0.
    # It happens to work for the current version of the schemas.
    "$SCHEMATOOLS" process dereference --skip-root-internal-references --create-internal-references "work.json" |
        "$SCHEMATOOLS" process merge-all-of --leave-invalid-properties /dev/stdin --to-file "work.json"

    # The conversion service screws our definitions up, so save them first.
    defs=$(cat "work.json" |
               jq -c '[(.definitions | to_entries[] | select(.key | test("_tmp")))] // []')

    cat "work.json" |
        convert_openapi |
        # Assign the external reference types sane names here. We were unable to do so before
        # the dereferencing step (the data we needed was not directly in the schema file),
        # so we might as well do it here.
        jq '
        def camelident: [splits(" ") | sub("[^A-Za-z0-9]"; "", "g")] | join(""); '"$defs as \$defs |"'
        # construct a mapping of temporary name -> new name; derive the new name from the title
        ([$defs[] | {"\(.key)": (.value.title | camelident)}] | add) as $renames |

        # remove the incorrect converted schema entry
        del(try(.components.schemas[$renames | keys[]])) |

        # add our saved definitions under the new names
        .components.schemas += ([$defs[] | {"key": $renames[.key], "value": .value}] | from_entries) |

        # fixup references
        (.. | ."$ref"? | select(type == "string")) |=
        if endswith("_tmp") then
            "#/components/schemas/\($renames[sub(".*(?<a>[0-9]+_tmp)$"; "\(.a)")])"
        else
            .
        end' |
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
