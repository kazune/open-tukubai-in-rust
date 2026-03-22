MAN_SRCS := $(wildcard docs/man-src/*.*.md)
MAN_PAGES := $(patsubst docs/man-src/%.md,docs/man/%,$(MAN_SRCS))

.PHONY: man man-lint man-check clean-man

man: $(MAN_PAGES)

man-lint: $(MAN_PAGES)
	mandoc -Tlint docs/man/*.1 docs/man/*.7

man-check: man man-lint

docs/man/%: docs/man-src/%.md
	mkdir -p docs/man
	pandoc -s -t man $< -o $@

clean-man:
	rm -f docs/man/*.1 docs/man/*.7
