fn main() {}

#[ragna::gpu]
mod gpu {
    fn loops() {
        'label: while true {}
    }
}
