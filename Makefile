.PHONY: all
all: piggui porky

.PHONY: porky
porky:
	cd porky && $(MAKE)

.PHONY: piggui
piggui:
	cd piggui && $(MAKE)