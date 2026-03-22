MAN_SRCS := $(wildcard docs/man-src/*.*.md)
MAN_PAGES := $(patsubst docs/man-src/%.md,docs/man/%,$(MAN_SRCS))

.PHONY: man clean-man

man: $(MAN_PAGES)

docs/man/%: docs/man-src/%.md
	mkdir -p docs/man
	pandoc -s -t man $< -o $@

clean-man:
	rm -f docs/man/*.1 docs/man/*.7
