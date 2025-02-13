#![allow(dead_code)]
use std::fs::File;
use std::io::Read;

fn get_random_bytes(buf: &mut [u8]) {
    let mut file = File::open("/dev/urandom").expect("Failed to open /dev/urandom");
    file.read_exact(buf).expect("Failed to read random bytes");
}

fn random_1024_bit_odd() -> [u8; 128] {
    let mut num = [0u8; 128];
    get_random_bytes(&mut num);
    // set the first bit ensured number is odd
    num[0] |= 0b00000001;
    // set the last bit to 1 ensures  number  is 1024 bits
    num[127] |= 0b10000000;
    num
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct U1024 {
    limbs: [u64; 16],
}

impl U1024 {
    fn new(value: u64) -> Self {
        let mut limbs = [0u64; 16];
        limbs[0] = value;
        U1024 { limbs }
    }

    fn from_bytes(bytes: &[u8; 128]) -> Self {
        let mut limbs = [0u64; 16];
        for i in 0..16 {
            limbs[i] = u64::from_be_bytes([
                bytes[i * 8],
                bytes[i * 8 + 1],
                bytes[i * 8 + 2],
                bytes[i * 8 + 3],
                bytes[i * 8 + 4],
                bytes[i * 8 + 5],
                bytes[i * 8 + 6],
                bytes[i * 8 + 7],
            ]);
        }
        U1024 { limbs }
    }
}

impl U1024 {
    fn add(self, other: U1024) -> U1024 {
        let mut result = U1024::new(0);
        let mut carry = 0u64;

        for i in 0..16 {
            let (sum, overflow) = self.limbs[i].overflowing_add(other.limbs[i]);
            let (sum, overflow2) = sum.overflowing_add(carry);
            result.limbs[i] = sum;
            carry = if overflow || overflow2 { 1 } else { 0 };
        }

        result
    }
}

impl U1024 {
    fn subtract(self, other: U1024) -> U1024 {
        let mut result = U1024::new(0);
        let mut borrow = 0u64;

        for i in 0..16 {
            let (diff, overflow) = self.limbs[i].overflowing_sub(other.limbs[i]);
            let (diff, overflow2) = diff.overflowing_sub(borrow);
            result.limbs[i] = diff;
            borrow = (overflow || overflow2) as u64;
        }

        result
    }
}
impl U1024 {
    fn multiply(&self, other: &U1024) -> U1024 {
        let mut result = U1024::new(0);

        for i in 0..16 {
            let mut carry = 0u64;
            for j in 0..16 {
                let (prod_low, prod_carry) = self.limbs[i].overflowing_mul(other.limbs[j]);
                let (sum, overflow) = result.limbs[i + j].overflowing_add(prod_low);
                result.limbs[i + j] = sum + carry;
                carry = if prod_carry || overflow { 1 } else { 0 };
            }
        }

        result
    }
}

impl U1024 {
    fn modulus(self, modulus: U1024) -> U1024 {
        let mut remainder = self.clone();
        while remainder >= modulus {
            remainder = remainder.subtract(modulus.clone());
        }

        remainder
    }
}
impl U1024 {
    fn mod_exp(self, exponent: U1024, modulus: U1024) -> U1024 {
        let mut base = self.modulus(modulus.clone());
        let mut exp = exponent;
        let mut result = U1024::new(1);

        while exp != U1024::new(0) {
            if exp.limbs[0] % 2 == 1 {
                result = result.multiply(&base).modulus(modulus.clone());
            }
            base = base.multiply(&base).modulus(modulus.clone());
            exp = exp.subtract(U1024::new(1));
        }

        result
    }
}

impl U1024 {
    fn to_base10(&self) -> String {
        let mut result = String::new();
        let mut remainder = self.clone();

        while remainder != U1024::new(0) {
            let (quotient, digit) = remainder.divmod_by_10();
            result.push_str(&digit.to_string());
            remainder = quotient;
        }

        result.chars().rev().collect()
    }

    fn divmod_by_10(&self) -> (U1024, u64) {
        let mut quotient = U1024::new(0);
        let mut remainder = 0u64;

        for i in (0..16).rev() {
            let combined_value = (remainder as u128) << 64 | self.limbs[i] as u128;

            quotient.limbs[i] = (combined_value / 10) as u64;

            remainder = (combined_value % 10) as u64;
        }

        (quotient, remainder)
    }
}

fn main() {
    let p: U1024 = U1024::from_bytes(&random_1024_bit_odd());
    println!("{:?}", p.to_base10());
}
