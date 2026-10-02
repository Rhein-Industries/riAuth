#!/usr/bin/env python3
"""One synthetic loopback OIDC relying party for the redb recovery drill.

This module starts nothing on import. The caller owns its lifetime and registers
its exact callback using the public management API. Errors are fixed tags: never
format a request URL, response, claim or exception containing private material.
Requires Python 3.11+ and an OpenSSL executable on PATH.
"""

import base64
import hashlib
import hmac
import http.cookiejar
import http.cookies
import http.server
import json
import os
import re
import secrets
import shutil
import subprocess
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path


MAX_BODY = 256 * 1024
MAX_TOKEN = 32 * 1024
REQUEST_TIMEOUT = 5
COOKIE_NAME = "recovery_drill_rp"


class DrillFailure(RuntimeError):
    """A private-material-free failure tag, safe for the parent evidence."""


def require(condition, tag):
    if not condition:
        raise DrillFailure(tag)


def equal(left, right):
    return (isinstance(left, str) and isinstance(right, str)
            and hmac.compare_digest(left.encode(), right.encode()))


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate_json_field")
        result[key] = value
    return result


def json_object(raw):
    try:
        value = json.loads(raw, object_pairs_hook=unique_object)
    except (UnicodeError, ValueError, RecursionError):
        raise DrillFailure("invalid_json_response") from None
    require(isinstance(value, dict), "json_object_required")
    return value


def b64url(raw):
    return base64.urlsafe_b64encode(raw).decode("ascii").rstrip("=")


def decode_b64url(value):
    require(isinstance(value, str) and len(value) <= MAX_TOKEN
            and re.fullmatch(r"[A-Za-z0-9_-]+", value) is not None,
            "invalid_base64url")
    try:
        raw = base64.b64decode(value + "=" * (-len(value) % 4),
                               altchars=b"-_", validate=True)
    except ValueError:
        raise DrillFailure("invalid_base64url") from None
    require(b64url(raw) == value, "noncanonical_base64url")
    return raw


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, new_url):
        return None


def opener(cookie_jar=None):
    handlers = [urllib.request.ProxyHandler({}), NoRedirect()]
    if cookie_jar is not None:
        handlers.append(urllib.request.HTTPCookieProcessor(cookie_jar))
    return urllib.request.build_opener(*handlers)


def request(url, *, method="GET", form=None, bearer=None, agent=None,
            timeout=REQUEST_TIMEOUT):
    headers = {"Accept": "application/json"}
    body = None
    if form is not None:
        headers["Content-Type"] = "application/x-www-form-urlencoded"
        body = urllib.parse.urlencode(form).encode("ascii")
    if bearer is not None:
        require(isinstance(bearer, str) and 0 < len(bearer) <= MAX_TOKEN
                and re.fullmatch(r"[A-Za-z0-9_.-]+", bearer) is not None,
                "invalid_bearer_shape")
        headers["Authorization"] = "Bearer " + bearer
    try:
        client = agent if agent is not None else opener()
        try:
            response = client.open(urllib.request.Request(
                url, data=body, headers=headers, method=method), timeout=timeout)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            raw = response.read(MAX_BODY + 1)
            require(len(raw) <= MAX_BODY, "response_body_limit")
            return response.code, response.headers, raw
    except DrillFailure:
        raise
    except Exception:
        # urllib exceptions can contain the callback query or request headers.
        raise DrillFailure("http_request_failed") from None


def der(tag, body):
    size = len(body)
    length = bytes([size]) if size < 128 else size.to_bytes((size.bit_length() + 7) // 8, "big")
    if size >= 128:
        length = bytes([0x80 | len(length)]) + length
    return bytes([tag]) + length + body


def rsa_public_pem(key):
    modulus = decode_b64url(key.get("n"))
    exponent = decode_b64url(key.get("e"))
    n, e = int.from_bytes(modulus, "big"), int.from_bytes(exponent, "big")
    require(2048 <= n.bit_length() <= 4096 and n % 2 == 1
            and e == 65537, "unsupported_fixture_rsa_key")

    def integer(raw):
        require(raw and raw[0] != 0, "noncanonical_rsa_integer")
        return der(0x02, (b"\0" if raw[0] & 0x80 else b"") + raw)

    rsa = der(0x30, integer(modulus) + integer(exponent))
    # rsaEncryption AlgorithmIdentifier, then the PKCS#1 key in a BIT STRING.
    spki = der(0x30, bytes.fromhex("300d06092a864886f70d0101010500")
               + der(0x03, b"\0" + rsa))
    encoded = base64.b64encode(spki)
    pem = (b"-----BEGIN PUBLIC KEY-----\n"
           + b"\n".join(encoded[i:i + 64] for i in range(0, len(encoded), 64))
           + b"\n-----END PUBLIC KEY-----\n")
    return pem, (n.bit_length() + 7) // 8


def private_file(path, raw):
    descriptor = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    with os.fdopen(descriptor, "wb") as output:
        output.write(raw)


class QuietServer(http.server.HTTPServer):
    def handle_error(self, request, client_address):
        # The default traceback can reveal handler arguments or callback URLs.
        pass


class LocalRelyingParty:
    def __init__(self, workspace, issuer, client_id):
        self.workspace = Path(workspace)
        require(re.fullmatch(r"http://127\.0\.0\.1:[1-9][0-9]{0,4}", issuer) is not None
                and 1 <= urllib.parse.urlsplit(issuer).port <= 65535,
                "loopback_issuer_required")
        self.issuer = issuer
        self.client_id = client_id
        self.lock = threading.Lock()
        self.pending = None
        self.cookie = None
        self.login_subject = None
        self.source_subject = None
        self.callback_failure = None
        self.closed = False
        candidate = shutil.which("openssl")
        require(candidate is not None, "openssl_executable_required")
        self.openssl = Path(candidate).resolve(strict=True)
        self.openssl_environment = os.environ.copy()
        for name in ("OPENSSL_CONF", "OPENSSL_MODULES", "RANDFILE"):
            self.openssl_environment.pop(name, None)
        try:
            result = subprocess.run([str(self.openssl), "version"], capture_output=True,
                                    timeout=REQUEST_TIMEOUT, env=self.openssl_environment)
            self.openssl_version = result.stdout.decode("ascii").strip()
            require(result.returncode == 0 and self.openssl_version.startswith("OpenSSL ")
                    and len(self.openssl_version) <= 256, "openssl_provider_required")
            with self.openssl.open("rb") as binary:
                self.openssl_sha256 = hashlib.file_digest(binary, "sha256").hexdigest()
        except DrillFailure:
            raise
        except Exception:
            raise DrillFailure("openssl_setup_failed") from None

        owner = self

        class Handler(http.server.BaseHTTPRequestHandler):
            timeout = REQUEST_TIMEOUT

            def log_message(self, format, *args):
                pass

            def send_error(self, code, message=None, explain=None):
                self.reply(code, {"accepted": False})

            def reply(self, status, body, cookie=None):
                raw = json.dumps(body).encode("ascii")
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(raw)))
                self.send_header("Cache-Control", "no-store")
                self.send_header("Connection", "close")
                if cookie is not None:
                    self.send_header("Set-Cookie", COOKIE_NAME + "=" + cookie
                                     + "; HttpOnly; SameSite=Lax; Path=/")
                self.end_headers()
                self.close_connection = True
                self.wfile.write(raw)

            def do_GET(self):
                try:
                    target = urllib.parse.urlsplit(self.path)
                    require(len(self.path) <= 8192 and not target.scheme
                            and not target.netloc and not target.fragment
                            and self.headers.get_all("Host") == [owner.authority]
                            and self.headers.get_all("Authorization") is None,
                            "invalid_rp_request")
                    if target.path == "/callback":
                        cookie = owner.callback(target.query)
                        self.reply(200, {"login": True}, cookie)
                    elif target.path == "/protected" and not target.query:
                        cookies = http.cookies.SimpleCookie()
                        cookies.load(self.headers.get("Cookie", ""))
                        with owner.lock:
                            authenticated = (len(cookies) == 1 and COOKIE_NAME in cookies
                                             and owner.login_subject is not None
                                             and equal(cookies[COOKIE_NAME].value, owner.cookie))
                        self.reply(200 if authenticated else 403,
                                   {"authenticated": authenticated})
                    else:
                        self.reply(404, {"accepted": False})
                except Exception as error:
                    with owner.lock:
                        owner.callback_failure = (str(error) if isinstance(error, DrillFailure)
                                                  else "rp_handler_failed")
                    self.reply(400, {"accepted": False})

        self.server = QuietServer(("127.0.0.1", 0), Handler)
        try:
            self.authority = f"127.0.0.1:{self.server.server_port}"
            self.origin = "http://" + self.authority
            self.redirect_uri = self.origin + "/callback"
            self.thread = threading.Thread(target=self.server.serve_forever,
                                           kwargs={"poll_interval": 0.1}, daemon=True)
            self.thread.start()
        except Exception:
            self.server.server_close()
            raise DrillFailure("rp_start_failed") from None

    def provider(self):
        return {"openssl_version": self.openssl_version,
                "openssl_sha256": self.openssl_sha256}

    def discovery(self):
        status, _, raw = request(self.issuer + "/.well-known/openid-configuration")
        require(status == 200, "discovery_http_status")
        document = json_object(raw)
        require(document.get("issuer") == self.issuer, "discovery_issuer_mismatch")
        for field, path in (("authorization_endpoint", "/oauth/authorize"),
                            ("token_endpoint", "/oauth/token"),
                            ("userinfo_endpoint", "/oauth/userinfo"),
                            ("jwks_uri", "/oauth/jwks")):
            require(document.get(field) == self.issuer + path, "discovery_endpoint_mismatch")
        for field, required in (("id_token_signing_alg_values_supported", "RS256"),
                                ("code_challenge_methods_supported", "S256")):
            values = document.get(field)
            require(isinstance(values, list) and required in values
                    and all(isinstance(value, str) for value in values),
                    "discovery_fixture_algorithms_missing")
        return document

    def callback_fields(self, query, state):
        try:
            fields = urllib.parse.parse_qs(query, keep_blank_values=True,
                                          strict_parsing=True, max_num_fields=4)
        except ValueError:
            raise DrillFailure("invalid_callback_query") from None
        require(set(fields) <= {"code", "state", "iss", "session_state"}
                and all(len(values) == 1 for values in fields.values()),
                "invalid_callback_fields")
        require(equal(fields.get("state", [None])[0], state)
                and fields.get("iss") == [self.issuer], "callback_binding_mismatch")
        code = fields.get("code", [None])[0]
        require(isinstance(code, str) and 0 < len(code) <= 4096
                and re.fullmatch(r"[A-Za-z0-9_-]+", code) is not None,
                "invalid_callback_code")
        return code

    def verify_id_token(self, token, access, pending, discovery):
        require(isinstance(token, str) and len(token) <= MAX_TOKEN, "invalid_id_token")
        segments = token.split(".")
        require(len(segments) == 3, "invalid_id_token")
        header = json_object(decode_b64url(segments[0]))
        claims = json_object(decode_b64url(segments[1]))
        signature = decode_b64url(segments[2])
        require(header.get("alg") == "RS256" and header.get("typ") == "JWT"
                and set(header) <= {"alg", "kid", "typ"}
                and isinstance(header.get("kid"), str) and 0 < len(header["kid"]) <= 128,
                "unsupported_id_token_header")
        status, _, raw = request(discovery["jwks_uri"])
        require(status == 200, "jwks_http_status")
        keys = json_object(raw).get("keys")
        require(isinstance(keys, list) and 0 < len(keys) <= 32
                and all(isinstance(key, dict) for key in keys), "invalid_jwks")
        matching = [key for key in keys if key.get("kid") == header["kid"]]
        require(len(matching) == 1, "ambiguous_signing_key")
        key = matching[0]
        require(key.get("kty") == "RSA" and key.get("alg") == "RS256"
                and key.get("use") == "sig"
                and not set(key).intersection({"d", "p", "q", "dp", "dq", "qi", "oth"}),
                "invalid_public_signing_key")
        pem, signature_size = rsa_public_pem(key)
        require(len(signature) == signature_size, "invalid_signature_size")
        with tempfile.TemporaryDirectory(prefix="rp-verify-", dir=self.workspace) as workspace:
            root = Path(workspace)
            public_key, signed, signature_file = root / "public.pem", root / "signed", root / "signature"
            private_file(public_key, pem)
            private_file(signed, (segments[0] + "." + segments[1]).encode("ascii"))
            private_file(signature_file, signature)
            try:
                result = subprocess.run([str(self.openssl), "dgst", "-sha256", "-verify",
                                         str(public_key), "-signature", str(signature_file), str(signed)],
                                        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                        timeout=REQUEST_TIMEOUT, env=self.openssl_environment)
            except Exception:
                raise DrillFailure("signature_verifier_failed") from None
            require(result.returncode == 0, "id_token_signature_rejected")

        at = int(time.time())
        issued, expires = claims.get("iat"), claims.get("exp")
        require(type(issued) is int and type(expires) is int
                and pending["started_at"] - 5 <= issued <= at + 5
                and expires > at and expires > issued, "id_token_time_rejected")
        require(claims.get("iss") == self.issuer and claims.get("aud") == self.client_id
                and equal(claims.get("nonce"), pending["nonce"]), "id_token_binding_mismatch")
        subject = claims.get("sub")
        require(isinstance(subject, str) and 0 < len(subject) <= 512, "id_token_subject_missing")
        require(equal(claims.get("at_hash"), b64url(hashlib.sha256(access.encode("ascii")).digest()[:16])),
                "id_token_access_hash_mismatch")
        return subject

    def callback(self, query):
        with self.lock:
            pending = self.pending
            require(pending is not None, "callback_already_consumed")
            code = self.callback_fields(query, pending["state"])
            self.pending = None
        discovery = pending["discovery"]
        status, _, raw = request(discovery["token_endpoint"], method="POST", form={
            "grant_type": "authorization_code", "client_id": self.client_id,
            "redirect_uri": self.redirect_uri, "code": code,
            "code_verifier": pending["verifier"],
        })
        require(status == 200, "token_exchange_http_status")
        tokens = json_object(raw)
        access = tokens.get("access_token")
        require(isinstance(access, str) and 0 < len(access) <= MAX_TOKEN
                and re.fullmatch(r"[A-Za-z0-9_.-]+", access) is not None
                and tokens.get("token_type") == "Bearer"
                and set(tokens.get("scope", "").split()) == {"openid", "profile"},
                "invalid_token_response")
        subject = self.verify_id_token(tokens.get("id_token"), access, pending, discovery)
        status, _, raw = request(discovery["userinfo_endpoint"], bearer=access)
        require(status == 200 and equal(json_object(raw).get("sub"), subject),
                "userinfo_binding_mismatch")
        cookie = secrets.token_urlsafe(32)
        with self.lock:
            self.cookie = cookie
            self.login_subject = subject
        return cookie

    def login(self, session_file, *, restored=False):
        jar = http.cookiejar.CookieJar()
        try:
            saved = json_object(Path(session_file).read_bytes())
            require(saved.get("issuer") == self.issuer
                    and type(saved.get("expires_at")) is int
                    and saved["expires_at"] > int(time.time())
                    and isinstance(saved.get("token"), str) and 0 < len(saved["token"]) <= MAX_TOKEN
                    and re.fullmatch(r"[A-Za-z0-9_.-]+", saved["token"]) is not None,
                    "invalid_fixture_session")
            pending = {"state": secrets.token_urlsafe(32), "nonce": secrets.token_urlsafe(32),
                       "verifier": secrets.token_urlsafe(48), "started_at": int(time.time()),
                       "discovery": self.discovery()}
            with self.lock:
                require(not self.closed and self.pending is None, "rp_login_unavailable")
                self.pending, self.cookie, self.login_subject = pending, None, None
                self.callback_failure = None
            status, headers, _ = request(pending["discovery"]["authorization_endpoint"],
                                         method="POST", bearer=saved.get("token"), form={
                "response_type": "code", "client_id": self.client_id,
                "redirect_uri": self.redirect_uri, "scope": "openid profile",
                "state": pending["state"], "nonce": pending["nonce"],
                "code_challenge": b64url(hashlib.sha256(pending["verifier"].encode("ascii")).digest()),
                "code_challenge_method": "S256", "decision": "approve",
            })
            require(status == 302, "authorization_http_status")
            locations = headers.get_all("Location", [])
            require(len(locations) == 1 and len(locations[0]) <= 8192, "invalid_authorization_redirect")
            location = urllib.parse.urlsplit(locations[0])
            require(location.scheme == "http" and location.netloc == self.authority
                    and location.path == "/callback" and not location.fragment
                    and urllib.parse.urlunsplit(location) == locations[0],
                    "callback_redirect_mismatch")
            self.callback_fields(location.query, pending["state"])
            # A new request/opener prevents forwarding the IdP session bearer.
            agent = opener(jar)
            status, _, raw = request(locations[0], agent=agent, timeout=30)
            with self.lock:
                failure = self.callback_failure
            if failure is not None:
                raise DrillFailure(failure)
            require(status == 200 and json_object(raw) == {"login": True}, "callback_http_status")
            status, _, raw = request(self.origin + "/protected")
            require(status == 403 and json_object(raw) == {"authenticated": False},
                    "rp_resource_not_protected")
            status, _, raw = request(self.origin + "/protected", agent=agent)
            require(status == 200 and json_object(raw) == {"authenticated": True},
                    "rp_authenticated_resource_failed")
            with self.lock:
                require(self.pending is None and self.cookie is not None
                        and self.login_subject is not None, "rp_login_not_completed")
                if restored:
                    require(equal(self.source_subject, self.login_subject), "restored_subject_changed")
                else:
                    require(self.source_subject is None, "source_login_already_recorded")
                    self.source_subject = self.login_subject
            return {"authorization_http_status": 302, "callback_http_status": 200,
                    "pkce": "S256", "token_exchange_http_status": 200,
                    "signature_algorithm": "RS256", "signature_verified": True,
                    "state_verified": True, "issuer_audience_nonce_time_verified": True,
                    "access_hash_verified": True, "userinfo_subject_verified": True,
                    "unauthenticated_protected_http_status": 403, "protected_http_status": 200,
                    "fresh_rp_session": True, "subject_matches_source": True if restored else None}
        except DrillFailure:
            raise
        except Exception:
            raise DrillFailure("rp_login_failed") from None
        finally:
            jar.clear()
            with self.lock:
                self.pending = self.cookie = self.login_subject = None

    def close(self):
        if self.closed:
            return
        try:
            self.server.shutdown()
        finally:
            self.server.server_close()
            self.thread.join(timeout=1)
            with self.lock:
                self.pending = self.cookie = self.login_subject = self.source_subject = None
                self.closed = True
        require(not self.thread.is_alive(), "rp_cleanup_failed")
