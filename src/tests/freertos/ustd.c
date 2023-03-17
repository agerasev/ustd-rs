#include <stdio.h>
#include <stdlib.h>

char __ustd_io_buffer[0x100];

size_t __ustd_io_buffer_size = sizeof(__ustd_io_buffer);

void __ustd_print_buffer() {
    printf("%s", __ustd_io_buffer);
}

void __ustd_panic() {
	abort();
}

void __ustd_exit() {
	exit(0);
}
