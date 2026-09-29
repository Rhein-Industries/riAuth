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
#include <lasso/id-ff/session.h>
#include <lasso/saml-2.0/saml2_helper.h>
#include <lasso/xml/saml-2.0/saml2_name_id.h>
#include <lasso/xml/saml-2.0/saml2_strings.h>
#include <lasso/xml/saml-2.0/samlp2_authn_request.h>

#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
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
    fprintf(stderr, "usage: lasso-saml-sp request|accept ...\n");
    return 2;
}
