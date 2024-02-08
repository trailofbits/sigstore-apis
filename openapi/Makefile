# We technically don't need to part these targets out because the script generates
# both targets when invoked, but there isn't a straightforward portable way to group
# multiple targets under the same commands.
.PHONY: all
all: fulcio.openapi.json rekor.openapi.json

fulcio.openapi.json: versions
	./update.sh

rekor.openapi.json: versions
	./update.sh
