use std::convert::TryInto;
/**
*
* The MD5 Message-Digest Algorithm, written in Rust.
* Implemented by michael dimovich (@mdimovich)
*
* The purpose of this code is to act as a learning resource
* for the md5 message digest algorithm. I wanted to provide
* a straightforward walk-through of the md5 algorithm using
* Rust.
*
* This code was implemented using the original md5 message digest
* RFC, written by R. Rivest in April 1992.
*
* For more information about the md5 algorithm, or the original C
* source code implementation, please reference RFC 1321.
*
* (source: https://datatracker.ietf.org/doc/html/rfc1321)
*
*/

/**
* This function is specified in the rfc as a way to generate the
* table that we use in rounds 1-4 as part of the hashing operation.
* The function is specified as:
* K[i] = floor(2^32 * abs(sin(i+1)))
*/
fn table_construction_function(i: u32) -> u32 {
    let x: f64 = i as f64;
    let sin_eval = x.sin().abs();
    // note: 4294967296 == 2^32
    return (4294967296.0 * sin_eval) as u32;
}

/**
* 4 auxiliary functions that take 3 32-bit
* integers as input and return 1 32 bit integer as output.
*
* these bitwise operations are the crux of the md5 hashing
* algorithm, and they are performed a number of times to end
* up with our final result.
*/
fn f(x: u32, y: u32, z: u32) -> u32 {
    x & y | !x & z
}

fn g(x: u32, y: u32, z: u32) -> u32 {
    x & z | y & !z
}

fn h(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

fn i(x: u32, y: u32, z: u32) -> u32 {
    y ^ (x | !z)
}

// utility function to convert a vector of type T with size N into an array of type T with size N.
fn vec_to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|_v: Vec<T>| panic!("error converting vector to array - sizes don't match"))
}

/**
* Round 1 function is specified as 16 operations using the following:
* Let [abcd, k s i] denote the operation -
* a = b + ((a + f(b,c,d) + x[k] + table[i]) <<< s);
* where x[k] is a 32 bit chunk of our original message,
* table[i] is an entry generated by our sine function, s is an unsigned
* integer, and a,b,c,d are our 32 bit registers we perform operations on.
*/
fn round_one_operations(
    mut a: u32,
    mut b: u32,
    mut c: u32,
    mut d: u32,
    table: &Vec<u32>,
    x: &Vec<u32>,
) -> [u32; 4] {
    macro_rules! round1 {
        ( $a:ident, $b:ident, $c:ident, $d:ident, $k:expr, $s:expr, $i: expr ) => {
            $a = $b.wrapping_add(
                ($a.wrapping_add(f($b, $c, $d))
                    .wrapping_add(x[$k])
                    .wrapping_add(table[$i]))
                .rotate_left($s),
            )
        };
    }

    round1!(a, b, c, d, 0, 7, 1);
    round1!(d, a, b, c, 1, 12, 2);
    round1!(c, d, a, b, 2, 17, 3);
    round1!(b, c, d, a, 3, 22, 4);

    round1!(a, b, c, d, 4, 7, 5);
    round1!(d, a, b, c, 5, 12, 6);
    round1!(c, d, a, b, 6, 17, 7);
    round1!(b, c, d, a, 7, 22, 8);

    round1!(a, b, c, d, 8, 7, 9);
    round1!(d, a, b, c, 9, 12, 10);
    round1!(c, d, a, b, 10, 17, 11);
    round1!(b, c, d, a, 11, 22, 12);

    round1!(a, b, c, d, 12, 7, 13);
    round1!(d, a, b, c, 13, 12, 14);
    round1!(c, d, a, b, 14, 17, 15);
    round1!(b, c, d, a, 15, 22, 16);

    return [a, b, c, d];
}

/**
* Round 2 is equivalent to round 1, except we swap out our f function for g as such:
* a = b + ((a + g(b,c,d) + x[k] + table[i]) <<< s);
*/
fn round_two_operations(
    mut a: u32,
    mut b: u32,
    mut c: u32,
    mut d: u32,
    table: &Vec<u32>,
    x: &Vec<u32>,
) -> [u32; 4] {
    macro_rules! round2 {
        ( $a:ident, $b:ident, $c:ident, $d:ident, $k:expr, $s:expr, $i:expr) => {
            $a = $b.wrapping_add(
                ($a.wrapping_add(g($b, $c, $d))
                    .wrapping_add(x[$k])
                    .wrapping_add(table[$i]))
                .rotate_left($s),
            )
        };
    }

    round2!(a, b, c, d, 1, 5, 17);
    round2!(d, a, b, c, 6, 9, 18);
    round2!(c, d, a, b, 11, 14, 19);
    round2!(b, c, d, a, 0, 20, 20);

    round2!(a, b, c, d, 5, 5, 21);
    round2!(d, a, b, c, 10, 9, 22);
    round2!(c, d, a, b, 15, 14, 23);
    round2!(b, c, d, a, 4, 20, 24);

    round2!(a, b, c, d, 9, 5, 25);
    round2!(d, a, b, c, 14, 9, 26);
    round2!(c, d, a, b, 3, 14, 27);
    round2!(b, c, d, a, 8, 20, 28);

    round2!(a, b, c, d, 13, 5, 29);
    round2!(d, a, b, c, 2, 9, 30);
    round2!(c, d, a, b, 7, 14, 31);
    round2!(b, c, d, a, 12, 20, 32);

    return [a, b, c, d];
}

/**
* Round 3 is equivalent to round 1, except we swap out our f function for h as such:
* a = b + ((a + h(b,c,d) + x[k] + table[i]) <<< s);
*/
fn round_three_operations(
    mut a: u32,
    mut b: u32,
    mut c: u32,
    mut d: u32,
    table: &Vec<u32>,
    x: &Vec<u32>,
) -> [u32; 4] {
    macro_rules! round3 {
        ( $a:ident, $b:ident, $c:ident, $d:ident, $k:expr, $s:expr, $i:expr  ) => {
            $a = $b.wrapping_add(
                ($a.wrapping_add(h($b, $c, $d))
                    .wrapping_add(x[$k])
                    .wrapping_add(table[$i]))
                .rotate_left($s),
            )
        };
    }

    round3!(a, b, c, d, 5, 4, 33);
    round3!(d, a, b, c, 8, 11, 34);
    round3!(c, d, a, b, 11, 16, 35);
    round3!(b, c, d, a, 14, 23, 36);

    round3!(a, b, c, d, 1, 4, 37);
    round3!(d, a, b, c, 4, 11, 38);
    round3!(c, d, a, b, 7, 16, 39);
    round3!(b, c, d, a, 10, 23, 40);

    round3!(a, b, c, d, 13, 4, 41);
    round3!(d, a, b, c, 0, 11, 42);
    round3!(c, d, a, b, 3, 16, 43);
    round3!(b, c, d, a, 6, 23, 44);

    round3!(a, b, c, d, 9, 4, 45);
    round3!(d, a, b, c, 12, 11, 46);
    round3!(c, d, a, b, 15, 16, 47);
    round3!(b, c, d, a, 2, 23, 48);

    return [a, b, c, d];
}

/**
* Round 4 is equivalent to round 1, except we swap out our f function for h as such:
* a = b + ((a + i(b,c,d) + x[k] + table[i]) <<< s);
*/
fn round_four_operations(
    mut a: u32,
    mut b: u32,
    mut c: u32,
    mut d: u32,
    table: &Vec<u32>,
    x: &Vec<u32>,
) -> [u32; 4] {
    macro_rules! round4 {
        ( $a:ident, $b:ident, $c:ident, $d:ident, $k:expr, $s:expr, $i:expr ) => {
            $a = $b.wrapping_add(
                ($a.wrapping_add(i($b, $c, $d))
                    .wrapping_add(x[$k])
                    .wrapping_add(table[$i]))
                .rotate_left($s),
            )
        };
    }

    round4!(a, b, c, d, 0, 6, 49);
    round4!(d, a, b, c, 7, 10, 50);
    round4!(c, d, a, b, 14, 15, 51);
    round4!(b, c, d, a, 5, 21, 52);

    round4!(a, b, c, d, 12, 6, 53);
    round4!(d, a, b, c, 3, 10, 54);
    round4!(c, d, a, b, 10, 15, 55);
    round4!(b, c, d, a, 1, 21, 56);

    round4!(a, b, c, d, 8, 6, 57);
    round4!(d, a, b, c, 15, 10, 58);
    round4!(c, d, a, b, 6, 15, 59);
    round4!(b, c, d, a, 13, 21, 60);

    round4!(a, b, c, d, 4, 6, 61);
    round4!(d, a, b, c, 11, 10, 62);
    round4!(c, d, a, b, 2, 15, 63);
    round4!(b, c, d, a, 9, 21, 64);

    return [a, b, c, d];
}

/**
* utility function to iterate over our slice of u8 ints
* and convert into a vector of unsigned 32 bit ints
*/
fn convert_u8_chunk_to_u32(chunk: &mut [u8]) -> Vec<u32> {
    let mut x: Vec<u32> = Vec::new();

    let mut count = 0;
    let mut temporary_vec: Vec<u8> = Vec::new();
    // iterate over our block and take
    // our 8 bit ints and convert them to
    // 32 bit integers
    for i in 0..chunk.len() {
        temporary_vec.push(chunk[i]);
        count += 1;
        if count == 4 {
            let temp_arr: [u8; 4] = vec_to_array(temporary_vec.clone());
            let value = u32::from_ne_bytes(temp_arr);
            x.push(value);
            count = 0;
            temporary_vec.clear();
        }
    }
    return x;
}

fn compute_md5_digest(mut v: &mut Vec<u8>) -> String {
    // as described in the rfc,
    // 4 32-bit words initialized as fixed constants.
    let mut word_a = 0x67452301u32;
    let mut word_b = 0xefcdab89u32;
    let mut word_c = 0x98badcfeu32;
    let mut word_d = 0x10325476u32;

    // construct the 64 element constant table.
    let table = construct_value_table();

    // let M[0 .. N-1] = words of resulting message, where N is multiple of 16
    for chunk in v.chunks_exact_mut(64) {
        let x = convert_u8_chunk_to_u32(chunk);

        // set all values of a,b,c,d to aa,bb,cc, and dd respectively.
        // this is to save the initial values.
        let word_aa = word_a;
        let word_bb = word_b;
        let word_cc = word_c;
        let word_dd = word_d;

        // execute round 1
        let result = round_one_operations(word_a, word_b, word_c, word_d, &table, &x);
        word_a = result[0];
        word_b = result[1];
        word_c = result[2];
        word_d = result[3];

        // execute round 2
        let result = round_two_operations(word_a, word_b, word_c, word_d, &table, &x);

        word_a = result[0];
        word_b = result[1];
        word_c = result[2];
        word_d = result[3];

        // execute round 3
        let result = round_three_operations(word_a, word_b, word_c, word_d, &table, &x);
        word_a = result[0];
        word_b = result[1];
        word_c = result[2];
        word_d = result[3];

        // execute round 4
        let result = round_four_operations(word_a, word_b, word_c, word_d, &table, &x);
        word_a = result[0];
        word_b = result[1];
        word_c = result[2];
        word_d = result[3];

        // at end of loop iteration, add original word
        // to the current word value
        word_a = word_a.wrapping_add(word_aa);
        word_b = word_b.wrapping_add(word_bb);
        word_c = word_c.wrapping_add(word_cc);
        word_d = word_d.wrapping_add(word_dd);
    }

    // format and return the final result, which
    // is a 128-bit digest string.
    let message_digest = format!(
        "{:08x}{:08x}{:08x}{:08x}",
        word_a.swap_bytes(),
        word_b.swap_bytes(),
        word_c.swap_bytes(),
        word_d.swap_bytes()
    );
    return message_digest;
}

/*
* BIT PADDING STEP
* 1. Append a 1 bit to the end of original message.
* 2. Until the bit length of the data is 64 bits
*    short of being a multiple of 512, append 0's
*    to the data.
* 3. With the last 64 bits, append the length in 64 bits
*    (in lower-order bits first).
*/
fn bit_padding(input: &str) -> Vec<u8> {
    let mut input_vector: Vec<u8> = convert_str_to_vec(input);
    let bit_length: u64 = (input.len() as u64) * 8u64; // todo - add support for > 2^64 bit size

    // 128_u8 is the equivalent of padding 1 as an unsigned 8-bit integer
    // with lower-order bits first
    input_vector.push(128_u8);
    //check if bit length % 512 is 448 (64 less than 512)
    while (input_vector.len() * 8) % 512 != 448 {
        input_vector.push(0_u8); // push in another 8-bit 0 padded value until the correct
                                 // result is reached;
    }

    let length_bits_as_u8_array = split_u64_to_u8_array(bit_length);
    input_vector.extend(length_bits_as_u8_array);

    return input_vector;
}

fn split_u64_to_u8_array(s: u64) -> [u8; 8] {
    let u8_array = [
        s as u8,
        (s >> 8) as u8,
        (s >> 16) as u8,
        (s >> 24) as u8,
        (s >> 32) as u8,
        (s >> 40) as u8,
        (s >> 48) as u8,
        (s >> 56) as u8,
    ];
    return u8_array;
}

fn construct_value_table() -> Vec<u32> {
    let mut t: Vec<u32> = Vec::new();
    t.push(0x00000000);
    for i in 1..=64 {
        t.push(table_construction_function(i));
    }
    return t;
}

// this should only work with utf-8 encoding and not full unicode support
// due to multi-byte unicode chars
fn convert_str_to_vec(input: &str) -> Vec<u8> {
    let mut byte_vec: Vec<u8> = Vec::new();
    byte_vec.extend(input.as_bytes());
    return byte_vec;
}

/**
* A Basic Overview of this MD5 Implementation:
* 1. Take in a command line string and convert into
*    a vector of unsigned 8 bit ints (Vec<u8>)
* 2. Pad the message with a single 1, followed by n 0's,
*    such that the message length in bits % 512 is equal to 448
* 3. Pad the last 64 bits with a little-endian representation
*    of the original message length.
* 4. Convert our vector of unsigned 8-bit integers into
*    blocks of 32 bit unsigned ints (u32).
* 5. Using the 4 auxiliary functions described in rfc 1321,
*    begin a series of 64 bitwise operations on each 512 bit
*    block of our message.
* 6. Once we are done, combine our 4 32 bit registers used to
*    during our bitwise operations to retrieve our message digest.
*    (This is a simplification, but describes the core functionality)
*/
pub fn md5(input: &str) -> String {
    let mut input_vec = bit_padding(input);
    return compute_md5_digest(&mut input_vec);
}

pub fn md5_bits(input: & mut Vec<u8>) -> String {
    return compute_md5_digest(input);
}
