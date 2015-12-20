// Copyright (c) 2010-2013, Susumu Yata
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// - Redistributions of source code must retain the above copyright notice, this
//   list of conditions and the following disclaimer.
// - Redistributions in binary form must reproduce the above copyright notice,
//   this list of conditions and the following disclaimer in the documentation
//   and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.


#ifndef MARISA_CMDOPT_H_
#define MARISA_CMDOPT_H_

#ifdef __cplusplus
extern "C" {
#endif

typedef struct cmdopt_option_ {
  // `name' specifies the name of this option.
  // An array of options must be terminated with an option whose name == NULL.
  const char *name;

  // `has_name' specifies whether an option takes an argument or not.
  // 0 specifies that this option does not have any argument.
  // 1 specifies that this option has an argument.
  // 2 specifies that this option may have an argument.
  int  has_arg;

  // `flag' specifies an integer variable which is overwritten by cmdopt_next()
  // with its return value.
  int *flag;

  // `val' specifies a return value of cmdopt_next(). This value is returned
  // when cmdopt_next() finds this option.
  int  val;
} cmdopt_option;

typedef struct cmdopt_t_ {
  // Command line arguments.
  int    argc;
  char **argv;

  // Option settings.
  const cmdopt_option *longopts;
  const char          *optstring;

  int   optind;     // Index of the next argument.
  char *nextchar;   // Next character.
  char *optarg;     // Argument of the last option.
  int   optopt;     // Label of the last option.
  char *optlong;    // Long option.
  int   opterr;     // Warning level (0: nothing, 1: warning, 2: all).
  int   longindex;  // Index of the last long option.
  int   optnum;     // Number of options.
} cmdopt_t;

// cmdopt_init() initializes a cmdopt_t for successive cmdopt_next()s.
void cmdopt_init(cmdopt_t *h, int argc, char **argv,
    const char *optstring, const cmdopt_option *longopts);

// cmdopt_get() analyzes command line arguments and gets the next option.
int cmdopt_get(cmdopt_t *h);

#ifdef  __cplusplus
}  // extern "C"
#endif

#endif  // MARISA_CMDOPT_H_

#include <stdio.h>

#include "cmdopt.h"

#ifdef __cplusplus
extern "C" {
#endif  // __cplusplus

// Moves `optind' to the end and shifts other arguments.
static void cmdopt_shift(cmdopt_t *h) {
  int   i;
  char *tmp;

  tmp = h->argv[h->optind];
  for (i = h->optind; i < h->argc - 1; i++) {
    h->argv[i] = h->argv[i + 1];
  }
  h->argv[i] = tmp;

  h->nextchar = NULL;
  h->optnum--;
}

// Moves to the next argument.
static void cmdopt_next(cmdopt_t *h) {
  h->optind++;
  h->nextchar = NULL;
}

// Checks if the current argument is an option or not.
static int cmdopt_check(cmdopt_t *h) {
  int         ret = 1;
  const char *arg = h->argv[h->optind];

  if (*arg++ != '-') {
    return 0;
  }

  if (*arg == '-') {
    arg++;
    ret++;
  }

  return ret - (*arg == '\0');
}

// Gets an argument of the current option.
static void cmdopt_getopt(cmdopt_t *h) {
  // Moves to the next argument if the current argument has no more characters.
  if (*h->nextchar == '\0') {
    cmdopt_next(h);
    h->nextchar = h->argv[h->optind];
  }

  // Checks whether the current option has an argument or not.
  if (h->optind < h->optnum) {
    h->optarg = h->nextchar;
    cmdopt_next(h);
  } else {
    h->optarg = NULL;
  }
}

// Searches an option.
static int cmdopt_search(cmdopt_t *h) {
  const char *ptr;

  // Updates an option character.
  h->optopt = *h->nextchar++;

  for (ptr = h->optstring; *ptr != '\0'; ptr++) {
    if (*ptr == h->optopt) {
      // Gets an option argument if required.
      if (ptr[1] == ':') {
        cmdopt_getopt(h);

        // Returns ':' if there is no argument.
        if (h->optarg == NULL && ptr[2] != ':') {
          return ':';
        }
      }
      return h->optopt;
    }
  }

  if (h->optopt == '-') {
    cmdopt_next(h);
    while (h->optind < h->optnum) {
      cmdopt_shift(h);
    }
    return -1;
  }

  // Returns '?' if the option character is undefined.
  return '?';
}

// Compares a long option with an argument and returns the length of the
// matched prefix.
static int cmdopt_match_len(const char *opt, const char *arg) {
  int len = 0;

  // Returns 0 if there is a mismatch.
  while ((*arg != '\0') && (*arg != '=')) {
    if (*arg++ != *opt++) {
      return 0;
    }
    len++;
  }

  // Returns a negative value in case of a perfect match.
  if ((*arg == '\0') || (*arg == '=')) {
    return -len;
  }

  return len;
}

// Checks long options.
static int cmdopt_match(cmdopt_t *h) {
  int i, len;
  int max = 0, max_optind = -1;

  // Returns -1 if there are no long options.
  if (h->longopts == NULL) {
    return max_optind;
  }

  for (i = 0; h->longopts[i].name != NULL; i++) {
    len = cmdopt_match_len(h->longopts[i].name, h->nextchar);
    if (len < 0) {
      // In case of a perfect match.
      h->nextchar -= len;
      return i;
    } else if (len > max) {
      // In case of a prefix match.
      max = len;
      max_optind = i;
    } else if (len == max) {
      // There are other candidates.
      max_optind = -1;
    }
  }

  // If there is no perfect match, adopts the longest one.
  h->nextchar += max;
  return max_optind;
}

// Gets an argument of a long option.
static void cmdopt_getopt_long(cmdopt_t *h) {
  if (*h->nextchar == '=') {
    h->optarg = h->nextchar + 1;
    cmdopt_next(h);
  } else {
    cmdopt_next(h);

    // Checks whether there are more options or not.
    if (h->optind < h->optnum) {
      h->optarg = h->argv[h->optind];
      cmdopt_next(h);
    } else {
      h->optarg = NULL;
    }
  }
}

// Searches long options.
static int cmdopt_search_long(cmdopt_t *h) {
  const cmdopt_option *option;

  // Keeps the long option.
  h->optlong = h->argv[h->optind];

  // Gets the next option.
  h->longindex = cmdopt_match(h);
  if (h->longindex  < 0) {
    cmdopt_next(h);
    return '?';
  }

  // Gets an argument if required.
  option = h->longopts + h->longindex;
  if (option->has_arg) {
    cmdopt_getopt_long(h);

    // Return ':' if there are no more arguments.
    if (h->optarg == NULL) {
      return ':';
    }
  } else if (*h->nextchar == '=') {
    // Returns '?' for an extra option argument.
    cmdopt_getopt_long(h);
    return '?';
  }

  // Overwrites a variable if specified in settings.
  if (option->flag != NULL) {
    *option->flag = option->val;
    return 0;
  }

  return option->val;
}

// Analyze command line option.
static int cmdopt_main(cmdopt_t *h) {
  int type;

  // Initializes the internal state.
  h->optopt = 0;
  h->optlong = NULL;
  h->optarg = NULL;
  h->longindex = 0;

  while (h->optind < h->optnum) {
    if (h->nextchar == NULL) {
      // Checks whether the next argument is an option or not.
      type = cmdopt_check(h);
      if (type == 0) {
        cmdopt_shift(h);
      } else {
        h->nextchar = h->argv[h->optind] + type;
        if (type == 2) {
          return cmdopt_search_long(h);
        }
      }
    } else {
      if (*h->nextchar == '\0') {
        cmdopt_next(h);
        continue;
      }
      // Searches an option string.
      return cmdopt_search(h);
    }
  }

  return -1;
}

// cmdopt_init() initializes a cmdopt_t for successive cmdopt_get()s.
void cmdopt_init(cmdopt_t *h, int argc, char **argv,
    const char *optstring, const cmdopt_option *longopts) {
  static const char empty_optstring[] = "";

  h->argc = argc;
  h->argv = argv;
  h->optnum = h->argc;

  h->longopts = longopts;
  h->optstring = (optstring != NULL) ? optstring : empty_optstring;

  h->optind = 1;
  h->nextchar = NULL;
  h->optarg = NULL;
  h->optopt = 0;
  h->optlong = NULL;
  h->opterr = 1;
  h->longindex = 0;
}

// cmdopt_get() analyzes command line arguments and gets the next option.
int cmdopt_get(cmdopt_t *h) {
  int value = cmdopt_main(h);

  // Prints a warning to the standard error stream if enabled.
  if (h->opterr) {
    if (value == ':') {
      // Warning for a lack of an option argument.
      if (h->optlong == NULL) {
        fprintf(stderr, "option requires an argument -- %c\n", h->optopt);
      } else {
        fprintf(stderr, "option `--%s' requires an argument\n",
            h->longopts[h->longindex].name);
      }
    } else if (value == '?') {
      // Warning for an invalid option.
      if (h->optlong == NULL) {
        fprintf(stderr, "invalid option -- %c\n", h->optopt);
      } else {
        fprintf(stderr, "unrecognized option `%s'\n", h->optlong);
      }
    } else if ((value != -1) && (h->opterr == 2)) {
      // Actually this is not for warning, but for debugging.
      if (h->optlong == NULL) {
        fprintf(stderr, "option with `%s' -- %c\n", h->optarg, h->optopt);
      } else {
        fprintf(stderr, "option `--%s' with `%s'\n",
            h->longopts[h->longindex].name, h->optarg);
      }
    }
  }
  return value;
}

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
