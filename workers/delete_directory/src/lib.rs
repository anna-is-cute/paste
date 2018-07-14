use std::ffi::CStr;
use std::fs;
use std::os::raw::c_char;
use std::path::Path;

#[no_mangle]
pub unsafe fn delete_directory(path: *const c_char) {
  let path = CStr::from_ptr(path).to_string_lossy();

  do_the_thing(&path);
}

fn do_the_thing(path: &str) {
  let path = Path::new(&path);
  if !path.exists() || !path.is_dir() {
    return;
  }

  if let Err(e) = fs::remove_dir_all(path) {
    eprintln!("could not delete {}: {}", path.to_string_lossy(), e);
  }
}
