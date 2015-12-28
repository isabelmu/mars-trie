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

/*
#include <algorithm>
#include <cstring>
#include <sstream>

#include <marisa/grimoire/trie/config.h>
#include <marisa/grimoire/trie/header.h>
#include <marisa/grimoire/trie/key.h>
#include <marisa/grimoire/trie/range.h>
#include <marisa/grimoire/trie/tail.h>
#include <marisa/grimoire/trie/state.h>

#include "marisa-assert.h"

namespace {

void TestConfig() {
  TEST_START();

  marisa::grimoire::trie::Config config;

  ASSERT(config.num_tries() == MARISA_DEFAULT_NUM_TRIES);
  ASSERT(config.tail_mode() == MARISA_DEFAULT_TAIL);
  ASSERT(config.node_order() == MARISA_DEFAULT_ORDER);
  ASSERT(config.cache_level() == MARISA_DEFAULT_CACHE);

  config.parse(10 | MARISA_BINARY_TAIL | MARISA_LABEL_ORDER |
      MARISA_TINY_CACHE);

  ASSERT(config.num_tries() == 10);
  ASSERT(config.tail_mode() == MARISA_BINARY_TAIL);
  ASSERT(config.node_order() == MARISA_LABEL_ORDER);
  ASSERT(config.cache_level() == MARISA_TINY_CACHE);

  config.parse(0);

  ASSERT(config.num_tries() == MARISA_DEFAULT_NUM_TRIES);
  ASSERT(config.tail_mode() == MARISA_DEFAULT_TAIL);
  ASSERT(config.node_order() == MARISA_DEFAULT_ORDER);
  ASSERT(config.cache_level() == MARISA_DEFAULT_CACHE);

  TEST_END();
}

void TestHeader() {
  TEST_START();

  marisa::grimoire::trie::Header header;

  {
    marisa::grimoire::Writer writer;
    writer.open("trie-test.dat");
    header.write(writer);
  }

  {
    marisa::grimoire::Mapper mapper;
    mapper.open("trie-test.dat");
    header.map(mapper);
  }

  {
    marisa::grimoire::Reader reader;
    reader.open("trie-test.dat");
    header.read(reader);
  }

  TEST_END();
}

void TestKey() {
  TEST_START();

  marisa::grimoire::trie::Key key;

  ASSERT(key.ptr() == NULL);
  ASSERT(key.length() == 0);
  ASSERT(key.id() == 0);
  ASSERT(key.terminal() == 0);

  const char *str = "xyz";

  key.set_str(str, 3);
  key.set_weight(10.0F);
  key.set_id(20);


  ASSERT(key.ptr() == str);
  ASSERT(key.length() == 3);
  ASSERT(key[0] == 'x');
  ASSERT(key[1] == 'y');
  ASSERT(key[2] == 'z');
  ASSERT(key.weight() == 10.0F);
  ASSERT(key.id() == 20);

  key.set_terminal(30);
  ASSERT(key.terminal() == 30);

  key.substr(1, 2);

  ASSERT(key.ptr() == str + 1);
  ASSERT(key.length() == 2);
  ASSERT(key[0] == 'y');
  ASSERT(key[1] == 'z');

  marisa::grimoire::trie::Key key2;
  key2.set_str("abc", 3);

  ASSERT(key == key);
  ASSERT(key != key2);
  ASSERT(key > key2);
  ASSERT(key2 < key);

  marisa::grimoire::trie::ReverseKey r_key;

  ASSERT(r_key.ptr() == NULL);
  ASSERT(r_key.length() == 0);
  ASSERT(r_key.id() == 0);
  ASSERT(r_key.terminal() == 0);

  r_key.set_str(str, 3);
  r_key.set_weight(100.0F);
  r_key.set_id(200);

  ASSERT(r_key.ptr() == str);
  ASSERT(r_key.length() == 3);
  ASSERT(r_key[0] == 'z');
  ASSERT(r_key[1] == 'y');
  ASSERT(r_key[2] == 'x');
  ASSERT(r_key.weight() == 100.0F);
  ASSERT(r_key.id() == 200);

  r_key.set_terminal(300);
  ASSERT(r_key.terminal() == 300);

  r_key.substr(1, 2);

  ASSERT(r_key.ptr() == str);
  ASSERT(r_key.length() == 2);
  ASSERT(r_key[0] == 'y');
  ASSERT(r_key[1] == 'x');

  marisa::grimoire::trie::ReverseKey r_key2;
  r_key2.set_str("abc", 3);

  ASSERT(r_key == r_key);
  ASSERT(r_key != r_key2);
  ASSERT(r_key > r_key2);
  ASSERT(r_key2 < r_key);

  TEST_END();
}

void TestRange() {
  TEST_START();

  marisa::grimoire::trie::Range range;

  ASSERT(range.begin() == 0);
  ASSERT(range.end() == 0);
  ASSERT(range.key_pos() == 0);

  range.set_begin(1);
  range.set_end(2);
  range.set_key_pos(3);

  ASSERT(range.begin() == 1);
  ASSERT(range.end() == 2);
  ASSERT(range.key_pos() == 3);

  range = marisa::grimoire::trie::make_range(10, 20, 30);

  ASSERT(range.begin() == 10);
  ASSERT(range.end() == 20);
  ASSERT(range.key_pos() == 30);

  marisa::grimoire::trie::WeightedRange w_range;

  ASSERT(w_range.begin() == 0);
  ASSERT(w_range.end() == 0);
  ASSERT(w_range.key_pos() == 0);
  ASSERT(w_range.weight() == 0.0F);

  w_range.set_begin(10);
  w_range.set_end(20);
  w_range.set_key_pos(30);
  w_range.set_weight(40.0F);

  ASSERT(w_range.begin() == 10);
  ASSERT(w_range.end() == 20);
  ASSERT(w_range.key_pos() == 30);
  ASSERT(w_range.weight() == 40.0F);

  marisa::grimoire::trie::WeightedRange w_range2 =
      marisa::grimoire::trie::make_weighted_range(100, 200, 300, 400.0F);

  ASSERT(w_range2.begin() == 100);
  ASSERT(w_range2.end() == 200);
  ASSERT(w_range2.key_pos() == 300);
  ASSERT(w_range2.weight() == 400.0F);

  ASSERT(w_range < w_range2);
  ASSERT(w_range2 > w_range);

  TEST_END();
}

void TestEntry() {
  TEST_START();

  marisa::grimoire::trie::Entry entry;

  ASSERT(entry.ptr() == NULL);
  ASSERT(entry.length() == 0);
  ASSERT(entry.id() == 0);

  const char *str = "XYZ";

  entry.set_str(str, 3);
  entry.set_id(123);

  ASSERT(entry.ptr() == str);
  ASSERT(entry.length() == 3);
  ASSERT(entry[0] == 'Z');
  ASSERT(entry[1] == 'Y');
  ASSERT(entry[2] == 'X');
  ASSERT(entry.id() == 123);

  TEST_END();
}

void TestHistory() {
  TEST_START();

  marisa::grimoire::trie::History history;

  ASSERT(history.node_id() == 0);
  ASSERT(history.louds_pos() == 0);
  ASSERT(history.key_pos() == 0);
  ASSERT(history.link_id() == MARISA_INVALID_LINK_ID);
  ASSERT(history.key_id() == MARISA_INVALID_KEY_ID);

  history.set_node_id(100);
  history.set_louds_pos(200);
  history.set_key_pos(300);
  history.set_link_id(400);
  history.set_key_id(500);

  ASSERT(history.node_id() == 100);
  ASSERT(history.louds_pos() == 200);
  ASSERT(history.key_pos() == 300);
  ASSERT(history.link_id() == 400);
  ASSERT(history.key_id() == 500);

  TEST_END();
}

void TestState() {
  TEST_START();

  marisa::grimoire::trie::State state;

  ASSERT(state.key_buf().empty());
  ASSERT(state.history().empty());
  ASSERT(state.node_id() == 0);
  ASSERT(state.query_pos() == 0);
  ASSERT(state.history_pos() == 0);
  ASSERT(state.status_code() == marisa::grimoire::trie::MARISA_READY_TO_ALL);

  state.set_node_id(10);
  state.set_query_pos(100);
  state.set_history_pos(1000);
  state.set_status_code(
      marisa::grimoire::trie::MARISA_END_OF_PREDICTIVE_SEARCH);

  ASSERT(state.node_id() == 10);
  ASSERT(state.query_pos() == 100);
  ASSERT(state.history_pos() == 1000);
  ASSERT(state.status_code() ==
      marisa::grimoire::trie::MARISA_END_OF_PREDICTIVE_SEARCH);

  state.lookup_init();
  ASSERT(state.status_code() == marisa::grimoire::trie::MARISA_READY_TO_ALL);

  state.reverse_lookup_init();
  ASSERT(state.status_code() == marisa::grimoire::trie::MARISA_READY_TO_ALL);

  state.common_prefix_search_init();
  ASSERT(state.status_code() ==
      marisa::grimoire::trie::MARISA_READY_TO_COMMON_PREFIX_SEARCH);

  state.predictive_search_init();
  ASSERT(state.status_code() ==
      marisa::grimoire::trie::MARISA_READY_TO_PREDICTIVE_SEARCH);

  TEST_END();
}

}  // namespace

int main() try {
  TestConfig();
  TestHeader();
  TestKey();
  TestRange();
  TestEntry();
  TestTextTail();
  TestBinaryTail();
  TestHistory();
  TestState();

  return 0;
} catch (const marisa::Exception &ex) {
  std::cerr << ex.what() << std::endl;
  throw;
}
*/
