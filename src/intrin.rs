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

#ifndef MARISA_GRIMOIRE_INTRIN_H_
#define MARISA_GRIMOIRE_INTRIN_H_

#include "marisa/base.h"

#if defined(__x86_64__) || defined(_M_X64)
 #define MARISA_X64
#elif defined(__i386__) || defined(_M_IX86)
 #define MARISA_X86
#else  // defined(__i386__) || defined(_M_IX86)
 #ifdef MARISA_USE_POPCNT
  #undef MARISA_USE_POPCNT
 #endif  // MARISA_USE_POPCNT
 #ifdef MARISA_USE_SSE4A
  #undef MARISA_USE_SSE4A
 #endif  // MARISA_USE_SSE4A
 #ifdef MARISA_USE_SSE4
  #undef MARISA_USE_SSE4
 #endif  // MARISA_USE_SSE4
 #ifdef MARISA_USE_SSE4_2
  #undef MARISA_USE_SSE4_2
 #endif  // MARISA_USE_SSE4_2
 #ifdef MARISA_USE_SSE4_1
  #undef MARISA_USE_SSE4_1
 #endif  // MARISA_USE_SSE4_1
 #ifdef MARISA_USE_SSSE3
  #undef MARISA_USE_SSSE3
 #endif  // MARISA_USE_SSSE3
 #ifdef MARISA_USE_SSE3
  #undef MARISA_USE_SSE3
 #endif  // MARISA_USE_SSE3
 #ifdef MARISA_USE_SSE2
  #undef MARISA_USE_SSE2
 #endif  // MARISA_USE_SSE2
#endif  // defined(__i386__) || defined(_M_IX86)

#ifdef MARISA_USE_POPCNT
 #ifndef MARISA_USE_SSE3
  #define MARISA_USE_SSE3
 #endif  // MARISA_USE_SSE3
 #ifdef _MSC_VER
  #include <intrin.h>
 #else  // _MSC_VER
  #include <popcntintrin.h>
 #endif  // _MSC_VER
#endif  // MARISA_USE_POPCNT

#ifdef MARISA_USE_SSE4A
 #ifndef MARISA_USE_SSE3
  #define MARISA_USE_SSE3
 #endif  // MARISA_USE_SSE3
 #ifndef MARISA_USE_POPCNT
  #define MARISA_USE_POPCNT
 #endif  // MARISA_USE_POPCNT
#endif  // MARISA_USE_SSE4A

#ifdef MARISA_USE_SSE4
 #ifndef MARISA_USE_SSE4_2
  #define MARISA_USE_SSE4_2
 #endif  // MARISA_USE_SSE4_2
#endif  // MARISA_USE_SSE4

#ifdef MARISA_USE_SSE4_2
 #ifndef MARISA_USE_SSE4_1
  #define MARISA_USE_SSE4_1
 #endif  // MARISA_USE_SSE4_1
 #ifndef MARISA_USE_POPCNT
  #define MARISA_USE_POPCNT
 #endif  // MARISA_USE_POPCNT
#endif  // MARISA_USE_SSE4_2

#ifdef MARISA_USE_SSE4_1
 #ifndef MARISA_USE_SSSE3
  #define MARISA_USE_SSSE3
 #endif  // MARISA_USE_SSSE3
#endif  // MARISA_USE_SSE4_1

#ifdef MARISA_USE_SSSE3
 #ifndef MARISA_USE_SSE3
  #define MARISA_USE_SSE3
 #endif  // MARISA_USE_SSE3
 #ifdef MARISA_X64
  #define MARISA_X64_SSSE3
 #else  // MARISA_X64
  #define MARISA_X86_SSSE3
 #endif  // MAIRSA_X64
 #include <tmmintrin.h>
#endif  // MARISA_USE_SSSE3

#ifdef MARISA_USE_SSE3
 #ifndef MARISA_USE_SSE2
  #define MARISA_USE_SSE2
 #endif  // MARISA_USE_SSE2
#endif  // MARISA_USE_SSE3

#ifdef MARISA_USE_SSE2
 #ifdef MARISA_X64
  #define MARISA_X64_SSE2
 #else  // MARISA_X64
  #define MARISA_X86_SSE2
 #endif  // MAIRSA_X64
 #include <emmintrin.h>
#endif  // MARISA_USE_SSE2

#ifdef _MSC_VER
 #if MARISA_WORD_SIZE == 64
  #include <intrin.h>
  #pragma intrinsic(_BitScanForward64)
 #else  // MARISA_WORD_SIZE == 64
  #include <intrin.h>
  #pragma intrinsic(_BitScanForward)
 #endif  // MARISA_WORD_SIZE == 64
#endif  // _MSC_VER

#endif  // MARISA_GRIMOIRE_INTRIN_H_
