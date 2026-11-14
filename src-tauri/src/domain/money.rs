use super::ModelError;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Self = Self(0);

    pub const fn from_minor(minor: i64) -> Self {
        Self(minor)
    }

    pub const fn minor(self) -> i64 {
        self.0
    }

    pub fn checked_add(self, other: Self) -> Result<Self, ModelError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(ModelError::Overflow)
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, ModelError> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or(ModelError::Overflow)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Balance {
    price: Money,
    receipts: Money,
    adjustments: Money,
    refunds: Money,
    net: Money,
}

impl Balance {
    pub fn new(
        price: Money,
        receipts: Money,
        adjustments: Money,
        refunds: Money,
    ) -> Result<Self, ModelError> {
        let net = receipts.checked_add(adjustments)?.checked_sub(refunds)?;
        if price.minor() < 0 || receipts.minor() < 0 || refunds.minor() < 0 || net.minor() < 0 {
            return Err(ModelError::InvalidAmount);
        }
        Ok(Self {
            price,
            receipts,
            adjustments,
            refunds,
            net,
        })
    }

    pub const fn price(self) -> Money {
        self.price
    }
    pub const fn receipts(self) -> Money {
        self.receipts
    }
    pub const fn adjustments(self) -> Money {
        self.adjustments
    }
    pub const fn refunds(self) -> Money {
        self.refunds
    }
    pub const fn net(self) -> Money {
        self.net
    }
    pub fn pending(self) -> Money {
        Money::from_minor((self.price.minor() - self.net.minor()).max(0))
    }
    pub fn credit(self) -> Money {
        Money::from_minor((self.net.minor() - self.price.minor()).max(0))
    }
    pub fn is_paid(self) -> bool {
        self.net >= self.price
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credit_adjustment_refund_and_overflow_use_integer_arithmetic() {
        let m = Money::from_minor;
        let credit = Balance::new(m(60_000), m(75_000), m(0), m(0)).unwrap();
        assert_eq!(credit.credit(), m(15_000));
        assert_eq!(credit.pending(), Money::ZERO);
        let corrected = Balance::new(m(60_000), m(50_000), m(-20_000), m(0)).unwrap();
        assert_eq!(corrected.net(), m(30_000));
        assert_eq!(corrected.pending(), m(30_000));
        assert_eq!(
            Balance::new(m(60_000), m(75_000), m(0), m(15_000))
                .unwrap()
                .credit(),
            Money::ZERO
        );
        assert_eq!(
            Balance::new(m(0), m(i64::MAX), m(1), m(0)),
            Err(ModelError::Overflow)
        );
        assert_eq!(
            Balance::new(m(0), m(1), m(0), m(2)),
            Err(ModelError::InvalidAmount)
        );
        assert_eq!(m(i64::MIN).checked_sub(m(1)), Err(ModelError::Overflow));
        assert!(Balance::new(m(0), m(0), m(0), m(0)).unwrap().is_paid());
    }
}
