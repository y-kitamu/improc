macro_rules! sample {
    ($e0:expr, $e1: expr) => {
        println!("{}, {}", $e0, $e1);
    };
}

fn main() {
    sample!(0, 0);

    let a: usize = 1;
    let b: usize = 2;

    let v = vec![1, 2, 3, 4, 5];
    let aa = &a;
    let bb = &b;

    let c = aa + bb;
    println!("c = {}, {}", c, v[0]);
}
