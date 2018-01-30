#
# justfile: automation tasks using the `just` package
#

default: test

build:
	cargo build
clean:
	cargo clean
test:
    cargo test --no-fail-fast


update: update-data

# Update the content of JSON data files by re-downloading it
# from the static data endpoint in PoE API.
update-data: update-currencies update-maps update-cards
DATA_URL = "https://www.pathofexile.com/api/trade/data/static"
update-currencies:
	curl {{DATA_URL}} 2>/dev/null | jq '.["result"]["currency"]' >./data/currency.json
update-maps:
	curl {{DATA_URL}} 2>/dev/null | jq '.["result"]["maps"]' >./data/maps.json
update-cards:
	curl {{DATA_URL}} 2>/dev/null | jq '.["result"]["cards"]' >./data/cards.json
