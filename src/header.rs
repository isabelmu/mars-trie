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

const HEADER_SIZE: usize = 16;

struct Header;

const header: &'static str = "We love Marisa.";

impl Header {

    fn new() -> Header {
        Header
    }

/*
  void map(Mapper &mapper) {
    const char *ptr;
    mapper.map(&ptr, HEADER_SIZE);
    MARISA_THROW_IF(!test_header(ptr), MARISA_FORMAT_ERROR);
  }
  void read(Reader &reader) {
    char buf[HEADER_SIZE];
    reader.read(buf, HEADER_SIZE);
    MARISA_THROW_IF(!test_header(buf), MARISA_FORMAT_ERROR);
  }
  void write(Writer &writer) const {
    writer.write(get_header(), HEADER_SIZE);
  }
*/

    fn io_size() -> usize {
        HEADER_SIZE
    }

    fn get_header() -> &'static str {
        header
    }

    fn test_header<'a>(x: &'a str) -> bool {
        header == x
    }
}

