use uuid::Uuid;

pub fn halfuuid(uuid: Uuid) -> u64 {
    let mut b = [0; 8];
    b.copy_from_slice(&uuid.as_u128().to_be_bytes()[8..16]);
    u64::from_be_bytes(b)
}

pub fn fix(x: f64, min: f64, max: f64) -> f64 {
    debug_assert!(min < max);
    (x * (max - min) + min).max(0.0).min(1.0)
}

#[cfg(test)]
#[test]
fn test_fix() {
    use assert_approx_eq::assert_approx_eq;

    assert_approx_eq!(fix(0.0, 0.1, 0.9), 0.1);
    assert_approx_eq!(fix(0.5, 0.1, 0.9), 0.5);
    assert_approx_eq!(fix(1.1, 0.1, 0.9), 0.98);
    assert_approx_eq!(fix(0.5, 0.1, 0.5), 0.3);
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug, Default, Clone, Copy)]
pub struct AwayHome<T> {
    pub away: T,
    pub home: T,
}

impl<T> AwayHome<T> {
    pub fn map_opt<F, U>(&self, f: F) -> Option<AwayHome<U>>
    where
        F: Fn(&T) -> Option<U>,
    {
        Some(AwayHome {
            away: f(&self.away)?,
            home: f(&self.home)?,
        })
    }
}
