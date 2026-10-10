use tracing::trace;

pub fn retrieve_active_bitfields<T: buffa::Enumeration>(bitfield: i64) -> impl Iterator<Item = T> {
    let mut copy = bitfield;

    std::iter::from_fn(move || {
        while copy != 0 {
            let index = copy.trailing_zeros();
            copy &= copy - 1;

            if let Some(role) = T::from_i32(index as i32) {
                return Some(role);
            } else {
                trace!(enum_type = %std::any::type_name::<T>(), "Unknown index {} in bitfield {}", index, bitfield);
            }
        }
        None
    })
}

pub fn compile_bitfield<T: buffa::Enumeration>(items: impl IntoIterator<Item = T>) -> i64 {
    items
        .into_iter()
        .fold(0, |acc, item| acc | (1 << T::to_i32(&item) as i64))
}
