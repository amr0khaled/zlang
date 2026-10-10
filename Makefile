
CARGO:=cargo

FILE=./test/index.z

PROFILE=release

default: run

debug: PROFILE=dev
debug: run

run:
	CARGO_PROFILE=$(PROFILE) $(CARGO) run $(FILE) --profile $(PROFILE)

.PHONY: test
test:
	$(CARGO) test

clean:
	$(CARGO) clean

