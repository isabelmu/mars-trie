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

#ifndef MARISA_GRIMOIRE_IO_WRITER_H_
#define MARISA_GRIMOIRE_IO_WRITER_H_

#include <cstdio>
#include <iostream>

#include "marisa/base.h"

namespace marisa {
namespace grimoire {
namespace io {

class Writer {
 public:
  Writer();
  ~Writer();

  void open(const char *filename);
  void open(std::FILE *file);
  void open(int fd);
  void open(std::ostream &stream);

  template <typename T>
  void write(const T &obj) {
    write_data(&obj, sizeof(T));
  }

  template <typename T>
  void write(const T *objs, std::size_t num_objs) {
    MARISA_THROW_IF((objs == NULL) && (num_objs != 0), MARISA_NULL_ERROR);
    MARISA_THROW_IF(num_objs > (MARISA_SIZE_MAX / sizeof(T)),
                    MARISA_SIZE_ERROR);
    write_data(objs, sizeof(T) * num_objs);
  }

  void seek(std::size_t size);

  bool is_open() const;

  void clear();
  void swap(Writer &rhs);

 private:
  std::FILE *file_;
  int fd_;
  std::ostream *stream_;
  bool needs_fclose_;

  void open_(const char *filename);
  void open_(std::FILE *file);
  void open_(int fd);
  void open_(std::ostream &stream);

  void write_data(const void *data, std::size_t size);

  // Disallows copy and assignment.
  Writer(const Writer &);
  Writer &operator=(const Writer &);
};

}  // namespace io
}  // namespace grimoire
}  // namespace marisa

#endif  // MARISA_GRIMOIRE_IO_WRITER_H_

#include <stdio.h>

#ifdef _WIN32
 #include <io.h>
#else  // _WIN32
 #include <unistd.h>
#endif  // _WIN32

#include <limits>

#include "marisa/grimoire/io/writer.h"

namespace marisa {
namespace grimoire {
namespace io {

Writer::Writer()
    : file_(NULL), fd_(-1), stream_(NULL), needs_fclose_(false) {}

Writer::~Writer() {
  if (needs_fclose_) {
    ::fclose(file_);
  }
}

void Writer::open(const char *filename) {
  MARISA_THROW_IF(filename == NULL, MARISA_NULL_ERROR);

  Writer temp;
  temp.open_(filename);
  swap(temp);
}

void Writer::open(std::FILE *file) {
  MARISA_THROW_IF(file == NULL, MARISA_NULL_ERROR);

  Writer temp;
  temp.open_(file);
  swap(temp);
}

void Writer::open(int fd) {
  MARISA_THROW_IF(fd == -1, MARISA_CODE_ERROR);

  Writer temp;
  temp.open_(fd);
  swap(temp);
}

void Writer::open(std::ostream &stream) {
  Writer temp;
  temp.open_(stream);
  swap(temp);
}

void Writer::clear() {
  Writer().swap(*this);
}

void Writer::swap(Writer &rhs) {
  marisa::swap(file_, rhs.file_);
  marisa::swap(fd_, rhs.fd_);
  marisa::swap(stream_, rhs.stream_);
  marisa::swap(needs_fclose_, rhs.needs_fclose_);
}

void Writer::seek(std::size_t size) {
  MARISA_THROW_IF(!is_open(), MARISA_STATE_ERROR);
  if (size == 0) {
    return;
  } else if (size <= 16) {
    const char buf[16] = {};
    write_data(buf, size);
  } else {
    const char buf[1024] = {};
    do {
      const std::size_t count = (size < sizeof(buf)) ? size : sizeof(buf);
      write_data(buf, count);
      size -= count;
    } while (size != 0);
  }
}

bool Writer::is_open() const {
  return (file_ != NULL) || (fd_ != -1) || (stream_ != NULL);
}

void Writer::open_(const char *filename) {
  std::FILE *file = NULL;
#ifdef _MSC_VER
  MARISA_THROW_IF(::fopen_s(&file, filename, "wb") != 0, MARISA_IO_ERROR);
#else  // _MSC_VER
  file = ::fopen(filename, "wb");
  MARISA_THROW_IF(file == NULL, MARISA_IO_ERROR);
#endif  // _MSC_VER
  file_ = file;
  needs_fclose_ = true;
}

void Writer::open_(std::FILE *file) {
  file_ = file;
}

void Writer::open_(int fd) {
  fd_ = fd;
}

void Writer::open_(std::ostream &stream) {
  stream_ = &stream;
}

void Writer::write_data(const void *data, std::size_t size) {
  MARISA_THROW_IF(!is_open(), MARISA_STATE_ERROR);
  if (size == 0) {
    return;
  } else if (fd_ != -1) {
    while (size != 0) {
#ifdef _WIN32
      static const std::size_t CHUNK_SIZE =
          std::numeric_limits<int>::max();
      const unsigned int count = (size < CHUNK_SIZE) ? size : CHUNK_SIZE;
      const int size_written = ::_write(fd_, data, count);
#else  // _WIN32
      static const std::size_t CHUNK_SIZE =
          std::numeric_limits< ::ssize_t>::max();
      const ::size_t count = (size < CHUNK_SIZE) ? size : CHUNK_SIZE;
      const ::ssize_t size_written = ::write(fd_, data, count);
#endif  // _WIN32
      MARISA_THROW_IF(size_written <= 0, MARISA_IO_ERROR);
      data = static_cast<const char *>(data) + size_written;
      size -= size_written;
    }
  } else if (file_ != NULL) {
    MARISA_THROW_IF(::fwrite(data, 1, size, file_) != size, MARISA_IO_ERROR);
    MARISA_THROW_IF(::fflush(file_) != 0, MARISA_IO_ERROR);
  } else if (stream_ != NULL) {
    try {
      MARISA_THROW_IF(!stream_->write(static_cast<const char *>(data), size),
          MARISA_IO_ERROR);
    } catch (const std::ios_base::failure &) {
      MARISA_THROW(MARISA_IO_ERROR, "std::ios_base::failure");
    }
  }
}

}  // namespace io
}  // namespace grimoire
}  // namespace marisa
