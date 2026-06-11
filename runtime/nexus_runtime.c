#include "nexus_runtime.h"

void nexus_init(void) {
    GC_INIT();
}

NxList* nx_list_new(void) {
    NxList* list = GC_MALLOC(sizeof(NxList));
    list->cap = 8;
    list->len = 0;
    list->data = GC_MALLOC(sizeof(void*) * list->cap);
    return list;
}

void nx_list_push(NxList* list, void* elem) {
    if (list->len >= list->cap) {
        NxInt new_cap = list->cap * 2;
        void** new_data = GC_MALLOC(sizeof(void*) * new_cap);
        memcpy(new_data, list->data, sizeof(void*) * list->len);
        list->data = new_data;
        list->cap = new_cap;
    }
    list->data[list->len++] = elem;
}

void* nx_list_get(NxList* list, NxInt index) {
    if (index < 0 || index >= list->len) {
        nx_panic("indice fuera de rango");
    }
    return list->data[index];
}

void nx_list_set(NxList* list, NxInt index, void* elem) {
    if (index < 0 || index >= list->len) {
        nx_panic("indice fuera de rango");
    }
    list->data[index] = elem;
}

NxInt nx_list_len(NxList* list) {
    return list->len;
}

static unsigned int nx_map_hash(NxString key, NxInt bucket_count) {
    unsigned int h = 5381;
    const char* p = key;
    while (*p) {
        h = h * 33 + (unsigned char)*p;
        p++;
    }
    return h % (unsigned int)bucket_count;
}

NxMap* nx_map_new(void) {
    NxMap* map = GC_MALLOC(sizeof(NxMap));
    map->bucket_count = 16;
    map->len = 0;
    map->buckets = GC_MALLOC(sizeof(NxMapEntry*) * map->bucket_count);
    memset(map->buckets, 0, sizeof(NxMapEntry*) * map->bucket_count);
    return map;
}

void nx_map_insert(NxMap* map, NxString key, void* value) {
    unsigned int idx = nx_map_hash(key, map->bucket_count);
    NxMapEntry* entry = map->buckets[idx];
    while (entry) {
        if (strcmp(entry->key, key) == 0) {
            entry->value = value;
            return;
        }
        entry = entry->next;
    }
    NxMapEntry* new_entry = GC_MALLOC(sizeof(NxMapEntry));
    new_entry->key = key;
    new_entry->value = value;
    new_entry->next = map->buckets[idx];
    map->buckets[idx] = new_entry;
    map->len++;
}

void* nx_map_get(NxMap* map, NxString key) {
    unsigned int idx = nx_map_hash(key, map->bucket_count);
    NxMapEntry* entry = map->buckets[idx];
    while (entry) {
        if (strcmp(entry->key, key) == 0) {
            return entry->value;
        }
        entry = entry->next;
    }
    return NULL;
}

NxBool nx_map_contains(NxMap* map, NxString key) {
    return nx_map_get(map, key) != NULL ? NX_TRUE : NX_FALSE;
}

NxInt nx_map_len(NxMap* map) {
    return map->len;
}

NxString nx_int_to_string(NxInt n) {
    char* buf = GC_MALLOC(32);
    snprintf(buf, 32, "%lld", (long long)n);
    return buf;
}

NxString nx_bool_to_string(NxBool b) {
    return b ? "true" : "false";
}

NxString nx_char_to_string(NxChar c) {
    char* buf = GC_MALLOC(2);
    buf[0] = c;
    buf[1] = '\0';
    return buf;
}

NxString nx_string_concat(NxString a, NxString b) {
    size_t la = strlen(a);
    size_t lb = strlen(b);
    char* buf = GC_MALLOC(la + lb + 1);
    memcpy(buf, a, la);
    memcpy(buf + la, b, lb);
    buf[la + lb] = '\0';
    return buf;
}

NxInt nx_string_len(NxString s) {
    return (NxInt)strlen(s);
}

NxChar nx_string_char_at(NxString s, NxInt i) {
    return s[i];
}

void nx_println_int(NxInt n) {
    printf("%lld\n", (long long)n);
}

void nx_println_bool(NxBool b) {
    printf("%s\n", b ? "true" : "false");
}

void nx_println_char(NxChar c) {
    printf("%c\n", c);
}

void nx_println_string(NxString s) {
    printf("%s\n", s);
}

void nx_panic(NxString msg) {
    fprintf(stderr, "panic: %s\n", msg);
    exit(1);
}
