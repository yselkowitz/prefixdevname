Name:           prefixdevname
Version:        0.1.0
Release:        1%{?dist}
Summary:        Udev helper utility that provides network interface naming using user defined prefix

License:        MIT
URL:            https://www.github.com/msekletar/prefixdevname
Source0:        https://www.github.com/msekletar/prefixdevname/archive/%{name}-%{version}.tar.xz
Source1:        %{name}-%{version}-vendor.tar.xz

ExclusiveArch: %{rust_arches}

# In order to build on CentOS we need to use Software Collection
# https://www.softwarecollections.org/en/scls/rhscl/rust-toolset-7/
BuildRequires:  rust-toolset-7
BuildRequires:  git
BuildRequires:  systemd-devel

%description
This package provides udev helper utility that tries to consistently name all ethernet NICs using
user defined prefix (e.g. net.ifnames.prefix=net produces NIC names net0, net1, ...). Utility is
called from udev rule and it determines NIC name and writes out configuration file for udev's
net_setup_link built-in (e.g. /etc/systemd/network/71-net-ifnames-prefix-net0.link).

%prep
%autosetup -S git_am
%cargo_prep -V 1

%build
%cargo_build

%install
%make_install

%files
%defattr(-,root,root,-)
%license LICENSE
%doc README.md
%{_prefix}/lib/udev/%{name}
%{_prefix}/lib/udev/rules.d/*.rules
%dir %{_prefix}/lib/dracut/modules/71%{name}
%{_prefix}/lib/dracut/modules/71%{name}/*


%changelog
* Wed Aug 08 2018 Michal Sekletar <msekleta@redhat.com>
- initial package