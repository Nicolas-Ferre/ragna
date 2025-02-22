fn main() {}

#[ragna::gpu]
mod gpu {
    fn loops() {
        'outer: while true {}
    }
}
