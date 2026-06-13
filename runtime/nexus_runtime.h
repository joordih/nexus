#ifndef NEXUS_RUNTIME_H
#define NEXUS_RUNTIME_H

#include <gc.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <setjmp.h>

typedef long long NxInt;
typedef int NxBool;
typedef char NxChar;
typedef const char* NxString;

#define NX_TRUE 1
#define NX_FALSE 0

#define NX_LINE_BUF          4096
#define NX_PATH_BUF          4096
#define NX_CMD_BUF           8192
#define NX_FLAG_BUF          1024
#define NX_OUTPUT_PATH_BUF   2048
#define NX_CAPTURE_MAX       65535
#define NX_CAPTURE_ALLOC     (NX_CAPTURE_MAX + 1)
#define NX_INT_STR_BUF       32
#define NX_TRY_STACK_DEPTH   32

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
void nexus_set_args(int argc, char** argv);

NxString nx_read_header_line(void);
NxString nx_read_stdin_n(NxInt n);
void nx_write_stdout_n(NxString s, NxInt n);
NxList* nx_map_keys(NxMap* map);

NxInt nx_tcp_listen(NxInt port);
NxInt nx_tcp_accept(NxInt server_fd);
NxString nx_tcp_read_line(NxInt fd);
NxString nx_tcp_read(NxInt fd, NxInt n);
void nx_tcp_write(NxInt fd, NxString s, NxInt n);
void nx_tcp_close(NxInt fd);

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

NxInt nx_get_arg_count(void);
NxString nx_get_arg(NxInt i);
void nx_write_file(NxString path, NxString content);
void nx_exit(NxInt code);
NxList* nx_list_dir(NxString path);
void nx_compile_c(NxString c_path, NxString output);
NxBool nx_string_ends_with(NxString s, NxString suffix);
NxBool nx_string_lt(NxString a, NxString b);
NxString nx_run_capture(NxString cmd);
NxString nx_run_binary_capture(NxString path);

NxBool nx_file_exists(NxString path);
NxBool nx_mkdir(NxString path);
NxBool nx_delete(NxString path);
NxBool nx_rename(NxString old_path, NxString new_path);
NxInt nx_stat_size(NxString path);
NxBool nx_stat_is_dir(NxString path);
NxInt nx_fopen(NxString path, NxString mode);
void nx_fclose(NxInt handle);
NxString nx_fread_n(NxInt handle, NxInt n);
void nx_fwrite_n(NxInt handle, NxString s, NxInt n);

NxString nx_getenv(NxString name);
NxInt nx_time_now(void);

NxInt nx_tcp_connect(NxString host, NxInt port);

void nx_try_push(jmp_buf buf);
void nx_try_pop(void);
void nx_throw(void);

#endif
