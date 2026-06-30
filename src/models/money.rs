#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    cents: i64,
}

impl Money {
    pub fn from_cents(cents: i64) -> Self {
        Self { cents }
    }

    pub fn cents(self) -> i64 {
        self.cents
    }

    pub fn parse_dollars(input: &str) -> Result<Self, String> {
        let value = input.trim().replace(',', "");

        if value.is_empty() {
            return Err("Amount is required.".to_string());
        }

        if value.starts_with('-') {
            return Err("Amount cannot be negative.".to_string());
        }

        let parts: Vec<&str> = value.split('.').collect();
        if parts.len() > 2 {
            return Err("Amount must be a valid number, for example 25 or 25.50.".to_string());
        }

        let dollars = if parts[0].is_empty() {
            0
        } else {
            parts[0]
                .parse::<i64>()
                .map_err(|_| "Amount must use digits only.".to_string())?
        };

        let cents = match parts.get(1) {
            None => 0,
            Some(value) if value.is_empty() => 0,
            Some(value) if value.len() == 1 => value
                .parse::<i64>()
                .map(|digit| digit * 10)
                .map_err(|_| "Cents must use digits only.".to_string())?,
            Some(value) if value.len() == 2 => value
                .parse::<i64>()
                .map_err(|_| "Cents must use digits only.".to_string())?,
            Some(_) => return Err("Use at most 2 decimal places.".to_string()),
        };

        let total_cents = dollars
            .checked_mul(100)
            .and_then(|base| base.checked_add(cents))
            .ok_or_else(|| "Amount is too large.".to_string())?;

        if total_cents <= 0 {
            return Err("Amount must be more than $0.00.".to_string());
        }

        if total_cents > 100_000_000_00 {
            return Err("Amount is above the allowed demo limit for this operation.".to_string());
        }

        Ok(Self::from_cents(total_cents))
    }

    pub fn display(self) -> String {
        let dollars = self.cents / 100;
        let cents = self.cents.abs() % 100;
        format!("${}.{:02}", dollars, cents)
    }
}
