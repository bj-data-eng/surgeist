//! Centrally admitted resolved convolution parameters. Production pixels remain GPU-owned.
use crate::accounting::{
    Account, AccountedVec, RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderWork,
    SharedValue,
};
use crate::{Error, FilterDomains, Result};

use crate::{FilterColorSpace, FilterEdgeMode};

/// Parameter whose resolved value is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConvolutionParameter {
    /// Numeric horizontal order before truncation.
    OrderX,
    /// Numeric vertical order before truncation.
    OrderY,
    /// Horizontal integer target position.
    TargetX,
    /// Vertical integer target position.
    TargetY,
    /// One row-major kernel coefficient.
    Coefficient,
    /// An explicitly supplied divisor.
    Divisor,
    /// The primitive bias.
    Bias,
}

/// Numeric operation that could not be represented safely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConvolutionNumericPhase {
    /// Checked order product or coefficient backing size.
    TapCount,
    /// Row-major sum requested by the default divisor.
    DivisorSum,
    /// Host division and zero-or-normal float32 coefficient admission.
    WeightNormalization,
    /// Zero-or-normal float32 bias admission.
    BiasNarrowing,
    /// Conservative signed accumulation and bias overflow proof.
    AccumulationBound,
    /// Checked source, result or neighborhood lattice indexing.
    GridIndex,
    /// Representability of the local-coordinate to physical-grid mapping.
    GridMapping,
}

/// Typed refusal at the resolved convolution preparation boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConvolutionPreparationError {
    /// A reached resolved parameter is outside its valid domain.
    InvalidParameter {
        /// Parameter being validated.
        parameter: ConvolutionParameter,
        /// Supplied resolved value.
        value: f64,
    },
    /// Required numeric work cannot be represented under the native float32 policy.
    Numeric {
        /// Operation at which representation was refused.
        phase: ConvolutionNumericPhase,
        /// Supplied or intermediate value at that boundary.
        value: f64,
    },
}

/// Borrowed untrusted numeric attributes. Builders retain no coefficient allocation.
#[derive(Clone, Copy, Debug)]
pub struct ConvolutionInput<'a> {
    kernel: &'a [f64],
    order: Option<[f64; 2]>,
    target: [Option<f64>; 2],
    divisor: Option<f64>,
    bias: f64,
    preserve_alpha: bool,
    edge_mode: FilterEdgeMode,
    unit_length: Option<[f64; 2]>,
}
impl<'a> ConvolutionInput<'a> {
    /// Requests the standard 3×3 order, centered target, default divisor, zero bias and duplicate edges.
    #[must_use]
    pub const fn new(kernel: &'a [f64]) -> Self {
        Self {
            kernel,
            order: None,
            target: [None; 2],
            divisor: None,
            bias: 0.0,
            preserve_alpha: false,
            edge_mode: FilterEdgeMode::Duplicate,
            unit_length: None,
        }
    }
    /// Sets numeric order; an omitted second coordinate copies the first. Preparation truncates toward zero.
    #[must_use]
    pub fn with_order(mut self, x: f64, y: Option<f64>) -> Self {
        self.order = Some([x, y.unwrap_or(x)]);
        self
    }
    /// Sets each target independently; omitted coordinates use floor(order/2).
    #[must_use]
    pub const fn with_target(mut self, x: Option<f64>, y: Option<f64>) -> Self {
        self.target = [x, y];
        self
    }
    /// Sets the divisor. Zero requests the checked coefficient sum, or one if that sum is zero.
    #[must_use]
    pub const fn with_divisor(mut self, divisor: f64) -> Self {
        self.divisor = Some(divisor);
        self
    }
    /// Sets bias in the selected working color space.
    #[must_use]
    pub const fn with_bias(mut self, bias: f64) -> Self {
        self.bias = bias;
        self
    }
    /// Selects straight-RGB convolution with preserved kernel-grid source alpha.
    #[must_use]
    pub const fn with_preserve_alpha(mut self, preserve: bool) -> Self {
        self.preserve_alpha = preserve;
        self
    }
    /// Selects addressing outside the semantic source domain.
    #[must_use]
    pub const fn with_edge_mode(mut self, edge: FilterEdgeMode) -> Self {
        self.edge_mode = edge;
        self
    }
    /// Requests an explicit kernel lattice. Nonfinite or nonpositive coordinates select the source lattice.
    #[must_use]
    pub fn with_unit_length(mut self, x: f64, y: Option<f64>) -> Self {
        self.unit_length = Some([x, y.unwrap_or(x)]);
        self
    }
}

/// Immutable admitted coefficients. Clones share backing and retain its admission.
#[derive(Clone, Debug)]
pub struct PreparedConvolution {
    data: SharedValue<ConvolutionData>,
    _import: Option<SharedValue<()>>,
}
impl PartialEq for PreparedConvolution {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
#[derive(Debug, PartialEq)]
struct ConvolutionData {
    domains: FilterDomains,
    order: [u32; 2],
    operation: PreparedOperation,
}
#[derive(Debug, PartialEq)]
enum PreparedOperation {
    CountMismatchPassThrough,
    Executable {
        weights: AccountedVec<f32>,
        target: [u32; 2],
        bias: f32,
        preserve_alpha: bool,
        edge_mode: FilterEdgeMode,
        color_space: FilterColorSpace,
        unit_length: Option<[f64; 2]>,
    },
}
impl PreparedConvolution {
    pub(crate) fn payload_clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            _import: None,
        }
    }
    pub(crate) fn with_import(
        &self,
        import: Option<crate::accounting::CpuImport>,
        account: &Account,
    ) -> Result<Self> {
        Ok(Self {
            data: self.data.clone(),
            _import: Some(SharedValue::with_parts(
                (),
                None,
                import,
                &account.storage(),
            )?),
        })
    }
    pub(crate) fn domains(&self) -> FilterDomains {
        self.data.domains
    }
    pub(crate) fn order(&self) -> [u32; 2] {
        self.data.order
    }
    pub(crate) fn is_passthrough(&self) -> bool {
        matches!(
            self.data.operation,
            PreparedOperation::CountMismatchPassThrough
        )
    }
    pub(crate) fn weights(&self) -> &[f32] {
        match &self.data.operation {
            PreparedOperation::Executable { weights, .. } => weights,
            _ => &[],
        }
    }
    pub(crate) fn target(&self) -> [u32; 2] {
        match &self.data.operation {
            PreparedOperation::Executable { target, .. } => *target,
            _ => [0; 2],
        }
    }
    pub(crate) fn bias(&self) -> f32 {
        match &self.data.operation {
            PreparedOperation::Executable { bias, .. } => *bias,
            _ => 0.0,
        }
    }
    pub(crate) fn preserve_alpha(&self) -> bool {
        match &self.data.operation {
            PreparedOperation::Executable { preserve_alpha, .. } => *preserve_alpha,
            _ => false,
        }
    }
    pub(crate) fn edge_mode(&self) -> FilterEdgeMode {
        match &self.data.operation {
            PreparedOperation::Executable { edge_mode, .. } => *edge_mode,
            _ => FilterEdgeMode::None,
        }
    }
    pub(crate) fn color_space(&self) -> FilterColorSpace {
        match &self.data.operation {
            PreparedOperation::Executable { color_space, .. } => *color_space,
            _ => FilterColorSpace::Srgb,
        }
    }
    pub(crate) fn unit_length(&self) -> Option<[f64; 2]> {
        match &self.data.operation {
            PreparedOperation::Executable { unit_length, .. } => *unit_length,
            _ => None,
        }
    }
    pub(crate) fn same_allocation(&self, other: &Self) -> bool {
        self.data.same_allocation(&other.data)
    }
    pub(crate) fn own_backing_bytes(&self) -> Result<usize> {
        let capacity = match &self.data.operation {
            PreparedOperation::Executable { weights, .. } => weights.capacity(),
            _ => 0,
        };
        SharedValue::<ConvolutionData>::allocation_size()
            .checked_add(crate::accounting::bytes::<f32>(capacity)?)
            .ok_or_else(|| numeric(ConvolutionNumericPhase::TapCount, capacity as f64))
    }
}

/// Validates and admits one resolved convolution. Count mismatch is a clipped pass-through.
/// Numerical refusal preserves the previous completed pixels when used by a renderer.
/// Coefficients remain in original row-major order; execution rotates the kernel.
/// Division is performed in host f64, then each nonzero uploaded weight and bias
/// must be a finite normal f32. Conservative range admission rejects overflow.
/// Ordinary GPU rounding and intermediate subnormal flushing are permitted;
/// this contract promises neither universal subnormal preservation nor a global
/// error bound against real arithmetic. No device identity is retained here.
pub fn prepare_convolution(
    input: ConvolutionInput<'_>,
    domains: FilterDomains,
    color_space: FilterColorSpace,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedConvolution> {
    accounting.run(
        RenderAttemptKind::Convolution,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                account.work(RenderWork::FilterOperationVisits, 1)?;
                for rect in [domains.source(), domains.filter(), domains.result()] {
                    for value in [
                        rect.origin().x(),
                        rect.origin().y(),
                        rect.width(),
                        rect.height(),
                        rect.max().x(),
                        rect.max().y(),
                    ] {
                        account.work(RenderWork::CoordinateVisits, 1)?;
                        if !value.is_finite() {
                            return Err(Error::invalid_value(
                                "filter domain coordinate",
                                value,
                                "must be finite",
                            ));
                        }
                    }
                }
                let raw = input.order.unwrap_or([3.0; 2]);
                let order = [
                    order(raw[0], ConvolutionParameter::OrderX, account)?,
                    order(raw[1], ConvolutionParameter::OrderY, account)?,
                ];
                let count = usize::try_from(order[0])
                    .ok()
                    .and_then(|x| {
                        usize::try_from(order[1])
                            .ok()
                            .and_then(|y| x.checked_mul(y))
                    })
                    .filter(|count| *count <= isize::MAX as usize / std::mem::size_of::<f32>())
                    .ok_or_else(|| {
                        numeric(
                            ConvolutionNumericPhase::TapCount,
                            f64::from(order[0]) * f64::from(order[1]),
                        )
                    })?;
                let operation = if count != input.kernel.len() {
                    PreparedOperation::CountMismatchPassThrough
                } else {
                    let target = [
                        target(
                            input.target[0],
                            order[0],
                            ConvolutionParameter::TargetX,
                            account,
                        )?,
                        target(
                            input.target[1],
                            order[1],
                            ConvolutionParameter::TargetY,
                            account,
                        )?,
                    ];
                    account.work(RenderWork::CoordinateVisits, 1)?;
                    let use_default = input.divisor.is_none_or(|value| value == 0.0);
                    let mut divisor = input.divisor.unwrap_or(0.0);
                    if !divisor.is_finite() {
                        return Err(invalid(ConvolutionParameter::Divisor, divisor));
                    }
                    // Only default requests calculate the original sum; explicit divisors must not reject an unused overflowing sum.
                    for &coefficient in input.kernel {
                        account.work(RenderWork::KernelSampleVisits, 1)?;
                        if !coefficient.is_finite() {
                            return Err(invalid(ConvolutionParameter::Coefficient, coefficient));
                        }
                        if use_default {
                            divisor += coefficient;
                            if !divisor.is_finite() {
                                return Err(numeric(ConvolutionNumericPhase::DivisorSum, divisor));
                            }
                        }
                    }
                    if use_default && divisor == 0.0 {
                        divisor = 1.0;
                    }
                    let mut weights = account.retained(count)?;
                    for &coefficient in input.kernel {
                        account.work(RenderWork::KernelSampleVisits, 1)?;
                        let quotient = coefficient / divisor;
                        if coefficient != 0.0 && quotient == 0.0 {
                            return Err(numeric(
                                ConvolutionNumericPhase::WeightNormalization,
                                quotient,
                            ));
                        }
                        weights.push(narrow(
                            quotient,
                            ConvolutionNumericPhase::WeightNormalization,
                        )?)?;
                    }
                    account.work(RenderWork::CoordinateVisits, 1)?;
                    if !input.bias.is_finite() {
                        return Err(invalid(ConvolutionParameter::Bias, input.bias));
                    }
                    let bias = narrow(input.bias, ConvolutionNumericPhase::BiasNarrowing)?;
                    prove_accumulation(&weights, bias, account)?;
                    let unit_length = match input.unit_length {
                        Some(units) => {
                            account.work(RenderWork::CoordinateVisits, 1)?;
                            let valid_x = units[0].is_finite() && units[0] > 0.0;
                            account.work(RenderWork::CoordinateVisits, 1)?;
                            let valid_y = units[1].is_finite() && units[1] > 0.0;
                            (valid_x && valid_y).then_some(units)
                        }
                        None => None,
                    };
                    PreparedOperation::Executable {
                        weights,
                        target,
                        bias,
                        preserve_alpha: input.preserve_alpha,
                        edge_mode: input.edge_mode,
                        color_space,
                        unit_length,
                    }
                };
                Ok(PreparedConvolution {
                    data: SharedValue::new(
                        ConvolutionData {
                            domains,
                            order,
                            operation,
                        },
                        &account.storage(),
                    )?,
                    _import: None,
                })
            })
        },
    )
}
fn order(value: f64, parameter: ConvolutionParameter, account: &Account) -> Result<u32> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    let integer = value.trunc();
    if !value.is_finite() || integer < 1.0 || integer > f64::from(u32::MAX) {
        return Err(invalid(parameter, value));
    }
    Ok(integer as u32)
}
fn target(
    value: Option<f64>,
    order: u32,
    parameter: ConvolutionParameter,
    account: &Account,
) -> Result<u32> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    let value = value.unwrap_or(f64::from(order / 2));
    if !value.is_finite() || value < 0.0 || value >= f64::from(order) || value.fract() != 0.0 {
        return Err(invalid(parameter, value));
    }
    Ok(value as u32)
}
fn narrow(value: f64, phase: ConvolutionNumericPhase) -> Result<f32> {
    let narrowed = value as f32;
    if !value.is_finite() || (value != 0.0 && !narrowed.is_normal()) {
        return Err(numeric(phase, value));
    }
    Ok(narrowed)
}
fn prove_accumulation(weights: &[f32], bias: f32, account: &Account) -> Result<()> {
    let mut l1 = 0.0;
    for &weight in weights {
        account.work(RenderWork::KernelSampleVisits, 1)?;
        l1 += f64::from(weight).abs();
    }
    let taps = weights.len() as f64;
    let batches = (taps / 32.0).ceil();
    account.work(RenderWork::CoordinateVisits, 6)?;
    let r = (1.0 + 2.0f64.powi(-21)).powf(2.0 * taps + 2.0 * batches + 16.0);
    let u = r * l1;
    let ua = r * (u + f64::from(bias).abs());
    let uc = r * (u + f64::from(bias).abs() * ua);
    for value in [r, u, ua, uc] {
        if !value.is_finite() || value > f64::from(f32::MAX) / 4.0 {
            return Err(numeric(ConvolutionNumericPhase::AccumulationBound, value));
        }
    }
    Ok(())
}
fn invalid(parameter: ConvolutionParameter, value: f64) -> Error {
    ConvolutionPreparationError::InvalidParameter { parameter, value }.into()
}
fn numeric(phase: ConvolutionNumericPhase, value: f64) -> Error {
    ConvolutionPreparationError::Numeric { phase, value }.into()
}

#[cfg(test)]
mod tests;
