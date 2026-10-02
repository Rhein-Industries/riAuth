/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Loopback GNU Lasso SAML 2 service-provider helper.
 * This is a separate program. Do not link it into the riauth binary.
 *
 * request prints one signed AuthnRequest. accept forces Lasso signature
 * verification, then calls Lasso's audience and Conditions lifetime validators.
 * Recipient is compared here: Lasso 2.9.0 has no Recipient validator on the
 * login path. The NameID is printed only after every check and
 * lasso_login_accept_sso succeed.
 */
#include <lasso/lasso.h>
#include <lasso/id-ff/identity.h>
#include <lasso/id-ff/logout.h>
#include <lasso/id-ff/session.h>
#include <lasso/saml-2.0/saml2_helper.h>
#include <lasso/xml/saml-2.0/saml2_name_id.h>
#include <lasso/xml/saml-2.0/saml2_strings.h>
#include <lasso/xml/saml-2.0/samlp2_authn_request.h>
#include <lasso/xml/saml-2.0/samlp2_logout_request.h>

#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <unistd.h>

static const char *const RELAY = "lasso-relay";

static _Noreturn void fail(const char *fn, int rc) {
    const char *text = lasso_strerror(rc);
    fprintf(stderr, "lasso %s: %s (%d)\n", fn, text != NULL ? text : "unknown", rc);
    exit(1);
}

static void check(const char *fn, int rc) {
    if (rc != 0) {
        fail(fn, rc);
    }
}

static char *read_file(const char *path) {
    FILE *file = fopen(path, "rb");
    long length;
    char *buffer;
    if (file == NULL) {
        fprintf(stderr, "open %s: %s\n", path, strerror(errno));
        exit(1);
    }
    if (fseek(file, 0, SEEK_END) != 0 || (length = ftell(file)) < 0 || length > 1024 * 1024) {
        fprintf(stderr, "read %s: size\n", path);
        exit(1);
    }
    if (fseek(file, 0, SEEK_SET) != 0) {
        fprintf(stderr, "read %s: rewind\n", path);
        exit(1);
    }
    buffer = malloc((size_t)length + 1);
    if (buffer == NULL) {
        fail("malloc", LASSO_ERROR_OUT_OF_MEMORY);
    }
    if (length > 0 && fread(buffer, 1, (size_t)length, file) != (size_t)length) {
        fprintf(stderr, "read %s: short\n", path);
        exit(1);
    }
    buffer[length] = '\0';
    fclose(file);
    return buffer;
}

static void write_private(const char *path, const char *data) {
    int fd = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0600);
    size_t length;
    FILE *file;
    if (fd < 0) {
        fprintf(stderr, "write %s: %s\n", path, strerror(errno));
        exit(1);
    }
    file = fdopen(fd, "wb");
    if (file == NULL) {
        fprintf(stderr, "write %s: %s\n", path, strerror(errno));
        close(fd);
        exit(1);
    }
    length = strlen(data);
    if (fwrite(data, 1, length, file) != length || fclose(file) != 0) {
        fprintf(stderr, "write %s: short\n", path);
        exit(1);
    }
}

static void trim_trailing(char *text) {
    size_t length = strlen(text);
    while (length > 0 &&
           (text[length - 1] == '\n' || text[length - 1] == '\r' || text[length - 1] == ' ')) {
        text[--length] = '\0';
    }
}

static LassoServer *open_server(
    const char *sp_metadata,
    const char *sp_key,
    const char *sp_cert,
    const char *idp_metadata,
    const char *idp_entity) {
    LassoServer *server;
    check("lasso_init", lasso_init());
    /* lasso_set_min_signature_method is declared but not exported by this dylib.
     * The server field below selects RSA-SHA256 for the request we sign. */
    server = lasso_server_new(sp_metadata, sp_key, NULL, sp_cert);
    if (server == NULL) {
        fail("lasso_server_new", LASSO_ERROR_UNDEFINED);
    }
    server->signature_method = LASSO_SIGNATURE_METHOD_RSA_SHA256;
    check(
        "lasso_server_add_provider",
        lasso_server_add_provider(server, LASSO_PROVIDER_ROLE_IDP, idp_metadata, NULL, NULL));
    if (lasso_server_get_provider(server, idp_entity) == NULL) {
        fail("lasso_server_get_provider", LASSO_SERVER_ERROR_PROVIDER_NOT_FOUND);
    }
    return server;
}

static void tune_request(LassoLogin *login, const char *acs) {
    LassoProfile *profile = LASSO_PROFILE(login);
    LassoSamlp2AuthnRequest *request;
    if (!LASSO_IS_SAMLP2_AUTHN_REQUEST(profile->request)) {
        fail("authn_request", LASSO_ERROR_CAST_FAILED);
    }
    request = LASSO_SAMLP2_AUTHN_REQUEST(profile->request);
    g_free(request->ProtocolBinding);
    request->ProtocolBinding = g_strdup(LASSO_SAML2_METADATA_BINDING_POST);
    g_free(request->AssertionConsumerServiceURL);
    request->AssertionConsumerServiceURL = g_strdup(acs);
    request->AssertionConsumerServiceIndex = -1;
    request->AttributeConsumingServiceIndex = -1;
    request->ForceAuthn = FALSE;
    request->IsPassive = FALSE;
    if (request->ProviderName != NULL) {
        g_free(request->ProviderName);
        request->ProviderName = NULL;
    }
    if (request->Subject != NULL) {
        lasso_node_destroy(LASSO_NODE(request->Subject));
        request->Subject = NULL;
    }
    if (request->Conditions != NULL) {
        lasso_node_destroy(LASSO_NODE(request->Conditions));
        request->Conditions = NULL;
    }
    if (request->RequestedAuthnContext != NULL) {
        lasso_node_destroy(LASSO_NODE(request->RequestedAuthnContext));
        request->RequestedAuthnContext = NULL;
    }
    if (request->Scoping != NULL) {
        lasso_node_destroy(LASSO_NODE(request->Scoping));
        request->Scoping = NULL;
    }
    if (request->NameIDPolicy == NULL) {
        request->NameIDPolicy = LASSO_SAMLP2_NAME_ID_POLICY(lasso_samlp2_name_id_policy_new());
    }
    if (request->NameIDPolicy == NULL) {
        fail("name_id_policy", LASSO_ERROR_OUT_OF_MEMORY);
    }
    g_free(request->NameIDPolicy->Format);
    request->NameIDPolicy->Format = g_strdup(LASSO_SAML2_NAME_IDENTIFIER_FORMAT_PERSISTENT);
    if (request->NameIDPolicy->SPNameQualifier != NULL) {
        g_free(request->NameIDPolicy->SPNameQualifier);
        request->NameIDPolicy->SPNameQualifier = NULL;
    }
    request->NameIDPolicy->AllowCreate = TRUE;
    g_free(profile->msg_relayState);
    profile->msg_relayState = g_strdup(RELAY);
}

static void reject_condition(const char *kind, int state) {
    fprintf(stderr, "lasso %s: assertion %s is not valid (%d)\n", kind, kind, state);
    exit(1);
}

/* Lasso parses Recipient and the metadata ACS URL. It does not compare them. */
static void require_helper_recipient(LassoSaml2Assertion *assertion, LassoServer *server) {
    LassoSaml2SubjectConfirmationData *data;
    char *service;
    data = lasso_saml2_assertion_get_subject_confirmation_data(assertion, FALSE);
    service = lasso_provider_get_assertion_consumer_service_url(LASSO_PROVIDER(server), NULL);
    if (service == NULL || service[0] == '\0') {
        g_free(service);
        fprintf(stderr, "helper recipient: SP metadata has no assertion consumer service\n");
        exit(1);
    }
    if (data == NULL || data->Recipient == NULL || strcmp(data->Recipient, service) != 0) {
        g_free(service);
        fprintf(
            stderr,
            "helper recipient: assertion Recipient does not match the SP assertion consumer service\n");
        exit(1);
    }
    g_free(service);
}

/* Signature verification has already succeeded. Audience and Conditions
 * lifetime are Lasso API checks. Recipient is the helper comparison above. */
static void require_assertion_conditions(LassoLogin *login, LassoServer *server) {
    LassoNode *node = lasso_login_get_assertion(login);
    LassoSaml2Assertion *assertion;
    const char *audience;
    LassoSaml2AssertionValidationState audience_state;
    LassoSaml2AssertionValidationState lifetime_state;
    if (!LASSO_IS_SAML2_ASSERTION(node)) {
        fail("lasso_login_get_assertion", LASSO_PROFILE_ERROR_MISSING_ASSERTION);
    }
    assertion = LASSO_SAML2_ASSERTION(node);
    audience = LASSO_PROVIDER(server)->ProviderID;
    if (audience == NULL || audience[0] == '\0') {
        fail("provider_id", LASSO_PROFILE_ERROR_MISSING_SERVER);
    }
    audience_state = lasso_saml2_assertion_validate_audience(assertion, audience);
    if (audience_state != LASSO_SAML2_ASSERTION_VALID) {
        reject_condition("audience", (int)audience_state);
    }
    lifetime_state = lasso_saml2_assertion_validate_time_checks(assertion, 0, 0);
    if (lifetime_state != LASSO_SAML2_ASSERTION_VALID) {
        g_object_unref(node);
        reject_condition("lifetime", (int)lifetime_state);
    }
    require_helper_recipient(assertion, server);
    g_object_unref(node);
}

static void require_line(const char *value, const char *label) {
    if (value == NULL || value[0] == '\0' || strchr(value, '\n') != NULL || strchr(value, '\r') != NULL) {
        fprintf(stderr, "lasso %s: empty or multiline\n", label);
        exit(1);
    }
}

static void request_mode(int argc, char **argv) {
    LassoServer *server;
    LassoLogin *login;
    LassoProfile *profile;
    const char *url;
    const char *query = NULL;
    char *dump;
    if (argc != 9) {
        fprintf(stderr, "usage: lasso-saml-sp request SP_METADATA SP_KEY SP_CERT IDP_METADATA IDP_ENTITY ACS STATE_OUT\n");
        exit(2);
    }
    server = open_server(argv[2], argv[3], argv[4], argv[5], argv[6]);
    login = lasso_login_new(server);
    if (login == NULL) {
        fail("lasso_login_new", LASSO_ERROR_UNDEFINED);
    }
    profile = LASSO_PROFILE(login);
    lasso_profile_set_signature_hint(profile, LASSO_PROFILE_SIGNATURE_HINT_FORCE);
    lasso_profile_set_signature_verify_hint(profile, LASSO_PROFILE_SIGNATURE_VERIFY_HINT_FORCE);
    check(
        "lasso_login_init_authn_request",
        lasso_login_init_authn_request(login, argv[6], LASSO_HTTP_METHOD_REDIRECT));
    tune_request(login, argv[7]);
    check("lasso_login_build_authn_request_msg", lasso_login_build_authn_request_msg(login));
    url = profile->msg_url;
    if (url == NULL || (query = strchr(url, '?')) == NULL || query[1] == '\0') {
        fail("msg_url", LASSO_PROFILE_ERROR_BUILDING_QUERY_FAILED);
    }
    query++;
    require_line(query, "message");
    dump = lasso_login_dump(login);
    if (dump == NULL) {
        fail("lasso_login_dump", LASSO_ERROR_UNDEFINED);
    }
    write_private(argv[8], dump);
    g_free(dump);
    printf("binding: redirect\n");
    printf("relay: %s\n", RELAY);
    printf("message: %s\n", query);
    lasso_login_destroy(login);
    lasso_server_destroy(server);
    lasso_shutdown();
}

static void prime_empty_identity(LassoLogin *login) {
    LassoIdentity *identity = lasso_identity_new();
    char *identity_dump;
    if (identity == NULL) {
        fail("lasso_identity_new", LASSO_ERROR_UNDEFINED);
    }
    identity_dump = lasso_identity_dump(identity);
    if (identity_dump == NULL) {
        fail("lasso_identity_dump", LASSO_ERROR_UNDEFINED);
    }
    check(
        "lasso_profile_set_identity_from_dump",
        lasso_profile_set_identity_from_dump(LASSO_PROFILE(login), identity_dump));
    g_free(identity_dump);
    lasso_node_destroy(LASSO_NODE(identity));
}

static void accept_mode(int argc, char **argv) {
    LassoServer *server;
    LassoLogin *login;
    LassoProfile *profile;
    LassoSaml2NameID *name_id;
    char *dump;
    char *response;
    if (argc != 10) {
        fprintf(
            stderr,
            "usage: lasso-saml-sp accept SP_METADATA SP_KEY SP_CERT IDP_METADATA IDP_ENTITY STATE RESPONSE_B64 RELAY\n");
        exit(2);
    }
    server = open_server(argv[2], argv[3], argv[4], argv[5], argv[6]);
    dump = read_file(argv[7]);
    login = lasso_login_new_from_dump(server, dump);
    free(dump);
    if (login == NULL) {
        fail("lasso_login_new_from_dump", LASSO_ERROR_UNDEFINED);
    }
    profile = LASSO_PROFILE(login);
    lasso_profile_set_signature_hint(profile, LASSO_PROFILE_SIGNATURE_HINT_FORCE);
    lasso_profile_set_signature_verify_hint(profile, LASSO_PROFILE_SIGNATURE_VERIFY_HINT_FORCE);
    prime_empty_identity(login);
    if (profile->session == NULL) {
        profile->session = lasso_session_new();
        if (profile->session == NULL) {
            fail("lasso_session_new", LASSO_ERROR_UNDEFINED);
        }
    }
    /* process_authn_response_msg classifies the whole buffer. A form body is a
     * redirect query and is inflated. HTTP-POST is the SAMLResponse base64 value. */
    response = read_file(argv[8]);
    trim_trailing(response);
    require_line(response, "response");
    if (profile->msg_relayState == NULL || strcmp(profile->msg_relayState, argv[9]) != 0) {
        fprintf(stderr, "lasso relay: stored request relay does not match the POST RelayState\n");
        exit(1);
    }
    check("lasso_login_process_authn_response_msg", lasso_login_process_authn_response_msg(login, response));
    free(response);
    require_assertion_conditions(login, server);
    check("lasso_login_accept_sso", lasso_login_accept_sso(login));
    if (!LASSO_IS_SAML2_NAME_ID(profile->nameIdentifier)) {
        fail("name_identifier", LASSO_ERROR_CAST_FAILED);
    }
    name_id = LASSO_SAML2_NAME_ID(profile->nameIdentifier);
    require_line(name_id->content, "name_id");
    require_line(name_id->Format, "format");
    printf("signature: lasso\n");
    printf("audience: lasso\n");
    printf("lifetime: lasso\n");
    printf("recipient: helper\n");
    printf("name_id: %s\n", name_id->content);
    printf("format: %s\n", name_id->Format);
    lasso_login_destroy(login);
    lasso_server_destroy(server);
    lasso_shutdown();
}

/* New SLO modes use exclusive owned files, finite status output and a real
 * process deadline. The original request/accept modes above are unchanged. */
enum { SLO_FILE_LIMIT = 128 * 1024 };

static _Noreturn void slo_error(const char *stage) {
    fprintf(stderr, "lasso slo: %s\n", stage);
    exit(1);
}

static void slo_limits(void) {
    const struct rlimit output_limit = { SLO_FILE_LIMIT, SLO_FILE_LIMIT };
    if (setrlimit(RLIMIT_FSIZE, &output_limit) != 0 || signal(SIGALRM, SIG_DFL) == SIG_ERR) {
        slo_error("limits");
    }
    alarm(15);
}

static char *slo_read(const char *path) {
    int fd = open(path, O_RDONLY | O_NOFOLLOW | O_NONBLOCK);
    struct stat st;
    char *data;
    size_t used = 0;
    char extra;
    if (fd < 0 || fstat(fd, &st) != 0 || !S_ISREG(st.st_mode) || st.st_uid != geteuid() ||
        (st.st_mode & 0777) != 0600 || st.st_nlink != 1 || st.st_size <= 0 ||
        st.st_size > SLO_FILE_LIMIT) {
        slo_error("private input");
    }
    data = malloc((size_t)st.st_size + 1);
    if (data == NULL) {
        slo_error("allocation");
    }
    while (used < (size_t)st.st_size) {
        ssize_t n = read(fd, data + used, (size_t)st.st_size - used);
        if (n < 0 && errno == EINTR) {
            continue;
        }
        if (n <= 0) {
            slo_error("input length");
        }
        used += (size_t)n;
    }
    if (read(fd, &extra, 1) != 0 || close(fd) != 0 || memchr(data, '\0', used) != NULL) {
        slo_error("input framing");
    }
    data[used] = '\0';
    return data;
}

static void slo_write(const char *path, const char *data) {
    size_t length = data == NULL ? 0 : strnlen(data, SLO_FILE_LIMIT + 1);
    int fd;
    struct stat st;
    size_t used = 0;
    if (length == 0 || length > SLO_FILE_LIMIT) {
        slo_error("output length");
    }
    fd = open(path, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, 0600);
    if (fd < 0 || fchmod(fd, 0600) != 0 || fstat(fd, &st) != 0 || !S_ISREG(st.st_mode) ||
        st.st_uid != geteuid() || (st.st_mode & 0777) != 0600 || st.st_nlink != 1) {
        slo_error("private output");
    }
    while (used < length) {
        ssize_t n = write(fd, data + used, length - used);
        if (n < 0 && errno == EINTR) {
            continue;
        }
        if (n <= 0) {
            slo_error("output write");
        }
        used += (size_t)n;
    }
    if (close(fd) != 0) {
        slo_error("output close");
    }
}

static LassoServer *slo_server(char **argv) {
    int i;
    slo_limits();
    for (i = 2; i <= 5; i++) {
        char *input = slo_read(argv[i]);
        free(input);
    }
    if (strnlen(argv[6], 1025) > 1024) {
        slo_error("provider length");
    }
    return open_server(argv[2], argv[3], argv[4], argv[5], argv[6]);
}

static void slo_accept_state(int argc, char **argv) {
    LassoServer *server;
    LassoLogin *login;
    LassoProfile *profile;
    char *dump;
    char *response;
    char *identity_dump;
    char *session_dump;
    if (argc != 12) {
        slo_error("accept-state arguments");
    }
    server = slo_server(argv);
    dump = slo_read(argv[7]);
    login = lasso_login_new_from_dump(server, dump);
    free(dump);
    if (login == NULL) {
        fail("lasso_login_new_from_dump", LASSO_ERROR_UNDEFINED);
    }
    profile = LASSO_PROFILE(login);
    lasso_profile_set_signature_hint(profile, LASSO_PROFILE_SIGNATURE_HINT_FORCE);
    lasso_profile_set_signature_verify_hint(profile, LASSO_PROFILE_SIGNATURE_VERIFY_HINT_FORCE);
    prime_empty_identity(login);
    if (profile->session == NULL) {
        profile->session = lasso_session_new();
        if (profile->session == NULL) {
            fail("lasso_session_new", LASSO_ERROR_UNDEFINED);
        }
    }
    response = slo_read(argv[8]);
    trim_trailing(response);
    require_line(response, "response");
    if (strnlen(argv[9], 81) > 80 || profile->msg_relayState == NULL ||
        strcmp(profile->msg_relayState, argv[9]) != 0) {
        slo_error("login relay");
    }
    check("lasso_login_process_authn_response_msg", lasso_login_process_authn_response_msg(login, response));
    free(response);
    require_assertion_conditions(login, server);
    check("lasso_login_accept_sso", lasso_login_accept_sso(login));
    if (lasso_profile_get_identity(profile) == NULL || lasso_profile_get_session(profile) == NULL ||
        !LASSO_IS_SAML2_NAME_ID(profile->nameIdentifier) ||
        LASSO_SAML2_NAME_ID(profile->nameIdentifier)->Format == NULL ||
        strcmp(LASSO_SAML2_NAME_ID(profile->nameIdentifier)->Format,
               LASSO_SAML2_NAME_IDENTIFIER_FORMAT_PERSISTENT) != 0) {
        slo_error("accepted state");
    }
    identity_dump = lasso_identity_dump(lasso_profile_get_identity(profile));
    session_dump = lasso_session_dump(lasso_profile_get_session(profile));
    slo_write(argv[10], identity_dump);
    slo_write(argv[11], session_dump);
    g_free(identity_dump);
    g_free(session_dump);
    printf("accepted: true\npersisted: true\n");
    lasso_login_destroy(login);
    lasso_server_destroy(server);
    lasso_shutdown();
}

static LassoLogout *slo_restore(LassoServer *server, const char *identity, const char *session) {
    LassoLogout *logout = lasso_logout_new(server);
    LassoProfile *profile;
    char *dump;
    if (logout == NULL) {
        fail("lasso_logout_new", LASSO_ERROR_UNDEFINED);
    }
    profile = LASSO_PROFILE(logout);
    lasso_profile_set_signature_hint(profile, LASSO_PROFILE_SIGNATURE_HINT_FORCE);
    lasso_profile_set_signature_verify_hint(profile, LASSO_PROFILE_SIGNATURE_VERIFY_HINT_FORCE);
    dump = slo_read(identity);
    check("lasso_profile_set_identity_from_dump", lasso_profile_set_identity_from_dump(profile, dump));
    free(dump);
    dump = slo_read(session);
    check("lasso_profile_set_session_from_dump", lasso_profile_set_session_from_dump(profile, dump));
    free(dump);
    if (profile->session == NULL) {
        slo_error("restored session");
    }
    return logout;
}

static void slo_session_state(int argc, char **argv) {
    LassoServer *server;
    LassoLogout *logout;
    LassoSession *session;
    GList *names;
    GList *indices;
    if (argc != 9) {
        slo_error("session-state arguments");
    }
    server = slo_server(argv);
    logout = slo_restore(server, argv[7], argv[8]);
    session = LASSO_PROFILE(logout)->session;
    names = lasso_session_get_name_ids(session, argv[6]);
    indices = lasso_session_get_session_indexes(session, argv[6], NULL);
    printf("assertion: %s\nnames: %u\nindices: %u\nempty: %s\n",
           lasso_session_get_assertion(session, argv[6]) == NULL ? "absent" : "present",
           g_list_length(names), g_list_length(indices), lasso_session_is_empty(session) ? "true" : "false");
    g_list_free_full(names, g_object_unref);
    g_list_free_full(indices, g_free);
    lasso_logout_destroy(logout);
    lasso_server_destroy(server);
    lasso_shutdown();
}

static void slo_receive_logout(int argc, char **argv) {
    LassoServer *server;
    LassoLogout *logout;
    LassoProfile *profile;
    LassoSamlp2LogoutRequest *request;
    GList *indices;
    char *query;
    char *session_dump;
    if (argc != 12) {
        slo_error("logout arguments");
    }
    server = slo_server(argv);
    logout = slo_restore(server, argv[7], argv[8]);
    profile = LASSO_PROFILE(logout);
    query = slo_read(argv[9]);
    trim_trailing(query);
    require_line(query, "request");
    check("lasso_logout_process_request_msg", lasso_logout_process_request_msg(logout, query));
    free(query);
    check("lasso_profile_get_signature_status", lasso_profile_get_signature_status(profile));
    if (profile->http_request_method != LASSO_HTTP_METHOD_REDIRECT ||
        !LASSO_IS_SAMLP2_LOGOUT_REQUEST(profile->request) || profile->remote_providerID == NULL ||
        strcmp(profile->remote_providerID, argv[6]) != 0) {
        slo_error("signed request binding");
    }
    request = LASSO_SAMLP2_LOGOUT_REQUEST(profile->request);
    if (request->NameID == NULL || request->SessionIndex == NULL ||
        lasso_session_get_assertion(profile->session, argv[6]) == NULL) {
        slo_error("issued session binding");
    }
    indices = lasso_session_get_session_indexes(profile->session, argv[6], LASSO_NODE(request->NameID));
    if (g_list_length(indices) != 1 || strcmp(indices->data, request->SessionIndex) != 0) {
        slo_error("issued index mismatch");
    }
    g_list_free_full(indices, g_free);
    check("lasso_logout_validate_request", lasso_logout_validate_request(logout));
    indices = lasso_session_get_session_indexes(profile->session, argv[6], NULL);
    if (indices != NULL || lasso_session_get_assertion(profile->session, argv[6]) != NULL ||
        !lasso_session_is_empty(profile->session)) {
        slo_error("session retained");
    }
    check("lasso_logout_build_response_msg", lasso_logout_build_response_msg(logout));
    require_line(profile->msg_url, "logout response");
    /* lasso_session_dump deliberately returns an empty string for an empty
     * session. Serialize the actual emptied object with Lasso's generic dump,
     * so the next process can restore and independently inspect that state. */
    session_dump = lasso_node_dump(LASSO_NODE(profile->session));
    slo_write(argv[10], profile->msg_url);
    slo_write(argv[11], session_dump);
    g_free(session_dump);
    printf("signature: lasso\nmatched_indices: 1\nassertion_after: absent\nindices_after: 0\nempty_after: true\n");
    lasso_logout_destroy(logout);
    lasso_server_destroy(server);
    lasso_shutdown();
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: lasso-saml-sp request|accept ...\n");
        return 2;
    }
    if (strcmp(argv[1], "request") == 0) {
        request_mode(argc, argv);
        return 0;
    }
    if (strcmp(argv[1], "accept") == 0) {
        accept_mode(argc, argv);
        return 0;
    }
    if (strcmp(argv[1], "accept-state") == 0) {
        slo_accept_state(argc, argv);
        return 0;
    }
    if (strcmp(argv[1], "session-state") == 0) {
        slo_session_state(argc, argv);
        return 0;
    }
    if (strcmp(argv[1], "logout") == 0) {
        slo_receive_logout(argc, argv);
        return 0;
    }
    fprintf(stderr, "usage: lasso-saml-sp request|accept ...\n");
    return 2;
}
