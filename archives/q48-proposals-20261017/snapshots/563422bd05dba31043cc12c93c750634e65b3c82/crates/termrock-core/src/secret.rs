//! Memory zeroization helper.

pub fn wipe_string(value: String) {
    let mut bytes = value.into_bytes();
    let capacity = bytes.capacity();
    bytes.resize(capacity, 0);
    bytes.fill(0);
    core::hint::black_box(&bytes);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    drop(bytes);
}
