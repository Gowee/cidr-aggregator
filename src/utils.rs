use num_traits::PrimInt;

pub trait MathLog2 {
    // We follow the return type convention used in `.leading_zeros`.
    // https://users.rust-lang.org/t/why-the-return-type-of-int-leading-zeros-is-u32-of-u8/
    fn log2(self) -> u32;

    fn checked_log2(self) -> Option<u32>;
}

impl<T: PrimInt> MathLog2 for T {
    fn log2(self) -> u32 {
        std::mem::size_of::<Self>() as u32 * 8 - self.leading_zeros() - 1
    }

    fn checked_log2(self) -> Option<u32> {
        if self.count_ones() == 1 {
            Some(self.log2())
        } else {
            None
        }
    }
}

// macro_rules! implement_log2 {
//     ($int: ident) => {
//         impl MathLog2 for $int {
//             fn log2(self) -> u32 {
//                 // https://users.rust-lang.org/t/logarithm-of-integers/8506/5
//                 std::mem::size_of::<Self>() as u32 * 8 - self.leading_zeros() - 1
//             }
//         }
//     };
// }

// implement_log2!(u8);
// implement_log2!(u32);
// implement_log2!(u128);
