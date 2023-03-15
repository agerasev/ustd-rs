use core::time::Duration;
use freertos::Duration as FreeRtosDuration;

pub(crate) trait IntoFreertos<T> {
    fn into_freertos(self) -> T;
}

impl IntoFreertos<FreeRtosDuration> for Option<Duration> {
    fn into_freertos(self) -> FreeRtosDuration {
        match self {
            Some(t) => {
                assert!(t.as_millis() < u32::MAX as u128);
                FreeRtosDuration::ms(t.as_millis() as u32)
            }
            None => FreeRtosDuration::infinite(),
        }
    }
}
