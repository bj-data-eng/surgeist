//! Invert signed ICC colorants without rounding away a nonsingular matrix.

use super::{Account, Result, sample_invariant};

const FIXED: f64 = 65536.0;
// Scaling by this power of two keeps even binary64 subnormal inputs and
// their exact product residuals normal. Qualified native XYZ coordinates,
// including the indirect absolute adjustment, remain far below overflow.
const XYZ_SCALE: f64 = (1_u128 << 120) as f64;
const COEFFICIENT_CHUNK: i128 = 1 << 32;

pub(super) struct InverseMatrix {
    cofactors: [[i128; 3]; 3],
    determinant: i128,
}

impl InverseMatrix {
    pub(super) fn from_columns(columns: [[f64; 3]; 3], account: &Account) -> Result<Option<Self>> {
        // Signed s15Fixed16 values have exact binary64 representations.
        // Their pair products and three-term determinant fit in i128.
        let mut codes = [[0_i128; 3]; 3];
        for (code, component) in codes
            .iter_mut()
            .flatten()
            .zip(columns.into_iter().flatten())
        {
            *code = account.coordinate(|| (component * FIXED) as i128)?;
        }
        let [r, g, b] = codes;
        let cofactors = [
            cross(g, b, account)?,
            cross(b, r, account)?,
            cross(r, g, account)?,
        ];
        let mut determinant = 0_i128;
        for (code, cofactor) in r.into_iter().zip(cofactors[0]) {
            determinant = account.coordinate(|| determinant + code * cofactor)?;
        }
        Ok((determinant != 0).then_some(Self {
            cofactors,
            determinant,
        }))
    }

    pub(super) fn linear_controls(&self, mut xyz: [f64; 3], account: &Account) -> Result<[f64; 3]> {
        for value in &mut xyz {
            *value = account.coordinate(|| *value * XYZ_SCALE)?;
        }
        let mut controls = [0.0; 3];
        for (control, cofactors) in controls.iter_mut().zip(self.cofactors) {
            let mut numerator = Expansion::default();
            for (cofactor, xyz) in cofactors.into_iter().zip(xyz) {
                // Each chunk is exactly representable before multiplication.
                let low = cofactor % COEFFICIENT_CHUNK;
                let high = cofactor - low;
                numerator.add_product(high as f64, xyz, account)?;
                numerator.add_product(low as f64, xyz, account)?;
            }
            let numerator = numerator.estimate(account)?;
            *control = account.coordinate(|| {
                let control = (numerator / self.determinant as f64) * (FIXED / XYZ_SCALE);
                control.clamp(0.0, 1.0)
            })?;
        }
        Ok(controls)
    }
}

fn cross([a0, a1, a2]: [i128; 3], [b0, b1, b2]: [i128; 3], account: &Account) -> Result<[i128; 3]> {
    Ok([
        account.coordinate(|| a1 * b2 - a2 * b1)?,
        account.coordinate(|| a2 * b0 - a0 * b2)?,
        account.coordinate(|| a0 * b1 - a1 * b0)?,
    ])
}

// Three pairs of coefficient chunks contribute one product and one exact
// product residual each. Their expansion stays ordered least to most
// significant, and collapses to binary64 only after cancellation.
#[derive(Default)]
struct Expansion {
    parts: [f64; 12],
    length: usize,
}

impl Expansion {
    fn add_product(&mut self, coefficient: f64, value: f64, account: &Account) -> Result<()> {
        if coefficient == 0.0 || value == 0.0 {
            return Ok(());
        }
        let (product, residual) = account.coordinate(|| {
            let product = coefficient * value;
            (product, coefficient.mul_add(value, -product))
        })?;
        self.add(residual, account)?;
        self.add(product, account)
    }

    fn add(&mut self, mut value: f64, account: &Account) -> Result<()> {
        if value == 0.0 {
            return Ok(());
        }
        let mut length = 0;
        for index in 0..self.length {
            let part = self.parts[index];
            let (sum, residual) = account.coordinate(|| {
                let sum = value + part;
                let rounded_part = sum - value;
                let error = (value - (sum - rounded_part)) + (part - rounded_part);
                (sum, error)
            })?;
            if residual != 0.0 {
                self.parts[length] = residual;
                length += 1;
            }
            value = sum;
        }
        if value != 0.0 {
            *self.parts.get_mut(length).ok_or_else(sample_invariant)? = value;
            length += 1;
        }
        self.length = length;
        Ok(())
    }

    fn estimate(&self, account: &Account) -> Result<f64> {
        let mut sum = 0.0;
        for &part in &self.parts[..self.length] {
            sum = account.coordinate(|| sum + part)?;
        }
        Ok(sum)
    }
}
