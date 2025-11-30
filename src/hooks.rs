use anyhow::Result;
use retour::static_detour;
use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::ffi::c_char;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::config::CONFIG;
use crate::cxxstring::CxxString;
use crate::dictionary::DICTIONARY;
use crate::utils;

use r#macro::hook;

fn debug_log(msg: &str) {
  if let Ok(mut f) = OpenOptions::new().create(true).append(true).open("dfint_debug_log.txt") {
    let _ = writeln!(f, "{msg}");
  }
}

#[cfg(target_os = "linux")]
#[static_init::dynamic]
static ENABLER: usize = unsafe {
  match CONFIG.symbol.is_some() {
    true => {
      utils::symbol_handle_self::<*const i64>(&CONFIG.symbol.as_ref().unwrap().enabler.as_ref().unwrap()[1]) as usize
    }
    false => 0 as usize,
  }
};

static SEEN_PENDING: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

fn data_dir() -> PathBuf {
  std::env::current_dir().unwrap().join("df-ptbr-llm-mod").join("data")
}

fn llm_cache_lookup(original: &str) -> Option<String> {
  let db_path = data_dir().join("cache.db");
  let conn = Connection::open(db_path).ok()?;
  let mut statement = conn.prepare("SELECT dst FROM translations WHERE src = ?").ok()?;
  let mut rows = statement.query([original]).ok()?;
  match rows.next().ok()? {
    Some(row) => row.get(0).ok(),
    None => None,
  }
}

fn enqueue_for_translation(original: &str) {
    let original = original.trim();
    if original.is_empty() {
        return;
    }

    // Só enfileira na primeira vez que essa string aparecer na sessão
    {
        let mut seen = SEEN_PENDING.lock().unwrap();
        if !seen.insert(original.to_string()) {
            // Já vimos essa string antes, não grava de novo
            return;
        }
    }

    let pending = data_dir().join("pending.txt");
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(pending)
    {
        let _ = writeln!(f, "{original}");
    }
}


fn translate_bytes(original: &[u8]) -> Option<Vec<u8>> {
  debug_log("translate_bytes called");

  let original_str = match std::str::from_utf8(original) {
    Ok(value) => value.trim_end_matches('\0'),
    Err(_) => {
      debug_log("invalid UTF-8");
      return None;
    }
  };

  if original_str.is_empty() {
    return None;
  }

  debug_log(&format!("enqueue: {original_str}"));
  enqueue_for_translation(original_str);

  None
}

pub unsafe fn attach_all() -> Result<()> {
  if CONFIG.settings.enable_translation {
    attach_string_copy_n()?;
    attach_string_append_n()?;
    attach_std_string_ctor()?;
    attach_std_string_append()?;
    attach_std_string_assign()?;
    attach_addst()?;
    attach_addst_top()?;
    attach_addst_flag()?;
  }
  if CONFIG.settings.enable_search && CONFIG.encoding.parsed {
    attach_standardstringentry()?;
    attach_simplify_string()?;
    attach_upper_case_string()?;
    attach_lower_case_string()?;
    attach_capitalize_string_words()?;
    attach_capitalize_string_first_word()?;
  }
  Ok(())
}

pub unsafe fn enable_translation() -> Result<()> {
  enable_string_copy_n()?;
  enable_string_append_n()?;
  enable_std_string_ctor()?;
  enable_std_string_append()?;
  enable_std_string_assign()?;
  enable_addst()?;
  enable_addst_top()?;
  enable_addst_flag()?;
  Ok(())
}

pub unsafe fn enable_search() -> Result<()> {
  enable_standardstringentry()?;
  enable_simplify_string()?;
  enable_upper_case_string()?;
  enable_lower_case_string()?;
  enable_capitalize_string_words()?;
  enable_capitalize_string_first_word()?;
  Ok(())
}

pub unsafe fn enable_all() -> Result<()> {
  enable_translation()?;
  enable_search()?;
  Ok(())
}

pub unsafe fn disable_translation() -> Result<()> {
  disable_string_copy_n()?;
  disable_string_append_n()?;
  disable_std_string_ctor()?;
  disable_std_string_append()?;
  disable_std_string_assign()?;
  disable_addst()?;
  disable_addst_top()?;
  disable_addst_flag()?;
  Ok(())
}

pub unsafe fn disable_search() -> Result<()> {
  disable_standardstringentry()?;
  disable_simplify_string()?;
  disable_upper_case_string()?;
  disable_lower_case_string()?;
  disable_capitalize_string_words()?;
  disable_capitalize_string_first_word()?;
  Ok(())
}

pub unsafe fn disable_all() -> Result<()> {
  disable_translation()?;
  disable_search()?;
  Ok(())
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(bypass))]
fn string_copy_n(dst: *mut c_char, src: *const u8, size: usize) -> *mut c_char {
  unsafe {
    match (std::slice::from_raw_parts(src, size), size > 1) {
      (value, true) => match translate_bytes(value) {
        Some(translate) => {
          let (ptr, len, _) = translate.into_raw_parts();
          original!(dst, ptr, len - 1)
        }
        _ => original!(dst, src, size),
      },
      (_, _) => original!(dst, src, size),
    }
  }
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(bypass))]
fn string_append_n(dst: *mut c_char, src: *const u8, size: usize) -> *mut c_char {
  unsafe {
    match (std::slice::from_raw_parts(src, size), size > 1) {
      (value, true) => match translate_bytes(value) {
        Some(translate) => {
          let (ptr, len, _) = translate.into_raw_parts();
          original!(dst, ptr, len - 1)
        }
        _ => original!(dst, src, size),
      },
      (_, _) => original!(dst, src, size),
    }
  }
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(bypass))]
fn std_string_ctor(dst: *const u8, src: *const u8, size: usize) -> *const u8 {
  unsafe {
    match (std::slice::from_raw_parts(src, size), size > 1) {
      (value, true) => match translate_bytes(value) {
        Some(translate) => {
          let (ptr, len, _) = translate.into_raw_parts();
          original!(dst, ptr, len - 1)
        }
        _ => original!(dst, src, size),
      },
      (_, _) => original!(dst, src, size),
    }
  }
}

#[cfg_attr(target_os = "windows", hook(bypass))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn std_string_append(dst: *const u8, src: *const u8) -> *const u8 {
  unsafe {
    match std::ffi::CStr::from_ptr(src as *const c_char).to_bytes() {
      (value) => match translate_bytes(value) {
        Some(translate) => {
          let (ptr, _, _) = translate.into_raw_parts();
          original!(dst, ptr)
        }
        _ => original!(dst, src),
      },
      _ => original!(dst, src),
    }
  }
}

#[cfg_attr(target_os = "windows", hook(bypass))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn std_string_assign(dst: *const u8, src: *const u8) -> *const u8 {
  unsafe {
    match std::ffi::CStr::from_ptr(src as *const c_char).to_bytes() {
      (value) => match translate_bytes(value) {
        Some(translate) => {
          let (ptr, _, _) = translate.into_raw_parts();
          original!(dst, ptr)
        }
        _ => original!(dst, src),
      },
      _ => original!(dst, src),
    }
  }
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn addst(gps: usize, src: *const u8, justify: u8, space: u32) {
  unsafe {
    let s = CxxString::from_ptr(src);
    match s.to_bytes_without_nul() {
      converted => match translate_bytes(converted) {
        Some(translate) => {
          let (ptr, len, _) = translate.into_raw_parts();
          let mut cxxstr = CxxString::new(ptr, len - 1);
          #[cfg(target_os = "linux")]
          {
            if cxxstr.len < 16 {
              cxxstr.ptr = cxxstr.sso.buf.as_mut_ptr();
            }
          }
          original!(gps, cxxstr.as_ptr(), justify, space)
        }
        _ => original!(gps, src, justify, space),
      },
      _ => original!(gps, src, justify, space),
    }
  }
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn addst_top(gps: usize, src: *const u8, justify: u8, space: u32) {
  unsafe {
    let s = CxxString::from_ptr(src);
    match s.to_bytes_without_nul() {
      converted => match translate_bytes(converted) {
        Some(translate) => {
          let (ptr, len, _) = translate.into_raw_parts();
          let mut cxxstr = CxxString::new(ptr, len - 1);
          #[cfg(target_os = "linux")]
          {
            if cxxstr.len < 16 {
              cxxstr.ptr = cxxstr.sso.buf.as_mut_ptr();
            }
          }
          original!(gps, cxxstr.as_ptr(), justify, space)
        }
        _ => original!(gps, src, justify, space),
      },
      _ => original!(gps, src, justify, space),
    }
  }
}

#[cfg_attr(target_os = "windows", hook(bypass))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn addst_flag(gps: usize, src: *const u8, a3: usize, a4: usize, flag: u32) {
  unsafe {
    let s = CxxString::from_ptr(src);
    match s.to_bytes_without_nul() {
      converted => match translate_bytes(converted) {
        Some(translate) => {
          let (ptr, len, _) = translate.into_raw_parts();
          let mut cxxstr = CxxString::new(ptr, len - 1);
          #[cfg(target_os = "linux")]
          {
            if cxxstr.len < 16 {
              cxxstr.ptr = cxxstr.sso.buf.as_mut_ptr();
            }
          }
          original!(gps, cxxstr.as_ptr(), a3, a4, flag)
        }
        _ => original!(gps, src, a3, a4, flag),
      },
      _ => original!(gps, src, a3, a4, flag),
    }
  }
}

#[non_exhaustive]
struct StringEntry;

#[allow(dead_code)]
impl StringEntry {
  pub const LETTERS: u8 = 1;
  pub const SPACE: u8 = 2;
  pub const NUMBERS: u8 = 4;
  pub const CAPS: u8 = 8;
  pub const SYMBOLS: u8 = 16;
  pub const STRINGENTRY_FILENAME: u8 = 32;
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn standardstringentry(src: *const u8, maxlen: usize, flag: u8, events_ptr: *const u8, utf: *const u32) -> bool {
  unsafe {
    let utf_a = std::slice::from_raw_parts_mut(utf as *mut u32, 8);
    #[cfg(target_os = "linux")]
    {
      let utf_a = std::slice::from_raw_parts_mut(
        (*ENABLER + &CONFIG.offset.as_ref().unwrap().utf_input.unwrap()) as *mut u32,
        8,
      );
    }

    let content = CxxString::from_ptr(src);
    let mut any_valid = false;

    for i in 0..8 {
      if utf_a[i] == 0 || utf_a[i] == 10 || content.size() >= maxlen {
        break;
      }

      if utf_a[i] > 122 && CONFIG.encoding.utf.contains_key(&utf_a[i]) {
        let entry = CONFIG.encoding.utf[&utf_a[i]];
        utf_a[i] = match (flag & StringEntry::CAPS) > 0 {
          true => capitalize(entry),
          false => entry,
        } as u32;

        if content.size() < maxlen {
          content.push_back(utf_a[i] as u8);
          any_valid = true;
        }
      }
    }

    any_valid || original!(content.as_ptr(), maxlen, flag, events_ptr, utf_a.as_ptr())
  }
}

fn capitalize(symbol: u8) -> u8 {
  CONFIG.encoding.capitalize[symbol as usize]
}

#[allow(dead_code)]
fn lowercast(symbol: u8) -> u8 {
  CONFIG.encoding.lowercast[symbol as usize]
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn simplify_string(src: *const u8) {
  unsafe {
    let mut content = CxxString::from_ptr(src);
    for i in 0..content.len {
      content[i] = CONFIG.encoding.simplify[content[i] as usize];
    }
  }
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn upper_case_string(src: *const u8) {
  unsafe {
    let mut content = CxxString::from_ptr(src);
    for i in 0..content.len {
      content[i] = CONFIG.encoding.uppercase[content[i] as usize]
    }
  }
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn lower_case_string(src: *const u8) {
  unsafe {
    let mut content = CxxString::from_ptr(src);
    for i in 0..content.len {
      content[i] = CONFIG.encoding.lowercase[content[i] as usize]
    }
  }
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn capitalize_string_words(src: *const u8) {
  unsafe {
    let mut bracket_count: i32 = 0;
    let mut content = CxxString::from_ptr(src);
    for i in 0..content.len {
      match content[i] {
        91 => {
          bracket_count += 1;
          continue;
        }
        93 => {
          bracket_count -= 1;
          continue;
        }
        _ => (),
      };
      if bracket_count > 0 {
        continue;
      }
      let mut conf = false;
      if (i > 0 && content[i - 1] == 32 || content[i - 1] == 34)
        || (i >= 2 && content[i - 1] == 39 && (content[i - 2] == 32 || content[i - 2] == 44))
      {
        conf = true;
      }
      if i == 0 || conf {
        content[i] = CONFIG.encoding.uppercase[content[i] as usize];
      }
    }
  }
}

#[cfg_attr(target_os = "windows", hook(by_offset))]
#[cfg_attr(target_os = "linux", hook(by_symbol))]
fn capitalize_string_first_word(src: *const u8) {
  unsafe {
    let mut bracket_count: i32 = 0;
    let mut content = CxxString::from_ptr(src);
    for i in 0..content.len {
      match content[i] {
        91 => {
          bracket_count += 1;
          continue;
        }
        93 => {
          bracket_count -= 1;
          continue;
        }
        _ => (),
      };
      if bracket_count > 0 {
        continue;
      }
      let mut conf = false;
      if (i > 0 && content[i - 1] == 32 || content[i - 1] == 34)
        || (i >= 2 && content[i - 1] == 39 && (content[i - 2] == 32 || content[i - 2] == 44))
      {
        conf = true;
      }
      if i == 0 || conf {
        content[i] = CONFIG.encoding.uppercase[content[i] as usize];
        if content[i] != 32 && content[i] != 34 {
          return;
        }
      }
    }
  }
}
