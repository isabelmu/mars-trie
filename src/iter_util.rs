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

pub fn common_count<L: Iterator, R: Iterator, F: Fn(L::Item, R::Item) -> bool>(
  l: L, r: R, f: F) -> usize {
    let mut c = 0;
    for ((i, li), ri) in l.enumerate().zip(r) {
        c = i;
        if !f(li, ri) {
            return c;
        }
    }
    c
}

pub fn common_count_eq<L: Iterator, R: Iterator>(l: L, r: R) -> usize 
  where L::Item: PartialEq<R::Item> {
    common_count(l, r, |a, b| a == b)
}

#[cfg(test)]
mod test {
    use super::common_count;

    #[test]
    fn test_common_count() {
        let l = [4, 5, 6];
        let r = [4, 5, 7, 8];
        assert!(2 == common_count(l.iter(), r.iter(), |a, b| a == b));
    }
}

