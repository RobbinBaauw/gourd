#!/bin/sh
BASEDIR=`dirname "${0}"`
cd "$BASEDIR"


tar -czvf bundle.tar.gz gourd gourd_wrapper manpages/*.man completions/*
script="install-{{triple}}.sh"
uscript="uninstall-{{triple}}.sh"

binary_path=${INSTALL_PATH-"/usr/local/bin"}
man_path=${MANINSTALL_PATH-"/usr/local/share/man"}
fishc_path=${FISHINSTALL_PATH-"/usr/share/fish/completions"}

tmp=__extract__$RANDOM

printf "#!/bin/sh
PAYLOAD_LINE=\`awk '/^__PAYLOAD_BELOW__/ {print NR + 1; exit 0; }' \$0\`

BINARY_PATH=\"\${INSTALL_PATH:-${binary_path}}\"
MAN_PATH=\"\${MANINSTALL_PATH:-${man_path}}\"
FISH_PATH=\"\${FISHINSTALL_PATH:-${fishc_path}}\"

extract() {
  tail -n+\$PAYLOAD_LINE \$0 | tar -Oxzf - \$1 > \$2
  echo \"\$1 --> \$2\"
}

extract_par() {
  mkdir -pv \$3 2> /dev/null
  extract \$1 \$2
}

extract gourd \"\${BINARY_PATH}/gourd\"
chmod +x \"\${BINARY_PATH}/gourd\"

extract gourd_wrapper \"\${BINARY_PATH}/gourd_wrapper\"
chmod +x \"\${BINARY_PATH}/gourd_wrapper\"

# ADD INIT TARBALLS HERE!

extract_par manpages/gourd.1.man \"\${MAN_PATH}/man1/gourd.1\" \"\${MAN_PATH}/man1\"
extract_par manpages/gourd.toml.5.man \"\${MAN_PATH}/man5/gourd.toml.5\" \"\${MAN_PATH}/man5\"
extract_par manpages/gourd-tutorial.7.man \"\${MAN_PATH}/man7/gourd-tutorial.7\" \"\${MAN_PATH}/man7\"

echo \"Installing completions... this can fail and thats fine!\"
extract_par completions/gourd.fish \"\${FISH_PATH}/gourd.fish\" \"\${FISH_PATH}\" || true

exit 0
__PAYLOAD_BELOW__\n" > "$tmp"


cat "$tmp" "bundle.tar.gz" > "$script" && rm "$tmp"
rm bundle.tar.gz


printf "#!/bin/sh
BINARY_PATH=\"\${INSTALL_PATH:-${binary_path}}\"
MAN_PATH=\"\${MANINSTALL_PATH:-${man_path}}\"
FISH_PATH=\"\${FISHINSTALL_PATH:-${fishc_path}}\"

rmm() {
  echo \"delete \$1\"
  rm -v \$1
}

rmm \"\${BINARY_PATH}/gourd\"
rmm \"\${BINARY_PATH}/gourd_wrapper\"

# ADD INIT TARBALLS HERE!

rmm \"\${MAN_PATH}/man1/gourd.1\"
rmm \"\${MAN_PATH}/man5/gourd.toml.5\"
rmm \"\${MAN_PATH}/man7/gourd-tutorial.7\"

echo \"Uninstalling completions... this can fail and thats fine!\"
rmm \"\${FISH_PATH}/gourd.fish\" || true " > "$uscript"


chmod +x "$script"
chmod +x "$uscript"
