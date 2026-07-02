#include "nexus_runtime.h"
#include "nexus_link_config.h"

#include <openssl/ssl.h>
#include <openssl/err.h>

#include <time.h>
#include <errno.h>
#include <stdint.h>
#include <setjmp.h>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#pragma comment(lib, "ws2_32.lib")
#include <fcntl.h>
#include <io.h>
#include <direct.h>
#include <sys/stat.h>
#else
#include <sys/stat.h>
#include <unistd.h>
#include <dirent.h>
#endif

void nexus_init(void) {
    GC_INIT();
#ifdef _WIN32
    _setmode(_fileno(stdin), _O_BINARY);
    _setmode(_fileno(stdout), _O_BINARY);
#endif
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

NxList* nx_map_keys(NxMap* map) {
    NxList* result = nx_list_new();
    NxInt i = 0;
    while (i < map->bucket_count) {
        NxMapEntry* entry = map->buckets[i];
        while (entry) {
            size_t len = strlen(entry->key);
            char* copy = GC_MALLOC(len + 1);
            memcpy(copy, entry->key, len + 1);
            nx_list_push(result, (void*)copy);
            entry = entry->next;
        }
        i = i + 1;
    }
    return result;
}

NxString nx_read_header_line(void) {
    char* buf = GC_MALLOC(NX_LINE_BUF);
    NxInt pos = 0;
    int c;
    while (pos < NX_LINE_BUF - 1) {
        c = fgetc(stdin);
        if (c == EOF) {
            break;
        }
        if (c == '\r') {
            int next = fgetc(stdin);
            if (next != '\n' && next != EOF) {
                ungetc(next, stdin);
            }
            break;
        }
        if (c == '\n') {
            break;
        }
        buf[pos++] = (char)c;
    }
    buf[pos] = '\0';
    return buf;
}

NxString nx_read_stdin_n(NxInt n) {
    if (n <= 0) {
        return "";
    }
    char* buf = GC_MALLOC((size_t)n + 1);
    size_t total = 0;
    while (total < (size_t)n) {
        size_t got = fread(buf + total, 1, (size_t)n - total, stdin);
        if (got == 0) {
            break;
        }
        total += got;
    }
    buf[total] = '\0';
    return buf;
}

void nx_write_stdout_n(NxString s, NxInt n) {
    if (n <= 0) {
        return;
    }
    fwrite(s, 1, (size_t)n, stdout);
    fflush(stdout);
}

NxString nx_int_to_string(NxInt n) {
    char* buf = GC_MALLOC(NX_INT_STR_BUF);
    snprintf(buf, NX_INT_STR_BUF, "%lld", (long long)n);
    return buf;
}

#define NX_DBL_STR_BUF 64

NxString nx_double_to_string(double d) {
    char* buf = GC_MALLOC(NX_DBL_STR_BUF);
    snprintf(buf, NX_DBL_STR_BUF, "%.15g", d);
    return buf;
}

NxString nx_float_to_string(double f) {
    return nx_double_to_string(f);
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

void nx_println_double(double d) {
    NxString s = nx_double_to_string(d);
    nx_println_string(s);
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

void nx_print_string(NxString s) {
    printf("%s", s);
    fflush(stdout);
}

void nx_print_int(NxInt n) {
    printf("%lld", (long long)n);
    fflush(stdout);
}

NxString nx_read_line(void) {
    char* buf = GC_MALLOC(NX_LINE_BUF);
    if (!fgets(buf, NX_LINE_BUF, stdin)) {
        buf[0] = '\0';
    } else {
        size_t len = strlen(buf);
        if (len > 0 && buf[len - 1] == '\n') buf[len - 1] = '\0';
    }
    return buf;
}

NxString nx_read_file(NxString path) {
    FILE* f = fopen(path, "rb");
    if (!f) return "";
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);
    char* buf = GC_MALLOC(size + 1);
    fread(buf, 1, size, f);
    buf[size] = '\0';
    fclose(f);
    return buf;
}

NxInt nx_string_to_int(NxString s) {
    return (NxInt)atoll(s);
}

NxBool nx_string_equals(NxString a, NxString b) {
    return strcmp(a, b) == 0 ? NX_TRUE : NX_FALSE;
}

NxString nx_string_substring(NxString s, NxInt start, NxInt end_excl) {
    NxInt len = (NxInt)strlen(s);
    if (start < 0) start = 0;
    if (end_excl > len) end_excl = len;
    if (end_excl <= start) return "";
    NxInt n = end_excl - start;
    char* buf = GC_MALLOC(n + 1);
    memcpy(buf, s + start, n);
    buf[n] = '\0';
    return buf;
}

NxBool nx_string_starts_with(NxString s, NxString prefix) {
    return strncmp(s, prefix, strlen(prefix)) == 0 ? NX_TRUE : NX_FALSE;
}

NxBool nx_string_contains(NxString s, NxString sub) {
    return strstr(s, sub) != NULL ? NX_TRUE : NX_FALSE;
}

void nx_panic(NxString msg) {
    fprintf(stderr, "panic: %s\n", msg);
    exit(1);
}

static int _nx_argc = 0;
static char** _nx_argv = NULL;

void nexus_set_args(int argc, char** argv) {
    _nx_argc = argc;
    _nx_argv = argv;
}

NxInt nx_get_arg_count(void) {
    return (NxInt)_nx_argc;
}

NxString nx_get_arg(NxInt i) {
    if (i < 0 || i >= _nx_argc) return "";
    return _nx_argv[i];
}

void nx_write_file(NxString path, NxString content) {
    FILE* f = fopen(path, "wb");
    if (!f) {
        fprintf(stderr, "no se pudo escribir: %s\n", path);
        exit(1);
    }
    fwrite(content, 1, strlen(content), f);
    fclose(f);
}

void nx_exit(NxInt code) {
    exit((int)code);
}

static void nx_copy_path_os(char* dest, size_t dest_sz, NxString src) {
#ifdef _WIN32
    size_t i = 0;
    while (src[i] != '\0' && i + 1 < dest_sz) {
        dest[i] = src[i] == '/' ? '\\' : src[i];
        i++;
    }
    dest[i] = '\0';
#else
    strncpy(dest, src, dest_sz - 1);
    dest[dest_sz - 1] = '\0';
#endif
}

NxString nx_run_capture(NxString cmd) {
#ifdef _WIN32
    FILE* p = _popen(cmd, "r");
#else
    FILE* p = popen(cmd, "r");
#endif
    if (!p) return "";
    char* buf = GC_MALLOC(NX_CAPTURE_ALLOC);
    size_t total = 0;
    while (!feof(p) && total < NX_CAPTURE_MAX) {
        size_t n = fread(buf + total, 1, NX_CAPTURE_MAX - total, p);
        total += n;
    }
    buf[total] = '\0';
#ifdef _WIN32
    _pclose(p);
#else
    pclose(p);
#endif
    return buf;
}

static NxString nx_normalize_capture(NxString buf) {
    if (buf == NULL || buf[0] == '\0') {
        return buf;
    }
    size_t len = strlen(buf);
    char* out = GC_MALLOC(len + 1);
    size_t j = 0;
    size_t i = 0;
    while (i < len) {
        if (buf[i] != '\r') {
            out[j++] = buf[i];
        }
        i++;
    }
    out[j] = '\0';
    return out;
}

NxString nx_run_binary_capture(NxString path) {
    char os_path[NX_PATH_BUF];
    nx_copy_path_os(os_path, sizeof(os_path), path);
    char cmd[NX_CMD_BUF];
    snprintf(cmd, sizeof(cmd), "\"%s\"", os_path);
    NxString out = nx_normalize_capture(nx_run_capture(cmd));
    if (out[0] != '\0') {
        return out;
    }
#ifdef _WIN32
    snprintf(cmd, sizeof(cmd), "\"%s.exe\"", os_path);
    return nx_normalize_capture(nx_run_capture(cmd));
#else
    return out;
#endif
}

NxBool nx_string_lt(NxString a, NxString b) {
    return strcmp(a, b) < 0 ? NX_TRUE : NX_FALSE;
}

NxBool nx_string_ends_with(NxString s, NxString suffix) {
    size_t slen = strlen(s);
    size_t suflen = strlen(suffix);
    if (suflen > slen) return NX_FALSE;
    return strcmp(s + slen - suflen, suffix) == 0 ? NX_TRUE : NX_FALSE;
}

#ifdef _WIN32
NxList* nx_list_dir(NxString path) {
    NxList* result = nx_list_new();
    char os_path[NX_PATH_BUF];
    nx_copy_path_os(os_path, sizeof(os_path), path);
    char pattern[NX_PATH_BUF];
    snprintf(pattern, sizeof(pattern), "%s\\*", os_path);
    WIN32_FIND_DATAA fd;
    HANDLE h = FindFirstFileA(pattern, &fd);
    if (h == INVALID_HANDLE_VALUE) return result;
    do {
        if (strcmp(fd.cFileName, ".") != 0 && strcmp(fd.cFileName, "..") != 0) {
            if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)) {
                size_t len = strlen(fd.cFileName);
                char* copy = GC_MALLOC(len + 1);
                memcpy(copy, fd.cFileName, len + 1);
                nx_list_push(result, (void*)copy);
            }
        }
    } while (FindNextFileA(h, &fd));
    FindClose(h);
    return result;
}
#else
static NxBool nx_dir_entry_is_file(NxString dir, struct dirent* entry) {
    if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) {
        return NX_FALSE;
    }
    if (entry->d_type == DT_REG) {
        return NX_TRUE;
    }
    if (entry->d_type != DT_UNKNOWN) {
        return NX_FALSE;
    }
    char full[NX_PATH_BUF];
    snprintf(full, sizeof(full), "%s/%s", dir, entry->d_name);
    struct stat st;
    if (stat(full, &st) != 0) {
        return NX_FALSE;
    }
    return S_ISREG(st.st_mode) ? NX_TRUE : NX_FALSE;
}

NxList* nx_list_dir(NxString path) {
    NxList* result = nx_list_new();
    DIR* d = opendir(path);
    if (!d) return result;
    struct dirent* entry;
    while ((entry = readdir(d)) != NULL) {
        if (!nx_dir_entry_is_file(path, entry)) {
            continue;
        }
        size_t len = strlen(entry->d_name);
        char* copy = GC_MALLOC(len + 1);
        memcpy(copy, entry->d_name, len + 1);
        nx_list_push(result, (void*)copy);
    }
    closedir(d);
    return result;
}
#endif

NxBool nx_file_exists(NxString path) {
    char os_path[NX_PATH_BUF];
    nx_copy_path_os(os_path, sizeof(os_path), path);
#ifdef _WIN32
    DWORD attr = GetFileAttributesA(os_path);
    return attr != INVALID_FILE_ATTRIBUTES ? NX_TRUE : NX_FALSE;
#else
    struct stat st;
    return stat(os_path, &st) == 0 ? NX_TRUE : NX_FALSE;
#endif
}

NxBool nx_mkdir(NxString path) {
    char os_path[NX_PATH_BUF];
    nx_copy_path_os(os_path, sizeof(os_path), path);
#ifdef _WIN32
    if (CreateDirectoryA(os_path, NULL)) {
        return NX_TRUE;
    }
    return GetLastError() == ERROR_ALREADY_EXISTS ? NX_TRUE : NX_FALSE;
#else
    if (mkdir(os_path, 0755) == 0) {
        return NX_TRUE;
    }
    return errno == EEXIST ? NX_TRUE : NX_FALSE;
#endif
}

NxBool nx_delete(NxString path) {
    char os_path[NX_PATH_BUF];
    nx_copy_path_os(os_path, sizeof(os_path), path);
#ifdef _WIN32
    DWORD attr = GetFileAttributesA(os_path);
    if (attr == INVALID_FILE_ATTRIBUTES) {
        return NX_FALSE;
    }
    if (attr & FILE_ATTRIBUTE_DIRECTORY) {
        return RemoveDirectoryA(os_path) ? NX_TRUE : NX_FALSE;
    }
    return DeleteFileA(os_path) ? NX_TRUE : NX_FALSE;
#else
    struct stat st;
    if (stat(os_path, &st) != 0) {
        return NX_FALSE;
    }
    if (S_ISDIR(st.st_mode)) {
        return rmdir(os_path) == 0 ? NX_TRUE : NX_FALSE;
    }
    return unlink(os_path) == 0 ? NX_TRUE : NX_FALSE;
#endif
}

NxBool nx_rename(NxString old_path, NxString new_path) {
    char os_old[NX_PATH_BUF];
    char os_new[NX_PATH_BUF];
    nx_copy_path_os(os_old, sizeof(os_old), old_path);
    nx_copy_path_os(os_new, sizeof(os_new), new_path);
#ifdef _WIN32
    return MoveFileExA(os_old, os_new, MOVEFILE_REPLACE_EXISTING) ? NX_TRUE : NX_FALSE;
#else
    return rename(os_old, os_new) == 0 ? NX_TRUE : NX_FALSE;
#endif
}

NxInt nx_stat_size(NxString path) {
    char os_path[NX_PATH_BUF];
    nx_copy_path_os(os_path, sizeof(os_path), path);
#ifdef _WIN32
    WIN32_FILE_ATTRIBUTE_DATA data;
    if (!GetFileAttributesExA(os_path, GetFileExInfoStandard, &data)) {
        return -1;
    }
    if (data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
        return -1;
    }
    LARGE_INTEGER size;
    size.HighPart = data.nFileSizeHigh;
    size.LowPart = data.nFileSizeLow;
    return (NxInt)size.QuadPart;
#else
    struct stat st;
    if (stat(os_path, &st) != 0 || S_ISDIR(st.st_mode)) {
        return -1;
    }
    return (NxInt)st.st_size;
#endif
}

NxBool nx_stat_is_dir(NxString path) {
    char os_path[NX_PATH_BUF];
    nx_copy_path_os(os_path, sizeof(os_path), path);
#ifdef _WIN32
    DWORD attr = GetFileAttributesA(os_path);
    if (attr == INVALID_FILE_ATTRIBUTES) {
        return NX_FALSE;
    }
    return (attr & FILE_ATTRIBUTE_DIRECTORY) ? NX_TRUE : NX_FALSE;
#else
    struct stat st;
    if (stat(os_path, &st) != 0) {
        return NX_FALSE;
    }
    return S_ISDIR(st.st_mode) ? NX_TRUE : NX_FALSE;
#endif
}

NxInt nx_fopen(NxString path, NxString mode) {
    char os_path[NX_PATH_BUF];
    nx_copy_path_os(os_path, sizeof(os_path), path);
    FILE* f = fopen(os_path, mode);
    if (!f) {
        return -1;
    }
    return (NxInt)(intptr_t)f;
}

void nx_fclose(NxInt handle) {
    if (handle < 0) {
        return;
    }
    FILE* f = (FILE*)(intptr_t)handle;
    fclose(f);
}

NxString nx_fread_n(NxInt handle, NxInt n) {
    if (handle < 0 || n <= 0) {
        return "";
    }
    FILE* f = (FILE*)(intptr_t)handle;
    char* buf = GC_MALLOC((size_t)n + 1);
    size_t got = fread(buf, 1, (size_t)n, f);
    buf[got] = '\0';
    return buf;
}

void nx_fwrite_n(NxInt handle, NxString s, NxInt n) {
    if (handle < 0 || n <= 0) {
        return;
    }
    FILE* f = (FILE*)(intptr_t)handle;
    fwrite(s, 1, (size_t)n, f);
}

NxString nx_getenv(NxString name) {
    char* v = getenv(name);
    if (!v) {
        return "";
    }
    size_t len = strlen(v);
    char* copy = GC_MALLOC(len + 1);
    memcpy(copy, v, len + 1);
    return copy;
}

NxInt nx_time_now(void) {
    return (NxInt)time(NULL);
}

static jmp_buf _nx_try_bufs[NX_TRY_STACK_DEPTH];
static int _nx_try_depth = 0;

void nx_try_push(jmp_buf buf) {
    if (_nx_try_depth >= NX_TRY_STACK_DEPTH) {
        nx_panic("try stack overflow");
    }
    memcpy(_nx_try_bufs[_nx_try_depth], buf, sizeof(jmp_buf));
    _nx_try_depth++;
}

void nx_try_pop(void) {
    if (_nx_try_depth > 0) {
        _nx_try_depth--;
    }
}

void nx_throw(void) {
    if (_nx_try_depth <= 0) {
        nx_panic("uncaught throw");
    }
    longjmp(_nx_try_bufs[_nx_try_depth - 1], 1);
}

static const char* nx_link_path_or_default(const char* env_value, const char* baked_value) {
    if (env_value && env_value[0] != '\0') {
        return env_value;
    }
    if (baked_value && baked_value[0] != '\0') {
        return baked_value;
    }
    return NULL;
}

static void nx_path_flag(char* buf, size_t buf_size, const char* opt, const char* path) {
    if (!path) {
        buf[0] = '\0';
        return;
    }
    if (strchr(path, ' ') != NULL) {
        snprintf(buf, buf_size, "%s \"%s\"", opt, path);
    } else {
        snprintf(buf, buf_size, "%s %s", opt, path);
    }
}

void nx_compile_c(NxString c_path, NxString output) {
    char cmd[NX_CMD_BUF];
    const char* gc_include = nx_link_path_or_default(getenv("GC_INCLUDE"), NX_DEFAULT_GC_INCLUDE);
    const char* gc_lib = nx_link_path_or_default(getenv("GC_LIB"), NX_DEFAULT_GC_LIB);
    const char* ssl_include = nx_link_path_or_default(getenv("SSL_INCLUDE"), NX_DEFAULT_SSL_INCLUDE);
    const char* ssl_lib = nx_link_path_or_default(getenv("SSL_LIB"), NX_DEFAULT_SSL_LIB);
    const char* cc = getenv("CC");
    if (!cc) cc = "clang";
    char cc_quoted[NX_FLAG_BUF];
    if (strchr(cc, ' ') != NULL) {
        snprintf(cc_quoted, sizeof(cc_quoted), "\"%s\"", cc);
    } else {
        snprintf(cc_quoted, sizeof(cc_quoted), "%s", cc);
    }
    char inc_flag[NX_FLAG_BUF] = "";
    char lib_flag[NX_FLAG_BUF] = "";
    char ssl_inc_flag[NX_FLAG_BUF] = "";
    char ssl_lib_flag[NX_FLAG_BUF] = "";
    nx_path_flag(inc_flag, sizeof(inc_flag), "-I", gc_include);
    nx_path_flag(lib_flag, sizeof(lib_flag), "-L", gc_lib);
    nx_path_flag(ssl_inc_flag, sizeof(ssl_inc_flag), "-I", ssl_include);
    nx_path_flag(ssl_lib_flag, sizeof(ssl_lib_flag), "-L", ssl_lib);
    const char* gc_static = getenv("GC_STATIC");
#ifdef _WIN32
    int use_static_default = 1;
#else
    int use_static_default = 0;
#endif
    int use_static = gc_static ? atoi(gc_static) : use_static_default;
    const char* gc_link = use_static ? "-lgc -DGC_NOT_DLL" : "-lgc";

    char output_path[NX_OUTPUT_PATH_BUF];
#ifdef _WIN32
    size_t out_len = strlen(output);
    if (out_len < 4 || strcmp(output + out_len - 4, ".exe") != 0) {
        snprintf(output_path, sizeof(output_path), "%s.exe", output);
    } else {
        snprintf(output_path, sizeof(output_path), "%s", output);
    }
#else
    snprintf(output_path, sizeof(output_path), "%s", output);
#endif

#ifdef _WIN32
    const char* repro_flag = "-Xlinker /subsystem:console -Wl,/Brepro";
#else
    const char* repro_flag = "-Wl,--build-id=none -Wl,--hash-style=gnu -frandom-seed=nexus -fno-record-gcc-switches -fno-ident";
#endif
#ifdef _WIN32
    const char* ssl_link = "-llibssl -llibcrypto -lcrypt32 -ladvapi32 -luser32 -lws2_32";
#else
    const char* ssl_link = "-lssl -lcrypto";
#endif
    snprintf(cmd, sizeof(cmd),
        "%s %s runtime/nexus_runtime.c -I runtime %s %s %s %s %s %s %s -Wno-return-type -Wno-deprecated-declarations -o %s",
        cc_quoted, c_path, inc_flag, ssl_inc_flag, lib_flag, ssl_lib_flag, gc_link, ssl_link, repro_flag, output_path);
    int ret = system(cmd);
    if (ret != 0) {
        fprintf(stderr, "compilacion C fallo: %s\n", c_path);
        exit(1);
    }
}

#ifdef _WIN32
static NxBool _nx_winsock_ready = NX_FALSE;

static void nx_tcp_ensure_winsock(void) {
    if (_nx_winsock_ready) {
        return;
    }
    WSADATA wsa;
    if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0) {
        nx_panic("WSAStartup fallo");
    }
    _nx_winsock_ready = NX_TRUE;
}

NxInt nx_tcp_listen(NxInt port) {
    nx_tcp_ensure_winsock();
    SOCKET s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (s == INVALID_SOCKET) {
        nx_panic("socket fallo");
    }
    int opt = 1;
    setsockopt(s, SOL_SOCKET, SO_REUSEADDR, (const char*)&opt, sizeof(opt));
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons((unsigned short)port);
    if (bind(s, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
        nx_panic("bind fallo");
    }
    if (listen(s, SOMAXCONN) != 0) {
        nx_panic("listen fallo");
    }
    return (NxInt)s;
}

NxInt nx_tcp_accept(NxInt server_fd) {
    SOCKET client = accept((SOCKET)server_fd, NULL, NULL);
    if (client == INVALID_SOCKET) {
        nx_panic("accept fallo");
    }
    return (NxInt)client;
}

NxString nx_tcp_read_line(NxInt fd) {
    char* buf = GC_MALLOC(NX_LINE_BUF);
    NxInt pos = 0;
    while (pos < NX_LINE_BUF - 1) {
        char ch;
        int got = recv((SOCKET)fd, &ch, 1, 0);
        if (got <= 0) {
            break;
        }
        if (ch == '\r') {
            char next;
            int got2 = recv((SOCKET)fd, &next, 1, 0);
            if (got2 > 0 && next != '\n') {
                buf[pos++] = ch;
                buf[pos++] = next;
            }
            break;
        }
        if (ch == '\n') {
            break;
        }
        buf[pos++] = ch;
    }
    buf[pos] = '\0';
    return buf;
}

NxString nx_tcp_read(NxInt fd, NxInt n) {
    if (n <= 0) {
        return "";
    }
    char* buf = GC_MALLOC((size_t)n + 1);
    NxInt total = 0;
    while (total < n) {
        int got = recv((SOCKET)fd, buf + total, (int)(n - total), 0);
        if (got <= 0) {
            break;
        }
        total += got;
    }
    buf[total] = '\0';
    return buf;
}

void nx_tcp_write(NxInt fd, NxString s, NxInt n) {
    if (n <= 0) {
        return;
    }
    NxInt total = 0;
    while (total < n) {
        int sent = send((SOCKET)fd, s + total, (int)(n - total), 0);
        if (sent <= 0) {
            nx_panic("tcp write fallo");
        }
        total += sent;
    }
}

void nx_tcp_close(NxInt fd) {
    closesocket((SOCKET)fd);
}

NxInt nx_tcp_connect(NxString host, NxInt port) {
    nx_tcp_ensure_winsock();
    struct addrinfo hints;
    struct addrinfo* res = NULL;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    char port_str[16];
    snprintf(port_str, sizeof(port_str), "%d", (int)port);
    if (getaddrinfo(host, port_str, &hints, &res) != 0) {
        return -1;
    }
    SOCKET s = socket(res->ai_family, res->ai_socktype, res->ai_protocol);
    if (s == INVALID_SOCKET) {
        freeaddrinfo(res);
        return -1;
    }
    if (connect(s, res->ai_addr, (int)res->ai_addrlen) != 0) {
        closesocket(s);
        freeaddrinfo(res);
        return -1;
    }
    freeaddrinfo(res);
    return (NxInt)s;
}
#else
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <unistd.h>

NxInt nx_tcp_listen(NxInt port) {
    int s = socket(AF_INET, SOCK_STREAM, 0);
    if (s < 0) {
        nx_panic("socket fallo");
    }
    int opt = 1;
    setsockopt(s, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons((unsigned short)port);
    if (bind(s, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
        nx_panic("bind fallo");
    }
    if (listen(s, SOMAXCONN) != 0) {
        nx_panic("listen fallo");
    }
    return (NxInt)s;
}

NxInt nx_tcp_accept(NxInt server_fd) {
    int client = accept(server_fd, NULL, NULL);
    if (client < 0) {
        nx_panic("accept fallo");
    }
    return (NxInt)client;
}

NxString nx_tcp_read_line(NxInt fd) {
    char* buf = GC_MALLOC(NX_LINE_BUF);
    NxInt pos = 0;
    while (pos < NX_LINE_BUF - 1) {
        char ch;
        ssize_t got = read(fd, &ch, 1);
        if (got <= 0) {
            break;
        }
        if (ch == '\r') {
            char next;
            ssize_t got2 = read(fd, &next, 1);
            if (got2 > 0 && next != '\n') {
                buf[pos++] = ch;
                buf[pos++] = next;
            }
            break;
        }
        if (ch == '\n') {
            break;
        }
        buf[pos++] = ch;
    }
    buf[pos] = '\0';
    return buf;
}

NxString nx_tcp_read(NxInt fd, NxInt n) {
    if (n <= 0) {
        return "";
    }
    char* buf = GC_MALLOC((size_t)n + 1);
    NxInt total = 0;
    while (total < n) {
        ssize_t got = read(fd, buf + total, (size_t)(n - total));
        if (got <= 0) {
            break;
        }
        total += (NxInt)got;
    }
    buf[total] = '\0';
    return buf;
}

void nx_tcp_write(NxInt fd, NxString s, NxInt n) {
    if (n <= 0) {
        return;
    }
    NxInt total = 0;
    while (total < n) {
        ssize_t sent = write(fd, s + total, (size_t)(n - total));
        if (sent <= 0) {
            nx_panic("tcp write fallo");
        }
        total += (NxInt)sent;
    }
}

void nx_tcp_close(NxInt fd) {
    close(fd);
}

NxInt nx_tcp_connect(NxString host, NxInt port) {
    struct addrinfo hints;
    struct addrinfo* res = NULL;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    char port_str[16];
    snprintf(port_str, sizeof(port_str), "%d", (int)port);
    if (getaddrinfo(host, port_str, &hints, &res) != 0) {
        return -1;
    }
    int s = socket(res->ai_family, res->ai_socktype, res->ai_protocol);
    if (s < 0) {
        freeaddrinfo(res);
        return -1;
    }
    if (connect(s, res->ai_addr, res->ai_addrlen) != 0) {
        close(s);
        freeaddrinfo(res);
        return -1;
    }
    freeaddrinfo(res);
    return (NxInt)s;
}
#endif

#define NX_TLS_MAX_CONN 64

typedef struct NxTlsConn {
    SSL* ssl;
    SSL_CTX* ctx;
    NxInt fd;
    NxBool in_use;
} NxTlsConn;

static NxTlsConn _nx_tls_conns[NX_TLS_MAX_CONN];
static NxBool _nx_tls_init_done = NX_FALSE;

static void nx_tls_ensure_init(void) {
    if (_nx_tls_init_done) {
        return;
    }
    OPENSSL_init_ssl(OPENSSL_INIT_LOAD_SSL_STRINGS | OPENSSL_INIT_LOAD_CRYPTO_STRINGS, NULL);
    _nx_tls_init_done = NX_TRUE;
}

void nx_tls_ensure_init_test(void) {
    nx_tls_ensure_init();
}

static NxInt nx_tls_alloc_slot(void) {
    NxInt i = 0;
    while (i < NX_TLS_MAX_CONN) {
        if (!_nx_tls_conns[i].in_use) {
            return i;
        }
        i++;
    }
    return -1;
}

static NxBool nx_tls_handle_valid(NxInt handle) {
    return handle >= 0 && handle < NX_TLS_MAX_CONN && _nx_tls_conns[handle].in_use;
}

NxInt nx_tls_connect(NxString host, NxInt port) {
    nx_tls_ensure_init();
    NxInt slot = nx_tls_alloc_slot();
    if (slot < 0) {
        return -1;
    }
    NxInt fd = nx_tcp_connect(host, port);
    if (fd < 0) {
        return -1;
    }
    SSL_CTX* ctx = SSL_CTX_new(TLS_client_method());
    if (!ctx) {
        nx_tcp_close(fd);
        return -1;
    }
    const char* ca_bundle = getenv("NX_TLS_CA_BUNDLE");
    if (ca_bundle) {
        if (SSL_CTX_load_verify_locations(ctx, ca_bundle, NULL) == 1) {
            SSL_CTX_set_verify(ctx, SSL_VERIFY_PEER, NULL);
        }
    }
    SSL* ssl = SSL_new(ctx);
    if (!ssl) {
        SSL_CTX_free(ctx);
        nx_tcp_close(fd);
        return -1;
    }
    SSL_set_fd(ssl, (int)fd);
    SSL_set_tlsext_host_name(ssl, host);
    if (SSL_connect(ssl) != 1) {
        SSL_free(ssl);
        SSL_CTX_free(ctx);
        nx_tcp_close(fd);
        return -1;
    }
    _nx_tls_conns[slot].ssl = ssl;
    _nx_tls_conns[slot].ctx = ctx;
    _nx_tls_conns[slot].fd = fd;
    _nx_tls_conns[slot].in_use = NX_TRUE;
    return slot;
}

NxString nx_tls_read(NxInt handle, NxInt n) {
    if (n <= 0 || !nx_tls_handle_valid(handle)) {
        return "";
    }
    SSL* ssl = _nx_tls_conns[handle].ssl;
    char* buf = GC_MALLOC((size_t)n + 1);
    NxInt total = 0;
    while (total < n) {
        int got = SSL_read(ssl, buf + total, (int)(n - total));
        if (got <= 0) {
            break;
        }
        total += got;
    }
    buf[total] = '\0';
    return buf;
}

NxString nx_tls_read_line(NxInt handle) {
    char* buf = GC_MALLOC(NX_LINE_BUF);
    NxInt pos = 0;
    if (!nx_tls_handle_valid(handle)) {
        buf[0] = '\0';
        return buf;
    }
    SSL* ssl = _nx_tls_conns[handle].ssl;
    while (pos < NX_LINE_BUF - 1) {
        char ch;
        int got = SSL_read(ssl, &ch, 1);
        if (got <= 0) {
            break;
        }
        if (ch == '\r') {
            char next;
            int got2 = SSL_read(ssl, &next, 1);
            if (got2 > 0 && next != '\n') {
                buf[pos++] = ch;
                buf[pos++] = next;
            }
            break;
        }
        if (ch == '\n') {
            break;
        }
        buf[pos++] = ch;
    }
    buf[pos] = '\0';
    return buf;
}

void nx_tls_write(NxInt handle, NxString s, NxInt n) {
    if (n <= 0 || !nx_tls_handle_valid(handle)) {
        return;
    }
    SSL* ssl = _nx_tls_conns[handle].ssl;
    NxInt total = 0;
    while (total < n) {
        int sent = SSL_write(ssl, s + total, (int)(n - total));
        if (sent <= 0) {
            nx_panic("tls write fallo");
        }
        total += sent;
    }
}

void nx_tls_close(NxInt handle) {
    if (!nx_tls_handle_valid(handle)) {
        return;
    }
    SSL* ssl = _nx_tls_conns[handle].ssl;
    SSL_shutdown(ssl);
    SSL_free(ssl);
    SSL_CTX_free(_nx_tls_conns[handle].ctx);
    nx_tcp_close(_nx_tls_conns[handle].fd);
    _nx_tls_conns[handle].ssl = NULL;
    _nx_tls_conns[handle].ctx = NULL;
    _nx_tls_conns[handle].fd = -1;
    _nx_tls_conns[handle].in_use = NX_FALSE;
}
