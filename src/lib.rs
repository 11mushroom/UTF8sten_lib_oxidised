/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (c) 2025 11mushroom
 */

use std::{char};

const FFU32  :u32=0xffffffff;
const OCTPR  :u32=0b10000000;
const PB2    :u32=0b11000000;
const PB3    :u32=0b11100000;
const PB4    :u32=0b11110000;
const MASK3  :u32=0b00000111;
const MASK4  :u32=0b00001111;
const MASK5  :u32=0b00011111;
const MASK6  :u32=0b00111111;
const MASK8  :u32=0b11111111;
const RMASK2 :u32=0b11000000;
const RMASK3 :u32=0b11100000;
const RMASK4 :u32=0b11110000;
const RMASK5 :u32=0b11111000;

const ENC1BT_BASE:u32=0x00100 ; //base for encoding single byte
const ENC12B_BASE:u32=0x08000 ; //base for encoding 12 bits
const ENC2BT_BASE:u32=0x20000 ; //base for encoding 2 bytes

///function to calculate amount of encoded data will take in bytes
///see also [`en_len_chrs()`] for calculating how much will it take in [prim@char]s

#[inline(always)]
pub const fn getEnLen(len: usize) -> usize {
  (len/3)*6+(len%3)*2
}

///function to calculate length of encoded data in unicode characters
///used for v1 encoding
///for v2 encoding see [`en2_len_chrs()`]
#[inline(always)]
pub const fn en_len_chrs(len: usize) -> usize {
  (len/3)*2+len%3
}

///function to calculate length of encoded data in unicode characters
///used for v2 encoding
///for v1 encoding see [`en_len_chrs()`]
#[inline(always)]
pub const fn en2_len_chrs(len:usize) -> usize {
  (len/2)+(len%2)
}

/*
///function to calculate amount of decoded data will take in bytes

pub fn getStenLen(arr: &[u32]) -> usize {
  let mut res:usize=0;
  let mut bits:u32=0;

  for i in 0..arr.len() {
    if arr[i]<=0x8fff && arr[i]>=0x8000 {
      bits+=12;
    } else if arr[i]<=0xff {
      bits+=8;
    }

    //res+=bits/8;
    //bits=bits%8;
    res+=(bits>>3) as usize;
    bits&=7;
  }

  if bits>0 { 
    res+=1;
  }

  return res;
}
*/

/*fn gBit(num:u8, ind:u32) -> u32{
  (num>>ind)&1
}*/


///functions to get value of specific bit in number
#[inline(always)]
fn gBit(num:u32, ind:u32) -> u32{
  (num>>ind)&1
}

/// returns how many bytes codepoint will take in utf8 format
///
/// # Safety
/// codepoint must be valid unicode (this function doesn't check codepoint)
///
#[inline]
pub const unsafe fn utf8_len(code: u32) -> usize {
  match code {
    ..0x0080  => 1,
    ..0x0800  => 2,
    ..0x10000 => 3,
    _ => 4
  }
}

/// encodes one u32 codepoint into utf8 bytes using raw pointer
///
/// # Safety
/// doesn't check whether codepoint is valid unicode
/// so you must be sure that codepoint you give is valid (function doesn't check it)
///
/// doesn't check boundaries of destination
/// so you must be sure that destination is at least 4 bytes long (for full range of codepoints)
///
#[inline]
pub const unsafe fn u32_to_utf8_raw_unchecked (code: u32, dest: *mut u8) -> usize {
  let len = unsafe {utf8_len(code)};

  unsafe {
    match len {
      1 => {
        *dest = code as u8;
        return len;
      },

      2 => {
        *dest        = (PB2  |((code>>6)&MASK5)) as u8;
        *dest.add(1) = (OCTPR| (code&MASK6))     as u8;
        return len;
      },

      3 => {
        *dest        = (PB3  |((code>>12)&MASK4)) as u8;
        *dest.add(1) = (OCTPR|((code>>6)&MASK6))  as u8;
        *dest.add(2) = (OCTPR| (code&MASK6))      as u8;
        return len;
      },
      _ => {
        *dest        = (PB4  |((code>>18)&MASK3)) as u8;
        *dest.add(1) = (OCTPR|((code>>12)&MASK6)) as u8;
        *dest.add(2) = (OCTPR|((code>>6)&MASK6))  as u8;
        *dest.add(3) = (OCTPR| (code&MASK6))      as u8;
        len
      }
    }
  }
}

/// takes &\[[u32]\] slice of codepoints
/// and encodes it into utf8 format into array of bytes by raw pointer,
/// returns length of written utf8 bytes
///
/// # Safety
/// can produce invalid utf8 data as it doesn't check if codepoints are valid
///
/// can go out of bounds as it doesn't check boundaries of destination
///
pub unsafe fn u32_slice_to_utf8_unchecked(codes: *const u32, codes_len: usize, dest: *mut u8) -> usize {
    let mut dest_ptr = dest;
    for i in 0..codes_len {
        dest_ptr = unsafe {
            dest_ptr.add(u32_to_utf8_raw_unchecked(*codes.add(i), dest_ptr))
        };
    }

    unsafe { dest_ptr.offset_from_unsigned(dest) }
}

/// same as [`u32_slice_to_utf8_unchecked()`] but for &\[[prim@char]\] slice
///
/// # Safety
/// can produce invalid utf8 data as it doesn't check if codepoints are valid
///
/// can go out of bounds as it doesn't check boundaries of destination
///
#[inline(always)]
pub unsafe fn char_slice_to_utf8_unchecked(chrs: *const char, chrs_len: usize, dest: *mut u8) -> usize {
    unsafe { u32_slice_to_utf8_unchecked(chrs as *const u32, chrs_len, dest) }
}

//function to encode single code point into UTF-8
//it recives unsigned code and returns structure of array of bytes

/*fn UTF8_enc(code:u32) -> Vec<u8> {
  let len:u8 = 
    if code<0x10000 {
        if code<0x0800 {
            if code<0x0080 {1} else {2}
        } else {3}
    } else {4};

  let mut bytes:Vec<u8>=vec![0;len as usize];
 
  match len {
    1_u8 => {
      bytes[0]=code as u8;
    },

    2_u8 => {
      bytes[0]=(PB2|((code>>6)&MASK5)) as u8;
      bytes[1]=(OCTPR|(code&MASK6)) as u8;
    },

    3_u8 => {
      bytes[0]=(PB3|((code>>12)&MASK4)) as u8;
      bytes[1]=(OCTPR|((code>>6)&MASK6)) as u8;
      bytes[2]=(OCTPR|(code&MASK6)) as u8;
    },

    4_u8 => {
      bytes[0]=(PB4|((code>>18)&MASK3)) as u8;
      bytes[1]=(OCTPR|((code>>12)&MASK6)) as u8;
      bytes[2]=(OCTPR|((code>>6)&MASK6)) as u8;
      bytes[3]=(OCTPR|(code&MASK6)) as u8;
    },
    _ => {
    }
  }
  
  return bytes;
}*/

///function to calculate length of string not by bytes but by characters, including UTF-8 characters
#[inline]
fn calcLen(str:&String) -> usize {
  str.chars().count()
}

///function to decode string that contains UTF-8 characters and returns Vector with codepoints of characters
#[inline(always)]
pub fn UTF8_den( string:&String ) -> Vec<u32> {
  string.chars().map(|c| c as u32).collect::<Vec<u32>>()
}

///wrap around the [`enSten_to()`] function
///if you don't need to reuse buffer
#[inline(always)]
pub fn enSten(arr: &[u8]) -> Vec<char> {
  let mut res:Vec<char>=vec![0 as char;en_len_chrs(arr.len())];
  let _ = unsafe { enSten_to(arr, &mut res) };
  res
}


///function to encode bytes in UTF-8 characters,
///recives array of bytes and buffer to store result to, and returns size of data written
///use it if you need to reuse buffer
///otherwise you can use [`enSten()`]
///
///length of the buffer must be >= of the amount of data that will be written 
///
/// # Examples
///
/// ```
/// let arr = vec![0_u8; 12];
/// let mut buffer = vec![0 as char; utf8sten::en_len_chrs(arr.len())];
/// 
/// assert!(buffer.len() >= utf8sten::en_len_chrs(arr.len()));
/// let _ = unsafe { utf8sten::enSten_to(&arr, &mut buffer) };
/// ```
///
/// # Safety
/// can go out of bounds when writing to the buffer
/// so you must make sure that buffer if large enough
///
pub unsafe fn enSten_to(arr: &[u8], buffer:&mut [char]) -> usize {
  let len:usize=arr.len();
  let normal_len:usize = len-len%3;

  let mut dataI:usize=0;
  let mut i:usize=0;
  let mut buff:u32;

  unsafe {
    while i < normal_len {
        // store source 3 bytes in buffer to process it later
        buff=u32::from_le_bytes([*arr.get_unchecked(i), *arr.get_unchecked(i+1), *arr.get_unchecked(i+2), 0]);

        // SAFETY:
        // the range of codepoints 0x8000-0x8fff is entirely valid
        // so we don't need validation
        *buffer.get_unchecked_mut(dataI)=char::from_u32_unchecked(ENC12B_BASE|( buff    &0x0fff));
        dataI+=1;

        *buffer.get_unchecked_mut(dataI)=char::from_u32_unchecked(ENC12B_BASE|((buff>>12)&0x0fff));
        dataI+=1;

        i+=3;
    }

    while i<len {
      *buffer.get_unchecked_mut(dataI)=char::from_u32_unchecked(ENC1BT_BASE|*arr.get_unchecked(i) as u32);
      dataI+=1;
      i+=1;
    }
  }

  return dataI;
}

///function to encode bytes in UTF-8 characters, recives array of bytes and length of that array, and returns vector with codepoints with data stored in it
///uses old way to encode which can be slower
/// # Support
/// support is discontinued, it will be deleted in the future
///
pub fn legacy_enSten(arr: &[u8]) -> Vec<char> {
  let len:usize=arr.len();
  let enLen:usize=(len/3)*2+ len%3;
  let normal_len:usize=len-len%3;

  let mut res:Vec<char>=vec![0 as char;enLen];

  let mut codePoint:u32=0x8000;
  let mut subB:u8=0;
  let mut bits:u8=0;
  let mut cary:u8;
  let mut shift:u8;
  let mut bitsPass:u8=0;
  let mut dataI:usize=0;
  let mut i:usize=0;

  while i < normal_len {
    if bits==0 {
      bits=8;
    }

    cary=12-subB;
    shift=subB;


    if bits<=cary {
      subB+=bits;
      //codePoint|=((arr[i]>>bitsPass)&((1<<bits)-1))<<shift;
      codePoint|=(((arr[i] as u32)>>bitsPass)&(!(FFU32<<bits)))<<shift;
      bits=0;
      bitsPass=0;
      i += 1;

    } else {
      subB=12;
      //codePoint|=((arr[i]>>bitsPass)&((1<<cary)-1))<<shift;
      codePoint|=(((arr[i] as u32)>>bitsPass)&(!(FFU32<<cary)))<<shift;
      bits-=cary;
      bitsPass=cary;

    }


    if subB>=12||(i>=len) {
      res[dataI]=char::from_u32(codePoint).expect("[enSten()]: failed to convert codepoint into char");
      dataI += 1;
      codePoint=0x8000;
    }

    subB%=12;
  }

  while i<len {
    res[dataI]=char::from_u32(ENC1BT_BASE|arr[i] as u32).expect("[enSten()]: failed to encode single byte");
    i+=1;
    dataI+=1;
  }

  if dataI < enLen {
    res.truncate(dataI);
  }

  return res;
}

///wrap around the [`enSten2_to()`] function
///if you don't need to reuse buffer
///
/// # Panics
/// will panic for the same reasons that [`enSten2_to()`] panics
///
#[inline(always)]
pub fn enSten2(arr: &[u8]) -> Vec<char> {
  let mut res:Vec<char>=vec![0 as char;en2_len_chrs(arr.len())];
  let _ = unsafe { enSten2_to(arr, &mut res) };
  res
}

///function to encode bytes in UTF-8 characters,
///recives array of bytes and buffer to store result to, and returns size of data written
///use it if you need to reuse buffer
///otherwise you can use [`enSten2()`]
///
///length of the buffer must be >= of the amount of data that will be written
///
/// # Examples
///
/// ```
/// let arr = vec![0_u8; 12];
/// let mut buffer = vec![0 as char; utf8sten::en2_len_chrs(arr.len())];
/// 
/// assert!(buffer.len() >= utf8sten::en2_len_chrs(arr.len()));
/// let _ = unsafe { utf8sten::enSten2_to(&arr, &mut buffer) };
/// ```
///
/// # Safety
/// can go out of bounds when writing to the buffer
/// so you must make sure that buffer if large enough
///
/// works reliably with ascii table values (x<=0x7f) but
/// other byte values are just gamble, and can cause function to panic, use
/// [`v2_encode_valid()`] function to check your data
///
pub unsafe fn enSten2_to(arr: &[u8], buffer: &mut [char]) -> usize {
  let len:usize=arr.len();

  let mut i:usize=0;
  let mut dataI:usize=0;

  unsafe {
    for chn in arr.chunks_exact(2) {
        *buffer.get_unchecked_mut(dataI)=char::from_u32(ENC2BT_BASE|u32::from_le_bytes([chn[0],chn[1],0,0])) 
                          .expect(format!("data cannot be encoded in second format, cause of problem around {i} input byte").as_str());
        dataI+=1;
        i+=2;
    }
    
    if len%2==1 {
      *buffer.get_unchecked_mut(dataI)=char::from_u32_unchecked(ENC1BT_BASE | arr[len^1] as u32);
      dataI+=1;
    }
  }

  return dataI;
}

const V2_ST_VOID1:u32 = 42720;

/// checks whether data can be encoded with v2 encoding without panics
pub fn v2_encode_valid(arr:&[u8]) -> bool {
  let len:usize=arr.len();
  let flen:usize=len>>1;

  let mut i:usize=0;

  while i<flen {
    let code:u32=u32::from_le_bytes([arr[i],arr[i|1],0,0]);
    if code>=V2_ST_VOID1 {
      return false;
    }
    i+=2;
  }

  true
}

///check if slice of codepoints is a valid u8s v1 data
pub fn valid_en_v1(codes: &[u32]) -> bool {
    if codes.is_empty() { return true };

    let norm_len = codes.len()^(codes.len()&1);
    if norm_len<codes.len() {
        for ch in codes.chunks_exact(2) {
            if ch[0]&0xfffff000!=ENC12B_BASE ||
               ch[1]&0xfffff000!=ENC12B_BASE {
                return false;
            }
        }
        // check last element
        return codes[norm_len]&0xffffff00==ENC1BT_BASE;
    } else {
        for ch in codes[..norm_len-2].chunks_exact(2) {
            if ch[0]&0xfffff000!=ENC12B_BASE ||
               ch[1]&0xfffff000!=ENC12B_BASE {
                return false;
            }
        }
        // if not( is_0x8000(codes[end-2..end]) or is_0x100(codes[end-2..end]) )
        if (codes[norm_len-2]&0xfffff000!=ENC12B_BASE ||
            codes[norm_len-1]&0xfffff000!=ENC12B_BASE) &&
           (codes[norm_len-2]&0xffffff00!=ENC1BT_BASE ||
            codes[norm_len-1]&0xffffff00!=ENC1BT_BASE) {
            return false;
        }
    }

    true

}

///calculate maximum possible length of data being decoded from u8s v1 encoded codepoints
#[inline(always)]
pub fn de_len_max(en_len: usize) -> usize {
    en_len+en_len/2
}

///safe wrap around the [`deSten_to_raw_unchecked()`] function
///if you don't need to reuse buffer
///only for v1 encoded data
#[inline(always)]
pub fn deSten(codes: &[u32]) -> Result<Vec<u8>, &str> {
    let mut res:Vec<u8> = Vec::with_capacity(de_len_max(codes.len()));
    if !valid_en_v1(codes) {
        return Err("invalid u8s v1 data");
    }

    unsafe {
      let len = deSten_to_raw_unchecked(codes.as_ptr(), codes.len(), res.as_mut_ptr());
      res.set_len(len);
    }
    Ok(res)
}

///safe wrap around the [`deSten_to_raw_unchecked()`] function
///only for v1 encoded data
#[inline(always)]
pub fn deSten_to<'a>(codes: &'a [u32], buffer: &'a mut [u8]) -> Result<usize, &'static str> {
    if buffer.len() < de_len_max(codes.len()) {
        return Err("buffer overflow is possible, length of buffer must be >= than de_len_max()");
    } else if !valid_en_v1(codes) {
        return Err("invalid u8s v1 data");
    }

    unsafe {
        Ok(deSten_to_raw_unchecked(codes.as_ptr(), codes.len(), buffer.as_mut_ptr()))
    }
}

///function to decode data from codepoints, first version of encoding, but doesn't check if input
///data is valid
///it's more optimized specifically for decoding first version
///only for v1 decoding
///recives raw pointer to array of codepoints, length of the array and raw mutable pointer to buffer
///where to store result, returns size of data written
///use it if you need to reuse buffer or whatever
///otherwise you can use safe [`deSten_to()`] or [`deSten()`]
///
///length of the buffer should be 1.5 as long as array with codepoints (see [`de_len_max()`] for calculating it)
///
/// # Examples
/// ```
/// # use std::mem;
/// let u8s_message:Vec<u32> = unsafe {mem::transmute( utf8sten::enSten("hello".as_bytes()) )};
/// # assert!(utf8sten::valid_en_v1(&u8s_message));
/// let mut buffer:Vec<u8> = vec![0_u8; utf8sten::de_len_max(u8s_message.len())];
///
/// assert!(buffer.len() >= utf8sten::de_len_max(u8s_message.len()));
/// let len:usize = unsafe {
///     utf8sten::deSten_to_raw_unchecked(u8s_message.as_ptr(), u8s_message.len(), buffer.as_mut_ptr())
/// };
///
/// # assert_eq!(&buffer[..len], "hello".as_bytes());
/// ```
///
/// # Safety
/// can go out of bounds when writing to buffer as it doesn't check buffer's boundaries at all
/// 
pub unsafe fn deSten_to_raw_unchecked(codes: *const u32, codes_len: usize, buffer: *mut u8) -> usize {
    let even_len:isize = ((codes_len^(codes_len&1))-2) as isize;
    let even_len:usize = (even_len & !(even_len >> (isize::BITS-1))) as usize;
    let even_suf_len:usize = codes_len^(codes_len&1);
    let mut res_i:usize=0;
    let mut i:usize=0;
    let mut buff:u32;

    unsafe {
      while i<even_len {
        buff = (*codes.add(i)&0xfff) | ((*codes.add(i+1)&0xfff) << 12);
        *buffer.add(res_i  ) = ((buff    )&0xff) as u8;
        *buffer.add(res_i+1) = ((buff>>8 )&0xff) as u8;
        *buffer.add(res_i+2) = ((buff>>16)&0xff) as u8;
        res_i+=3;
        i+=2;
      }
      while i<even_suf_len {
        if *codes.add(i  )&0xfffff000 == ENC12B_BASE &&
           *codes.add(i+1)&0xfffff000 == ENC12B_BASE {
            buff = (*codes.add(i)&0xfff) | ((*codes.add(i+1)&0xfff) << 12);
            *buffer.add(res_i  ) = ((buff    )&0xff) as u8;
            *buffer.add(res_i+1) = ((buff>>8 )&0xff) as u8;
            *buffer.add(res_i+2) = ((buff>>16)&0xff) as u8;
            res_i+=3;
            i+=2;
            continue;
        }
        *buffer.add(res_i  ) = (*codes.add(i  )&0xff) as u8;
        *buffer.add(res_i+1) = (*codes.add(i+1)&0xff) as u8;
        res_i+=2;
        i+=2;
      }

      if i<codes_len {
        *buffer.add(res_i) = (*codes.add(i)&0xff) as u8;
        res_i+=1;
      }
    }
    res_i
}

///calculate maximum possible length of data being decoded from u8s v2 encoded codepoints
#[inline(always)]
pub fn de2_len_max(codes_len: usize) -> usize {
    codes_len*2
}

///check if slice of codepoints is a valid u8s v2 data
pub fn valid_en_v2(codes: &[u32]) -> bool {

    for i in 0..codes.len() {
      unsafe {
        // if not( is_0x20000(codes[i]) or is_0x100(codes[i]) )
        if *codes.get_unchecked(i)&0xffff0000!=ENC2BT_BASE &&
           *codes.get_unchecked(i)&0xffffff00!=ENC1BT_BASE {
            return false;
        }
      }
    }

    true
}

///wrap around the [`deSten2_to_raw_unchecked()`] function
///if you don't need to reuse buffer
///only for v2 encoded data
///
///will return Error if buffer is too small for safe use or if input data is invalid
#[inline(always)]
pub fn deSten2(codes: &[u32]) -> Result<Vec<u8>, &str> {
  let mut res:Vec<u8>=Vec::with_capacity(de2_len_max(codes.len()));
  if !valid_en_v2(codes) {
      return Err("invalid u8s v2 data");
  }

  unsafe{
    let len = deSten2_to_raw_unchecked(codes.as_ptr(), codes.len(), res.as_mut_ptr());
    res.set_len(len);
  }
  Ok(res)
}

///safe wrap around the [`deSten2_to_raw_unchecked()`] function
///only for v2 encoded data
///
///will return Error if buffer is too small for safe use or if input data is invalid
#[inline(always)]
pub fn deSten2_to<'a>(codes: &'a [u32], buffer: &'a mut [u8]) -> Result<usize, &'static str> {
    if buffer.len() < de2_len_max(codes.len()) {
        return Err("buffer overflow is possible, length of buffer must be >= than de2_len_max()");
    } else if !valid_en_v2(codes) {
        return Err("invalid u8s v2 data");
    }

    unsafe {
        Ok(deSten_to_raw_unchecked(codes.as_ptr(), codes.len(), buffer.as_mut_ptr()))
    }
}

///function to decode data from codepoints, second version of encoding, but doesn't check if input
///data is valid
///it's more optimized specifically for decoding second version
///only for v2 decoding
///recives raw pointer to array of codepoints, length of the array and raw mutable pointer to buffer
///where to store result, returns size of data written
///use it if you need to reuse buffer or whatever
///otherwise you can use [`deSten2_to()`] or [`deSten2()`]
///
///length of the buffer should be twice as long as array with codepoints ( [`de2_len_max()`] )
///
///# Examples
///```
/// # use std::mem;
/// let u8s_message:Vec<u32> = unsafe {mem::transmute( utf8sten::enSten2("hello".as_bytes()) )};
/// # assert!(utf8sten::valid_en_v2(&u8s_message));
/// let mut buffer:Vec<u8> = vec![0_u8; utf8sten::de2_len_max(u8s_message.len())];
///
/// assert!(buffer.len() >= utf8sten::de2_len_max(u8s_message.len()));
/// let len:usize = unsafe {
///     utf8sten::deSten2_to_raw_unchecked(u8s_message.as_ptr(), u8s_message.len(), buffer.as_mut_ptr())
/// };
///
/// # assert_eq!(&buffer[..len], "hello".as_bytes());
///```
///
/// # Safety
/// can go out of bounds when writing to buffer as it doesn't check buffer's boundaries at all
/// 
pub unsafe fn deSten2_to_raw_unchecked(codes: *const u32, codes_len: usize ,buffer: *mut u8) -> usize {
  let mut res_i:usize=0;

  for i in 0..codes_len {
    unsafe {
      match *codes.add(i) {
        x @ 0x020000..=0x02ffff => {
          *buffer.add(res_i  ) =  x     as u8;
          *buffer.add(res_i+1) = (x>>8) as u8;
          res_i+=2;
        },

        x @ 0x00..=0x1ff => {
          *buffer.add(res_i) = x as u8;
          res_i+=1;
        },
        _ => {},
      }
    }
  }

  return res_i;
}

/*
///wrap around the `deSten_to` function
///if you don't need to reuse buffer
#[doc(hidden)]
#[inline(always)]
pub fn deSten_legacy(arr: &[u32]) -> Vec<u8> {
  let mut res:Vec<mem::MaybeUninit<u8>>=Vec::with_capacity(arr.len()*2);
  unsafe{
    res.set_len(arr.len()*2);
    let len = deSten_to_legacy(arr, mem::transmute(res.as_mut_slice()));
    res.set_len(len);
    mem::transmute(res)
  }
}

///function to decode data from utf8sten codepoints into buffer
///decodes result of enSten and enSten2 functions
///recives slice of utf8sten codepoints and buffer to store result to, and returns size of data written
///use it if you need to reuse buffer
///otherwise you can use `deSten()`
///
///length of the buffer should be twice as long as slice with codepoints:
///```
///assert!(buffer.len() >= arr.len()*2);
///let _ = unsafe { deSten_to(arr, buffer) };
///```
#[doc(hidden)]
pub unsafe fn deSten_to_legacy(arr: &[u32], buffer: &mut [u8]) -> usize {
  let len:usize=arr.len();

  let mut dataI:usize=0;
  let mut bits:u8;
  let mut bitsPass:u8;
  let mut subB:u8=0;
  let mut cary:u8;
  let mut shift:u8;
  let mut proc:bool;


  for i in 0..len {
    proc=false;
    bitsPass=0;
    bits=0;

    if arr[i]<=0x8fff && arr[i]>=0x8000 {
      bits=12;
      proc=true;
    
    } else if arr[i]<=0x02ffff && arr[i]>=0x020000 {
      bits=16;
      proc=true;
    
    } else if arr[i]<=0x1ff && arr[i]>=0x100 {
      bits=8;
      proc=true;
      
    } else if arr[i]<=0xff {
      bits=8;
      proc=true;
    }
    
    //proccess data
    while bits>0 && proc {
      cary=8-subB;
      shift=subB;

      if bits<=cary {
        subB+=bits;
        //res.bytes[dataI] |= ((arr[i]>>bitsPass)&((1<<bits)-1))<<shift;
        buffer[dataI] |= (((arr[i]>>bitsPass)&(!(FFU32<<bits)))<<shift) as u8;
        bits=0;

      } else if bits>cary {
        subB=8;
        //res.bytes[dataI] |= ((arr[i]>>bitsPass)&((1<<cary)-1))<<shift;
        buffer[dataI] |= (((arr[i]>>bitsPass)&(!(FFU32<<cary)))<<shift) as u8;
        bits-=cary;
        bitsPass+=cary;
        
      }

      dataI+=(subB/8) as usize;
      unsafe { *buffer.get_unchecked_mut(dataI) = ((subB/8)-1) & *buffer.get_unchecked(dataI) };
      subB%=8;
    }

  }

  return dataI;

}
*/

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn enstenning_works() {
        eprintln!("\nSTART enSten");

        let buff:Vec<&str>=vec!["hello!", "hello", "hell"];
        let need:Vec<String>=["蕨蛆转舖", "蕨蛆Ŭů", "蕨蛆Ŭ"].into_iter().map(String::from).collect();
        assert_eq!(buff.len(), need.len());

        for i in 0..buff.len() {
          let res = enSten(buff[i].as_bytes());
          eprintln!("result{i}: {}", res.iter().collect::<String>());

          eprintln!("need{i}:   {}", need[i]);

          assert_eq!(res, need[i].chars().collect::<Vec<char>>());
        };
    }

    #[test]
    fn destenning_works() {
        eprintln!("\nSTART deSten");

        let buff:Vec<String>=["蕨蛆转舖", "蕨蛆Ŭů", "蕨蛆Ŭ"].into_iter().map(String::from).collect();
        let need:Vec<&str>=vec!["hello!", "hello", "hell"];
        assert_eq!(buff.len(), need.len());

        for i in 0..buff.len() {
          let res = deSten(&buff[i].chars().map(|c| c as u32).collect::<Vec<u32>>()).expect("testing values must be correct");
          eprintln!("result{i}: {}", String::from_utf8(res.clone()).expect("deSten test failed"));

          eprintln!("need{i}:   {}", need[i]);

          assert_eq!(res, need[i].as_bytes());
        };
    }
    
    #[test]
    fn enstenning_v2_works() {
        eprintln!("\nSTART enSten2");

        let buff:Vec<&str>=vec!["hello!", "hello"];
        let need:Vec<String>=["𦕨𦱬𢅯", "𦕨𦱬ů"].into_iter().map(String::from).collect();
        assert_eq!(buff.len(), need.len());

        for i in 0..buff.len() {
          let res = enSten2(buff[i].as_bytes());
          eprintln!("result{i}: {}", res.iter().collect::<String>());

          eprintln!("need{i}:   {}", need[i]);

          assert_eq!(res, need[i].chars().collect::<Vec<char>>());
        };
    }

    #[test]
    fn destenning_v2_works() {
        eprintln!("\nSTART deSten2");

        let buff:Vec<String>=["𦕨𦱬𢅯", "𦕨𦱬ů"].into_iter().map(String::from).collect();
        let need:Vec<&str>=vec!["hello!", "hello"];
        assert_eq!(buff.len(), need.len());

        for i in 0..buff.len() {
          let res = deSten2(&buff[i].chars().map(|c| c as u32).collect::<Vec<u32>>()).expect("testing values must be correct");
          eprintln!("result{i}: {}", String::from_utf8(res.clone()).expect("result should be valid"));

          eprintln!("need{i}:   {}", need[i]);

          assert_eq!(res, need[i].as_bytes());
        };
    }
}
