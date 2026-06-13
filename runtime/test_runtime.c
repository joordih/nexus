#include "nexus_runtime.h"

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

    printf("runtime OK\n");
    return 0;
}
