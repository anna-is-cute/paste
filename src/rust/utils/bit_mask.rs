use std::net::{IpAddr, Ipv6Addr, Ipv4Addr};

pub trait BitMask {
  fn to_masked(&self, mask: u8) -> Self;
}

impl BitMask for Ipv6Addr {
  fn to_masked(&self, mask: u8) -> Self {
    if mask > 128 {
      panic!("invalid mask: {}", mask);
    }

    let mut octets = [0u8; 16];

    let mut loc = 0;

    for octet in &mut self.octets() {
      loc += 8;
      if loc <= mask {
        octets[(loc as usize / 8) - 1] = *octet;
      } else if loc > mask && loc - mask < 8 {
        for i in 0..loc - mask {
          *octet &= !(1 << i);
        }
        octets[(loc as usize / 8) - 1] = *octet;
      }
    }

    octets.into()
  }
}

impl BitMask for Ipv4Addr {
  fn to_masked(&self, mask: u8) -> Self {
    if mask > 32 {
      panic!("invalid mask: {}", mask);
    }

    let mut octets = [0u8; 4];

    let mut loc = 0;

    for octet in &mut self.octets() {
      loc += 8;
      if loc <= mask {
        octets[(loc as usize / 8) - 1] = *octet;
      } else if loc > mask && loc - mask < 8 {
        for i in 0..loc - mask {
          *octet &= !(1 << i);
        }
        octets[(loc as usize / 8) - 1] = *octet;
      }
    }

    octets.into()
  }
}

impl BitMask for IpAddr {
  fn to_masked(&self, mask: u8) -> Self {
    match *self {
      IpAddr::V4(a) => a.to_masked(mask).into(),
      IpAddr::V6(a) => a.to_masked(mask).into(),
    }
  }
}
