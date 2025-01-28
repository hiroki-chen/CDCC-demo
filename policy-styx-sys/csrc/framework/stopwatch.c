#include <stdint.h>
#include <stdio.h>

#define CLOCK_REALTIME           0
#define CLOCK_MONOTONIC          1
#define CLOCK_PROCESS_CPUTIME_ID 2
#define CLOCK_THREAD_CPUTIME_ID  3

typedef long int time_t;

typedef int clockid_t;

struct timespec {
    time_t tv_sec;
    long tv_nsec;
};

int clock_gettime(clockid_t clock_id, struct timespec *tp);

static inline uint64_t time_get(void) {
	struct timespec ts;
	clock_gettime(CLOCK_MONOTONIC, &ts);
	return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

void pcd_eval_stopwatch_gettime(uint64_t *watch) {
	if(watch)
		*watch = time_get();
}

void pcd_eval_stopwatch_start(uint64_t *watch) {
	if(watch)
		*watch = time_get();
}

extern int printf(const char *fmt, ...);

void pcd_eval_stopwatch_lap(char *label, uint64_t *watch, char print) {
	uint64_t new_time;
	if(watch)
	{
		new_time = time_get();
		if (print)
			printf("%s took %lu\n", label, new_time - *watch);
		*watch = time_get();
	}
}