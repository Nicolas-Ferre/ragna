fn main() {}

#[ragna::gpu]
mod gpu {
    fn loops() {
        'label: while true {
            continue 'label;
            break 'label;
            break 0;
        }
        continue;
        break;
    }
}
