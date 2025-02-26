fn main() {}

#[ragna::gpu]
mod gpu {
    fn loops() {
        'label: while true {
            continue 'label;
            break 'label;
            break 0;
        }
        'label: for i in 0u..1u {}
        continue;
        break;
        for (a, b, c) in 0u..1u {}
    }
}
