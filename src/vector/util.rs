// Copyright (c) 2015, Johannes Muenzel
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

use std::iter;

/// Resize vector, default-initializing any extra elements.
pub fn vec_resize<T: Default + Clone>(v: &mut Vec<T>, new_len: usize) {
    // This should call std::Vec::resize when it is stabilized.
    if v.len() < new_len {
        let len = v.len();
        v.extend(iter::repeat(Default::default()).take(new_len - len));
    } else {
        v.truncate(new_len)
    }
}

#[cfg(test)]
mod test {
    use super::vec_resize;
    use std::iter;
    use quickcheck::quickcheck;

    #[test]
    fn test_resize() {
        fn prop<T: Clone + Default + Eq>(mut v: Vec<T>) -> bool {
            let old_size = v.len();
            let new_size = old_size * 2;

            let old = v.clone();
            if old.len() != old_size { return false; }

            vec_resize(&mut v, new_size);
            if v.len() != new_size { return false; }

            if !old.iter().eq(v.iter().take(old_size)) {
                return false;
            }
            let d: T = Default::default();
            if !iter::repeat(&d).take(old_size).eq(
                  v.iter().skip(old_size)) {
                return false;
            }
            true
        }

        quickcheck(prop::<u32> as fn(Vec<u32>) -> bool)
    }
}

