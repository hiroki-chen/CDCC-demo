#ifndef _PCD_STOPWATCH_H_
#define _PCD_STOPWATCH_H_

#ifdef PCD_CONFIG_EVAL

#include <stdint.h>

#define PCD_EVAL_DEFINE_WATCH(watch) uint64_t watch

void pcd_eval_stopwatch_gettime(uint64_t *watch);
void pcd_eval_stopwatch_start(uint64_t *watch);
void pcd_eval_stopwatch_lap(char *label, uint64_t *watch, char print);

#else 

#define PCD_EVAL_DEFINE_WATCH(watch)

#define pcd_eval_stopwatch_gettime(watch)
#define pcd_eval_stopwatch_start(watch)
#define pcd_eval_stopwatch_lap(label, watch, print)

#endif

#endif