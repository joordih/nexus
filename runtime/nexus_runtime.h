#ifndef NEXUS_RUNTIME_H
#define NEXUS_RUNTIME_H

#include <gc.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef long long NxInt;
typedef int NxBool;
typedef char NxChar;
typedef const char* NxString;

#define NX_TRUE 1
#define NX_FALSE 0

typedef struct NxList {
    void** data;
    NxInt len;
    NxInt cap;
} NxList;

typedef struct NxMapEntry {
    NxString key;
    void* value;
    struct NxMapEntry* next;
} NxMapEntry;

typedef struct NxMap {
    NxMapEntry** buckets;
    NxInt bucket_count;
    NxInt len;
} NxMap;

void nexus_init(void);

NxList* nx_list_new(void);
void nx_list_push(NxList* list, void* elem);
void* nx_list_get(NxList* list, NxInt index);
void nx_list_set(NxList* list, NxInt index, void* elem);
NxInt nx_list_len(NxList* list);

NxMap* nx_map_new(void);
void nx_map_insert(NxMap* map, NxString key, void* value);
void* nx_map_get(NxMap* map, NxString key);
NxBool nx_map_contains(NxMap* map, NxString key);
NxInt nx_map_len(NxMap* map);

NxString nx_int_to_string(NxInt n);
NxString nx_bool_to_string(NxBool b);
NxString nx_char_to_string(NxChar c);
NxString nx_string_concat(NxString a, NxString b);
NxInt nx_string_len(NxString s);
NxChar nx_string_char_at(NxString s, NxInt i);

void nx_println_int(NxInt n);
void nx_println_bool(NxBool b);
void nx_println_char(NxChar c);
void nx_println_string(NxString s);
void nx_print_string(NxString s);
void nx_print_int(NxInt n);

NxString nx_read_line(void);
NxString nx_read_file(NxString path);
NxInt nx_string_to_int(NxString s);
NxBool nx_string_equals(NxString a, NxString b);
NxString nx_string_substring(NxString s, NxInt start, NxInt end_excl);
NxBool nx_string_starts_with(NxString s, NxString prefix);
NxBool nx_string_contains(NxString s, NxString sub);

void nx_panic(NxString msg);

#endif
