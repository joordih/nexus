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

    printf("runtime OK\n");
    return 0;
}
