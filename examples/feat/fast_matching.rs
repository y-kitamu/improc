use std::thread;

struct Sample {
    vec: Vec<u32>,
}

impl Sample {
    fn sample(&mut self) {
        self.vec.push(10);
    }
}

fn main() {
    let mut sample = Sample { vec: Vec::new() };
}
