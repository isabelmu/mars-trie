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

