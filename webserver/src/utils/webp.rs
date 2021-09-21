use image::DynamicImage;

use std::convert::TryFrom;

pub fn convert(image: &DynamicImage, quality: f32) -> Option<Vec<u8>> {
  // convert the image to RGBA
  let image = image.to_rgba8();
  // get the dimensions of the image
  let (w, h) = image.dimensions();
  let w = i32::try_from(w).ok()?;
  let h = i32::try_from(h).ok()?;

  // get the flat samples of the image
  let flat = image.as_flat_samples();
  let samples: &[u8] = flat.samples;

  // libwebp will set this null pointer to the start of a buffer
  let mut output: *mut u8 = std::ptr::null_mut();

  // encode the image and get its length
  let len = unsafe {
    libwebp_sys::WebPEncodeRGBA(
      samples.as_ptr(),
      w,
      h,
      w * 4,
      quality,
      &mut output as *mut _,
    )
  };

  // if the length was 0 or the output pointer is still null, there was an error
  if len == 0 || output.is_null() {
    return None;
  }

  // create a slice from the output
  let slice = unsafe { std::slice::from_raw_parts(output, len) };
  // copy the memory into a vec
  let vec = slice.to_vec();
  // free the memory
  unsafe {
    libwebp_sys::WebPFree(output as *mut _);
  }
  // return the vec
  Some(vec)
}
