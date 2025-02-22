fn main() {}

#[ragna::gpu]
mod gpu {
    fn loops() {
        'label: while true {
            break 'label;
            break 0;
        }
    }
}
