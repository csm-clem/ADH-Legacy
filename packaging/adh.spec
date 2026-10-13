# Paquet rpm de L'Atlas des harmoniques.
# Construit par tools/release.sh, qui fournit l'arbre installé :
#   rpmbuild --define "_topdir $PWD/dist/rpm" \
#            --define "adh_version 2.0.0" \
#            --define "adh_stage $PWD/dist/stage" \
#            -bb packaging/adh.spec

%global app_id com.lecompositeur.adh

Name:           adh
Version:        %{adh_version}
Release:        1
Summary:        L'Atlas des harmoniques — harmoniques naturelles sur guitare et basse
License:        MIT OR Apache-2.0
URL:            https://lecompositeur.com/adh
BuildArch:      %{_arch}

# Le binaire est statique sur libc ; X11, Wayland et libGL sont chargés à l'exécution.
Requires:       libX11
Requires:       mesa-libGL
Requires:       wayland

%description
L'Atlas des harmoniques indique, sur une photo de l'instrument, quelle note sort
quand on pince une corde en un point précis, à quelle fréquence et à combien de
cents de la note tempérée, pour les onze premiers partiels.

%prep
# Rien à préparer : l'arbre est déjà construit par tools/release.sh.

%install
test -d "%{adh_stage}" || { echo "adh_stage : arbre d'installation absent" >&2; exit 1; }
mkdir -p "%{buildroot}"
cp -a "%{adh_stage}/." "%{buildroot}/"

%files
%license %{_docdir}/adh/LICENSE-MIT
%license %{_docdir}/adh/LICENSE-APACHE
%license %{_docdir}/adh/LICENSE-OFL
%doc %{_docdir}/adh/README.md
%doc %{_docdir}/adh/LICENSE-assets.md
%{_bindir}/adh-app
%{_bindir}/adh-dump
%{_datadir}/applications/%{app_id}.desktop
%{_datadir}/metainfo/%{app_id}.appdata.xml
%{_datadir}/icons/hicolor/256x256/apps/%{app_id}.png
%{_datadir}/icons/hicolor/scalable/apps/%{app_id}.svg
