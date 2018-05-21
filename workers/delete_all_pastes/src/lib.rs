use std::ffi::CStr;
use std::fs;
use std::os::raw::c_char;
use std::path::Path;

#[no_mangle]
pub unsafe fn delete_all_pastes(path: *const c_char) {
  let path = CStr::from_ptr(path).to_string_lossy();

  do_the_thing(&path);
}

fn do_the_thing(path: &str) {
  let path = Path::new(&path);
  if !path.exists() || !path.is_dir() {
    return;
  }

  fs::remove_dir_all(path).unwrap();
}
