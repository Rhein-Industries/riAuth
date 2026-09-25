#!/usr/bin/env bash
# Private loopback-only OpenLDAP fixture; does not read or modify a system directory.
set -euo pipefail
riauth_cargo="${CARGO:-cargo}"
riauth_ldap_root="$(mktemp -d "${TMPDIR:-/tmp}/riauth-ldap-test.XXXXXXXX")"
riauth_ldap_pid=""
cleanup() {
  local riauth_ldap_status=$?
  if [[ "$riauth_ldap_status" != 0 && -f "$riauth_ldap_root/slapd.log" ]]; then tail -60 "$riauth_ldap_root/slapd.log"; fi
  if [[ -n "$riauth_ldap_pid" ]]; then kill "$riauth_ldap_pid" 2>/dev/null || true; wait "$riauth_ldap_pid" 2>/dev/null || true; fi
  rm -rf "$riauth_ldap_root"
}
trap cleanup EXIT
riauth_ldap_port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')"
riauth_ldap_url="ldap://127.0.0.1:$riauth_ldap_port"
if [[ "$(uname)" = Darwin ]]; then
  riauth_ldap_prefix="${LDAP_PREFIX:-$(brew --prefix openldap)}"
  riauth_slapd="$riauth_ldap_prefix/libexec/slapd"
  riauth_schema="${LDAP_SCHEMA:-$(brew --prefix)/etc/openldap/schema}"
  riauth_backend=mdb
  riauth_modules=""
else
  # Ubuntu confines /usr/sbin/slapd to system paths with AppArmor. Run a
  # private copy so this disposable fixture can keep all files under /tmp.
  cp "${SLAPD:-/usr/sbin/slapd}" "$riauth_ldap_root/slapd"
  chmod 700 "$riauth_ldap_root/slapd"
  riauth_slapd="$riauth_ldap_root/slapd"
  riauth_schema=/etc/ldap/schema
  riauth_backend=mdb
  riauth_modules=$'modulepath /usr/lib/ldap\nmoduleload back_mdb'
fi
mkdir "$riauth_ldap_root/db"
cat >"$riauth_ldap_root/cert.cnf" <<'CONF'
[req]
distinguished_name = subject
x509_extensions = extensions
prompt = no
[subject]
CN = localhost
[extensions]
basicConstraints = critical,CA:TRUE
subjectAltName = DNS:localhost,IP:127.0.0.1
keyUsage = critical,keyCertSign,cRLSign
CONF
openssl req -x509 -newkey rsa:2048 -nodes -days 1 -config "$riauth_ldap_root/cert.cnf" -keyout "$riauth_ldap_root/ca.key" -out "$riauth_ldap_root/ca.crt" >"$riauth_ldap_root/cert.log" 2>&1
cat >"$riauth_ldap_root/leaf.cnf" <<'CONF'
basicConstraints = critical,CA:FALSE
subjectAltName = DNS:localhost,IP:127.0.0.1
keyUsage = critical,digitalSignature,keyEncipherment
extendedKeyUsage = serverAuth
CONF
openssl req -new -newkey rsa:2048 -nodes -subj /CN=localhost -keyout "$riauth_ldap_root/tls.key" -out "$riauth_ldap_root/tls.csr" >>"$riauth_ldap_root/cert.log" 2>&1
openssl x509 -req -in "$riauth_ldap_root/tls.csr" -CA "$riauth_ldap_root/ca.crt" -CAkey "$riauth_ldap_root/ca.key" -CAcreateserial -days 1 -extfile "$riauth_ldap_root/leaf.cnf" -out "$riauth_ldap_root/tls.crt" >>"$riauth_ldap_root/cert.log" 2>&1
riauth_root_hash="$(/usr/sbin/slappasswd -s fixture-directory-service-password)"
cat >"$riauth_ldap_root/slapd.conf" <<CONF
include $riauth_schema/core.schema
include $riauth_schema/cosine.schema
include $riauth_schema/inetorgperson.schema
pidfile $riauth_ldap_root/slapd.pid
argsfile $riauth_ldap_root/slapd.args
$riauth_modules
TLSCertificateFile $riauth_ldap_root/tls.crt
TLSCertificateKeyFile $riauth_ldap_root/tls.key
TLSCACertificateFile $riauth_ldap_root/ca.crt
database $riauth_backend
suffix "dc=riauth,dc=test"
rootdn "cn=fixture,dc=riauth,dc=test"
rootpw $riauth_root_hash
directory $riauth_ldap_root/db
access to attrs=userPassword by self write by anonymous auth by * none
access to * by * read
CONF
chmod 600 "$riauth_ldap_root/slapd.conf" "$riauth_ldap_root/tls.key"
"$riauth_slapd" -f "$riauth_ldap_root/slapd.conf" -h "$riauth_ldap_url" -d 256 >"$riauth_ldap_root/slapd.log" 2>&1 &
riauth_ldap_pid=$!
for riauth_attempt in {1..50}; do
  if ldapsearch -x -H "$riauth_ldap_url" -s base -b '' '(objectClass=*)' >/dev/null 2>&1; then break; fi
  if ! kill -0 "$riauth_ldap_pid" 2>/dev/null; then cat "$riauth_ldap_root/slapd.log"; exit 1; fi
  sleep 0.1
done
printf 'fixture-directory-service-password\n' >"$riauth_ldap_root/password"
chmod 600 "$riauth_ldap_root/password"
RIAUTH_TEST_LDAP_URL="$riauth_ldap_url" RIAUTH_TEST_LDAP_PASSWORD="$riauth_ldap_root/password" RIAUTH_TEST_LDAP_CA="$riauth_ldap_root/ca.crt" "$riauth_cargo" test --locked --test ldap -- --ignored --nocapture
