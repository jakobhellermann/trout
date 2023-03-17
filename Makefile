.PHONY: pages
pages:
	rm -rf pages
	cd web && rm -rf .parcel-cache && make
