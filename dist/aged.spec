Name:           aged
Version:        0.1.0 # x-release-please-version
Release:        1%{?dist}
Summary:        Age Bracket Verification Daemon

License:        MIT
URL:            https://github.com/example/aged
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  make

Requires:       dbus

Recommends:     gnome-keyring

%define cargo_profile release-with-debug

%description
aged is a daemon that stores a user's birthdate, defines
jurisdiction-specific age brackets, and exposes a D-Bus (and CLI)
API for applications to query which bracket the user falls into.

%prep
%autosetup

%build
# NOTE: systemd user units must go under prefix/lib, not %%{_libdir}
# which expands to /usr/lib64 on 64-bit RPM distros.
make CARGO_PROFILE=%{cargo_profile} PREFIX=%{_prefix} LIBDIR=%{_prefix}/lib

%install
make install CARGO_PROFILE=%{cargo_profile} DESTDIR=%{buildroot} PREFIX=%{_prefix} LIBDIR=%{_prefix}/lib

%files
%license LICENSE
%{_bindir}/aged
%{_prefix}/lib/systemd/user/aged.service
%{_datadir}/dbus-1/services/org.aged.Daemon.service
%{_datadir}/aged/jurisdictions.toml
%doc %{_datadir}/doc/aged/aged.conf.toml
