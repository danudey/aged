PREFIX    ?= /usr/local
BINDIR    ?= $(PREFIX)/bin
LIBDIR    ?= $(PREFIX)/lib
DATADIR   ?= $(PREFIX)/share
DESTDIR   ?=

CARGO     ?= cargo
INSTALL   ?= install
SED       ?= sed

BUILDDIR  := target/dist

.PHONY: all build dist install uninstall clean

all: build dist

build:
	$(CARGO) build --release

dist:
	@mkdir -p $(BUILDDIR)
	$(SED) 's|ExecStart=.*|ExecStart=$(BINDIR)/aged daemon|' dist/aged.service > $(BUILDDIR)/aged.service
	$(SED) 's|Exec=.*|Exec=$(BINDIR)/aged daemon|' dist/org.aged.Daemon.service > $(BUILDDIR)/org.aged.Daemon.service

install: all
	$(INSTALL) -Dm755 target/release/aged                    $(DESTDIR)$(BINDIR)/aged
	$(INSTALL) -Dm644 $(BUILDDIR)/aged.service               $(DESTDIR)$(LIBDIR)/systemd/user/aged.service
	$(INSTALL) -Dm644 $(BUILDDIR)/org.aged.Daemon.service    $(DESTDIR)$(DATADIR)/dbus-1/services/org.aged.Daemon.service
	$(INSTALL) -Dm644 dist/jurisdictions.toml                $(DESTDIR)$(DATADIR)/aged/jurisdictions.toml
	$(INSTALL) -Dm644 dist/aged.conf.toml                    $(DESTDIR)$(DATADIR)/doc/aged/aged.conf.toml

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/aged
	rm -f $(DESTDIR)$(LIBDIR)/systemd/user/aged.service
	rm -f $(DESTDIR)$(DATADIR)/dbus-1/services/org.aged.Daemon.service
	rm -f $(DESTDIR)$(DATADIR)/aged/jurisdictions.toml
	rm -f $(DESTDIR)$(DATADIR)/doc/aged/aged.conf.toml

clean:
	$(CARGO) clean
	rm -rf $(BUILDDIR)
