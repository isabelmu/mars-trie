// Similar error codes were used by exceptions in the C++ 'marisa-trie'. Here
// they can be repurposed as returned error codes.
pub enum ErrorCode {
    OK,
  
    /// An object was not ready for a requested operation. For example, an
    /// attempt to modify a fixed vector.
    State,
  
    /// Bounds means that an operation has tried to access an out of range
    /// address.
    Bounds,
  
    /// Range means that an out of range value has appeared in operation.
    Range,
  
    /// Code means that an undefined code has appeared in operation.
    Code,
  
    /// Size means that a size has exceeded a library limitation.
    Size,
  
    /// Memory means that a memory allocation has failed.
    Memory,
  
    /// IO means that an I/O operation has failed.
    IO,
  
    /// Format means that input was in invalid format.
    Format,
}

