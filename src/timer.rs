use core::{time::Duration, ops::Add};
// from https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/blob/master/07_timestamps/src/_arch/aarch64/time.rs

use aarch64_cpu::{registers::{CNTFRQ_EL0, CNTPCT_EL0}, asm::barrier};
use tock_registers::interfaces::Readable;

#[derive(Copy, Clone, PartialOrd, PartialEq)]
struct GenericTimerCounterValue(u64);

impl GenericTimerCounterValue {
    pub const MAX: Self = GenericTimerCounterValue(u64::MAX);

    #[inline(always)]
    fn frequency() -> u64 {
        // CNTFRQ_EL0.get()
        500_000_000/10
    }
}

impl Add for GenericTimerCounterValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        GenericTimerCounterValue(self.0.wrapping_add(other.0))
    }
}

const NANOSEC_PER_SEC: u64 = 1_000_000_000;

impl From<GenericTimerCounterValue> for Duration {
    fn from(counter_value: GenericTimerCounterValue) -> Self {
        if counter_value.0 == 0 {
            return Duration::ZERO;
        }

        let frequency: u64 = GenericTimerCounterValue::frequency();

        // Div<NonZeroU64> implementation for u64 cannot panic.
        let secs = counter_value.0 / GenericTimerCounterValue::frequency();

        // This is safe, because frequency can never be greater than u32::MAX, which means the
        // largest theoretical value for sub_second_counter_value is (u32::MAX - 1). Therefore,
        // (sub_second_counter_value * NANOSEC_PER_SEC) cannot overflow an u64.
        //
        // The subsequent division ensures the result fits into u32, since the max result is smaller
        // than NANOSEC_PER_SEC. Therefore, just cast it to u32 using `as`.
        let sub_second_counter_value = counter_value.0 % frequency;
        let nanos: u32 = (sub_second_counter_value * NANOSEC_PER_SEC / frequency).try_into().unwrap();

        Duration::new(secs, nanos)
    }
}

pub fn resolution() -> Duration {
    Duration::from(GenericTimerCounterValue(1))
}

fn max_duration() -> Duration {
    Duration::from(GenericTimerCounterValue::MAX)
}

impl TryFrom<Duration> for GenericTimerCounterValue {
    type Error = &'static str;

    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        if duration < resolution() {
            return Ok(GenericTimerCounterValue(0));
        }

        if duration > max_duration() {
            return Err("Conversion error. Duration too big");
        }

        let frequency = GenericTimerCounterValue::frequency();
        let duration: u128 = duration.as_nanos();

        // This is safe, because frequency can never be greater than u32::MAX, and
        // (Duration::MAX.as_nanos() * u32::MAX) < u128::MAX.
        let counter_value = duration * u128::from(frequency) / u128::from(NANOSEC_PER_SEC);

        // Since we checked above that we are <= max_duration(), just cast to u64.
        Ok(GenericTimerCounterValue(counter_value.try_into().unwrap()))
    }
}

#[inline(always)]
fn read_cntpct() -> GenericTimerCounterValue {
    // Prevent that the counter is read ahead of time due to out-of-order execution.
    barrier::isb(barrier::SY);
    GenericTimerCounterValue(CNTPCT_EL0.get())
}

/// The uptime since power-on of the device.
///
/// This includes time consumed by firmware and bootloaders.
pub fn uptime() -> Duration {
    read_cntpct().into()
}

/// Spin for a given duration.
pub fn spin_for(duration: Duration) {
    let curr_counter_value = read_cntpct();

    let counter_value_delta: GenericTimerCounterValue = duration.try_into().unwrap_or(GenericTimerCounterValue::MAX);
    let counter_value_target = curr_counter_value + counter_value_delta;

    // Busy wait.
    //
    // Read CNTPCT_EL0 directly to avoid the ISB that is part of [`read_cntpct`].
    while GenericTimerCounterValue(CNTPCT_EL0.get()) < counter_value_target {}
}
