use std::io::{self, Read as _};

fn main() {
    let mut input = "".to_owned();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut input = input.split_whitespace();
    macro_rules! read(
        ([$tt:tt]) => (read!([$tt; read!(usize)]));
        ([$tt:tt; $n:expr]) => ((0..$n).map(|_| read!($tt)).collect::<Vec<_>>());
        (($($tt:tt),+)) => (($(read!($tt)),*));
        (Usize1) => (read!(usize) - 1);
        (Byte) => (read!(char) as u8);
        (Bytes) => (read!(String).into_bytes());
        ($ty:ty) => (input.next().unwrap().parse::<$ty>().unwrap());
    );

    let n = read!(usize);
}
