fn main() {}

#[ragna::gpu]
mod gpu {
    fn loops() {
        'label: while true {
            continue 'label;
            break 'label;
            break 0;
        }
        'label: for i in 0_u32..1_u32 {}
        continue;
        break;
    }
}
