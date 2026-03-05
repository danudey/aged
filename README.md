# aged — Age Bracket Verification Daemon

`aged` is a daemon that stores a user's date of birth and exposes a D-Bus
API for applications to query which age bracket the user falls into under
a given jurisdiction.

California and Colorado have begun requiring OS vendors to collect date of
birth during device setup and expose a bracketed age API to applications.
`aged` fulfills this role on Linux: it stores the birthdate securely, ships
jurisdiction-specific bracket definitions, and lets any application on the
session bus ask "is this user a child, a minor, or an adult?" without ever
revealing the actual date of birth.

Because a lot of the specifics of how this will be implemented are still
up in the air, bills have yet to pass, etc., many things in this repository
are subject to change; this includes the D-Bus name and interface, the
specifics of what and how we return to clients, and so on.

None of the design is set in stone; please feel free to provide feedback on
issues like user privacy, security, unintentional data disclosure, and so on.

This entire project — code, packaging, documentation — was written by
Claude (Anthropic) as an experiment in AI-assisted software development
using [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

Lastly: please do not provide feedback on your opinion of these laws. Whether
we approve of them or not, they exist and, for the time being, at least some
users and vendors will be subject to them. The creation of this project is
not an endorsement of these laws or their purposes (stated or alleged).

## Usage

### Starting the daemon

The daemon runs as a systemd user service. If installed from a package or
via `make install`, D-Bus activation will start it automatically on first
use. To run it manually:

```
aged daemon
```

### Storing a birthdate

```
aged set-birthdate 1990-05-15
```

The date is stored in the user's keychain via the
[Secret Service](https://specifications.freedesktop.org/secret-service/latest/)
D-Bus API (e.g. GNOME Keyring or KWallet). If no Secret Service provider is
available, `aged` falls back to a plain file at
`~/.config/aged/data.toml`.

### Querying an age bracket

```
aged get-age-bracket --jurisdiction US/California
```

This prints a single label such as `child`, `minor`, or `adult`.

If you set a default jurisdiction, the `--jurisdiction` flag can be omitted:

```
aged set-default-jurisdiction US/California
aged get-age-bracket
```

### Listing jurisdictions

```
aged list-jurisdictions
```

### Detecting jurisdiction from location

With the `geoclue` feature enabled, `aged` can use
[GeoClue2](https://gitlab.freedesktop.org/geoclue/geoclue/-/wikis/home) to
detect the user's jurisdiction automatically:

```
aged detect-jurisdiction
```

## D-Bus API

Applications can query the daemon directly over the session bus without
going through the CLI.

- **Bus name:** `org.aged.Daemon`
- **Object path:** `/org/aged/Daemon`
- **Interface:** `org.aged.Daemon`

| Method | Signature | Description |
|---|---|---|
| `SetBirthdate` | `(s) → ()` | Store birthdate (`YYYY-MM-DD`) |
| `GetAgeBracket` | `(s) → s` | Get bracket for a jurisdiction (empty string = default) |
| `ListJurisdictions` | `() → as` | List configured jurisdiction names |
| `GetDefaultJurisdiction` | `() → s` | Get stored default jurisdiction |
| `SetDefaultJurisdiction` | `(s) → ()` | Set default jurisdiction |
| `DetectJurisdiction` | `() → s` | Detect jurisdiction from location |

Example with `busctl`:

```
busctl --user call org.aged.Daemon /org/aged/Daemon org.aged.Daemon GetAgeBracket s "US/California"
```

## Configuration

### Daemon configuration

`~/.config/aged/config.toml`:

```toml
[storage]
backend = "secret-service"  # or "file"

[jurisdictions]
extra_paths = []
```

### Jurisdiction definitions

Built-in jurisdictions are compiled into the binary. To add or override
jurisdictions, create `~/.config/aged/jurisdictions.toml`:

```toml
[[jurisdiction]]
name = "US/California"
region = "US-CA"
brackets = [
    { max_age = 13, label = "child" },
    { max_age = 18, label = "minor" },
    { label = "adult" },
]
```

Each bracket has an optional `max_age` (exclusive upper bound). Brackets
are evaluated in order; the first whose `max_age` exceeds the user's age
wins. The last bracket must omit `max_age` as the catch-all.

## Feature flags

| Flag | Default | Description |
|---|---|---|
| `systemd` | Yes | systemd readiness notification via `sd_notify` |
| `geoclue` | No | GeoClue2 location-based jurisdiction detection |

Build with optional features:

```
cargo build --release --features geoclue
```

Build without systemd support:

```
cargo build --release --no-default-features
```

## Building and installing

### From source

```
make
sudo make install
```

This installs to `/usr/local` by default. To change the prefix:

```
make PREFIX=/usr
sudo make install PREFIX=/usr
```

### Building packages

#### Debian / Ubuntu

```
dpkg-buildpackage -us -uc
```

#### RPM (Fedora, openSUSE, etc.)

```
rpmbuild -ba dist/aged.spec
```

#### Arch Linux

Copy `dist/PKGBUILD` into a clean build directory with the source tarball,
then:

```
makepkg
```

### Staging for packaging

All install targets respect `DESTDIR` for staged installs:

```
make install DESTDIR=/tmp/aged-pkg PREFIX=/usr
```

## Testing

```
cargo test
```

## License

MIT — see [LICENSE](LICENSE).

## Contributing

Contributions are welcome. Please note the following:

**AI-written code:** Contributions written or partly written by AI are
welcome but must be disclosed. Please note in your pull request which
portions were AI-generated or AI-assisted. Low-effort drive-by PRs may be
closed without comment.

**Description rewriting:** You are welcome to use AI tools to help rephrase
or clarify your PR descriptions or issue reports if you find that helpful,
but doing so is not required or expected. 
