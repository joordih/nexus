#include "nexus_runtime.h"

#include <openssl/ssl.h>
#include <openssl/x509.h>
#include <openssl/evp.h>
#include <openssl/pem.h>

int main(void) {
    nexus_init();

    void* ptr = GC_MALLOC(1024);
    if (!ptr) {
        fprintf(stderr, "GC_MALLOC FAIL\n");
        return 1;
    }

    NxList* list = nx_list_new();
    NxInt a = 10;
    NxInt b = 20;
    nx_list_push(list, &a);
    nx_list_push(list, &b);
    if (nx_list_len(list) != 2) {
        fprintf(stderr, "list len FAIL\n");
        return 1;
    }

    NxMap* map = nx_map_new();
    NxInt val = 42;
    nx_map_insert(map, "clave", &val);
    NxInt* got = (NxInt*)nx_map_get(map, "clave");
    if (!got || *got != 42) {
        fprintf(stderr, "map get FAIL\n");
        return 1;
    }

    nx_map_insert(map, "alpha", &val);
    NxList* keys = nx_map_keys(map);
    if (nx_list_len(keys) != 2) {
        fprintf(stderr, "map keys len FAIL\n");
        return 1;
    }

    char header_buf[] = "Content-Length: 5\r\n";
    FILE* tmp = tmpfile();
    if (!tmp) {
        fprintf(stderr, "tmpfile FAIL\n");
        return 1;
    }
    fwrite(header_buf, 1, strlen(header_buf), tmp);
    fwrite("hello", 1, 5, tmp);
    fflush(tmp);
    rewind(tmp);

    char line_buf[256];
    int line_pos = 0;
    int ch;
    while (line_pos < 255) {
        ch = fgetc(tmp);
        if (ch == EOF) break;
        if (ch == '\r') {
            int next = fgetc(tmp);
            if (next != '\n' && next != EOF) ungetc(next, tmp);
            break;
        }
        if (ch == '\n') break;
        line_buf[line_pos++] = (char)ch;
    }
    line_buf[line_pos] = '\0';
    if (strcmp(line_buf, "Content-Length: 5") != 0) {
        fprintf(stderr, "header line parse FAIL\n");
        return 1;
    }

    char body_buf[16];
    size_t body_got = fread(body_buf, 1, 5, tmp);
    body_buf[body_got] = '\0';
    if (strcmp(body_buf, "hello") != 0) {
        fprintf(stderr, "body read FAIL\n");
        return 1;
    }
    fclose(tmp);

    if (strcmp(nx_double_to_string(1.5), "1.5") != 0) {
        fprintf(stderr, "double_to_string FAIL\n");
        return 1;
    }

    nx_tls_ensure_init_test();

    EVP_PKEY* pkey = EVP_RSA_gen(2048);
    if (!pkey) { fprintf(stderr, "tls keygen FAIL\n"); return 1; }
    X509* cert = X509_new();
    X509_set_version(cert, 2);
    ASN1_INTEGER_set(X509_get_serialNumber(cert), 1);
    X509_gmtime_adj(X509_getm_notBefore(cert), 0);
    X509_gmtime_adj(X509_getm_notAfter(cert), 31536000L);
    X509_set_pubkey(cert, pkey);
    X509_NAME* name = X509_get_subject_name(cert);
    X509_NAME_add_entry_by_txt(name, "CN", MBSTRING_ASC, (const unsigned char*)"localhost", -1, -1, 0);
    X509_set_issuer_name(cert, name);
    X509_sign(cert, pkey, EVP_sha256());

    SSL_CTX* sctx = SSL_CTX_new(TLS_server_method());
    SSL_CTX_use_certificate(sctx, cert);
    SSL_CTX_use_PrivateKey(sctx, pkey);
    SSL_CTX* cctx = SSL_CTX_new(TLS_client_method());

    SSL* ssrv = SSL_new(sctx);
    SSL* scli = SSL_new(cctx);
    SSL_set_accept_state(ssrv);
    SSL_set_connect_state(scli);

    BIO* cli_rbio; BIO* cli_wbio; BIO* srv_rbio; BIO* srv_wbio;
    BIO_new_bio_pair(&cli_wbio, 0, &srv_rbio, 0);
    BIO_new_bio_pair(&srv_wbio, 0, &cli_rbio, 0);
    SSL_set_bio(scli, cli_rbio, cli_wbio);
    SSL_set_bio(ssrv, srv_rbio, srv_wbio);

    int hs = 0;
    for (int it = 0; it < 50 && hs < 3; it++) {
        if (SSL_do_handshake(scli) == 1) hs |= 1;
        if (SSL_do_handshake(ssrv) == 1) hs |= 2;
    }
    if (hs != 3) { fprintf(stderr, "tls handshake FAIL\n"); return 1; }

    const char* payload = "ping";
    if (SSL_write(scli, payload, 4) != 4) { fprintf(stderr, "tls write FAIL\n"); return 1; }
    char rx[8] = {0};
    int rn = SSL_read(ssrv, rx, 4);
    if (rn != 4 || strncmp(rx, "ping", 4) != 0) { fprintf(stderr, "tls roundtrip FAIL\n"); return 1; }

    SSL_free(scli);
    SSL_free(ssrv);
    SSL_CTX_free(cctx);
    SSL_CTX_free(sctx);
    X509_free(cert);
    EVP_PKEY_free(pkey);

    printf("runtime OK\n");
    return 0;
}
