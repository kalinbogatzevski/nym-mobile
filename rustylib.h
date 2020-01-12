#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

char *init(const char *id);

void init_release(char *to);

char *start_ws(const char *id);

void start_ws_release(char *to);
