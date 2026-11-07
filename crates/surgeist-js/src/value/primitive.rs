//! Context-free primitive semantics and optional host admission.
//!
//! BigInt prices are deterministic canonical-32-bit-word reference reservations,
//! not instruction counts, peak-memory guarantees, or mid-kernel cancellation.
//! The pinned provider is opaque: admission precedes entry and a checkpoint
//! follows it. Whole-kernel quotes include their inner kernels exactly once;
//! physical bridge copies are reserved separately.

use std::{collections::TryReserveError, convert::Infallible, hash::Hasher};

use num_traits::ToPrimitive;

use crate::{
    JsBigInt, JsError, JsResult, JsString, JsValue, JsVariant,
    builtins::{
        Number,
        number::{f64_to_int32, f64_to_uint32},
    },
    error::JsNativeError,
    js_string,
    string::{StringError, StringWork},
    value::{AbstractRelation, Numeric},
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum PrimitiveWork {
    Checkpoint,
    String(StringWork),
    BigIntCopy { words: u64 },
    BigIntRead { words: u64 },
    BigIntCapacity { bytes: u64 },
    BigIntKernel { units: u64 },
    BigIntConvert { units: u64 },
}

#[derive(Debug)]
pub(crate) enum PrimitiveError<E> {
    Abrupt(JsError),
    Refused(E),
    WorkOverflow,
    Allocation(TryReserveError),
}

impl<E> From<JsError> for PrimitiveError<E> {
    fn from(error: JsError) -> Self {
        Self::Abrupt(error)
    }
}

impl<E> From<JsNativeError> for PrimitiveError<E> {
    fn from(error: JsNativeError) -> Self {
        Self::Abrupt(error.into())
    }
}

impl<E> From<StringError<E>> for PrimitiveError<E> {
    fn from(error: StringError<E>) -> Self {
        match error {
            StringError::Refused(error) => Self::Refused(error),
            StringError::SizeOverflow => Self::WorkOverflow,
            StringError::Allocation(error) => Self::Allocation(error),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PrimitiveUnaryOp {
    Plus,
    Minus,
    Not,
    BitNot,
    TypeOf,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PrimitiveBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Ushr,
    Eq,
    NotEq,
    StrictEq,
    StrictNotEq,
    Lt,
    Le,
    Gt,
    Ge,
}

struct PrimitiveContext<'a, F, const METERED: bool> {
    admit: &'a mut F,
}

type PrimitiveResult<T, E> = Result<T, PrimitiveError<E>>;

fn checked<E>(value: Option<u64>) -> PrimitiveResult<u64, E> {
    value.ok_or(PrimitiveError::WorkOverflow)
}

fn words<E>(bits: u64) -> PrimitiveResult<u64, E> {
    checked(bits.checked_add(31)).map(|n| n / 32)
}

fn primitive_request<E>(
    admit: &mut (impl FnMut(PrimitiveWork) -> Result<(), E> + ?Sized),
    work: PrimitiveWork,
) -> PrimitiveResult<(), E> {
    admit(work).map_err(PrimitiveError::Refused)
}

/// Obtain a borrowed canonical iterator without an unadmitted metadata read.
/// Locked num-bigint 0.5.1's U32Digits::new checks the highest native word on
/// 64-bit targets to omit an empty high half. Reserve one canonical metadata
/// word for a nonzero operand before this constant-time iterator construction;
/// the 32-bit layout may require less. No magnitude walk or allocation occurs.
fn canonical_bigint_digits<'a, E>(
    raw: &'a num_bigint::BigInt,
    admit: &mut (impl FnMut(PrimitiveWork) -> Result<(), E> + ?Sized),
) -> PrimitiveResult<num_bigint::U32Digits<'a>, E> {
    if raw.sign() != num_bigint::Sign::NoSign {
        primitive_request(admit, PrimitiveWork::BigIntRead { words: 1 })?;
    }
    Ok(raw.iter_u32_digits())
}

/// Copy a literal's raw magnitude into the shared JavaScript BigInt producer.
///
/// Locked num-bigint 0.5.1 clones BigInt -> BigUint -> BigDigits. Inline digits
/// copy directly; a heap magnitude clones its normalized native-digit slice
/// using Vec::to_vec (zero/one-native-digit shortcuts become inline). Canonical
/// W 32-bit digits therefore bound the source copy, and 4*round_even(W) bytes
/// bound the requested native magnitude payload across its 32/64-bit layouts.
/// W<=1 requires no heap magnitude payload. The RawBigInt descriptor is a
/// separate logical Rc payload; allocator headers and spare capacity are not
/// physical-memory measurements. Inline shortcuts may use less than reserved.
///
/// Both payload and copy requests precede the opaque clone/Rc allocation. The
/// provider has no fallible allocation or mid-call cancellation hook: this is
/// not a complete OOM, elapsed-time, or physical peak-memory guarantee. Accepted
/// reservations remain recorded if a later request or checkpoint refuses.
pub(crate) fn try_copy_raw_bigint<E>(
    raw: &num_bigint::BigInt,
    admit: &mut (impl FnMut(PrimitiveWork) -> Result<(), E> + ?Sized),
) -> PrimitiveResult<JsBigInt, E> {
    primitive_request(admit, PrimitiveWork::Checkpoint)?;
    let count = u64::try_from(canonical_bigint_digits(raw, admit)?.len())
        .map_err(|_| PrimitiveError::WorkOverflow)?;
    let payload = if count <= 1 {
        0
    } else {
        checked(
            count
                .checked_add(count % 2)
                .and_then(|words| words.checked_mul(4)),
        )?
    };
    let descriptor = u64::try_from(std::mem::size_of::<num_bigint::BigInt>())
        .map_err(|_| PrimitiveError::WorkOverflow)?;
    primitive_request(
        admit,
        PrimitiveWork::BigIntCapacity {
            bytes: checked(payload.checked_add(descriptor))?,
        },
    )?;
    primitive_request(admit, PrimitiveWork::BigIntCopy { words: count })?;
    let result = JsBigInt::new(raw.clone());
    primitive_request(admit, PrimitiveWork::Checkpoint)?;
    Ok(result)
}

/// Hash a compiler literal key by sign and canonical magnitude code words.
///
/// The protocol is write_u8(0=zero, 1=positive, 2=negative), write_u64(word
/// count), then write_u32 for each least-significant-first canonical word. It
/// intentionally does not reproduce the provider's native-word Hash protocol;
/// mathematically equal BigInts use identical writes on either native layout.
/// Each magnitude block of at most 256 words is admitted before reads/writes.
/// Hasher implementation work remains caller-owned; accepted writes are not
/// rolled back after refusal, including refusal at the final checkpoint.
pub(crate) fn try_bigint_hash<H: Hasher, E>(
    value: &JsBigInt,
    state: &mut H,
    admit: &mut (impl FnMut(PrimitiveWork) -> Result<(), E> + ?Sized),
) -> PrimitiveResult<(), E> {
    primitive_request(admit, PrimitiveWork::Checkpoint)?;
    let raw = value.as_inner();
    let sign = match raw.sign() {
        num_bigint::Sign::NoSign => 0,
        num_bigint::Sign::Plus => 1,
        num_bigint::Sign::Minus => 2,
    };
    let mut digits = canonical_bigint_digits(raw, admit)?;
    let mut remaining = digits.len();
    let count = u64::try_from(remaining).map_err(|_| PrimitiveError::WorkOverflow)?;
    state.write_u8(sign);
    state.write_u64(count);
    while remaining != 0 {
        let block = remaining.min(256);
        primitive_request(
            admit,
            PrimitiveWork::BigIntRead {
                words: block as u64,
            },
        )?;
        for _ in 0..block {
            state.write_u32(digits.next().expect("canonical digit count"));
        }
        remaining -= block;
    }
    primitive_request(admit, PrimitiveWork::Checkpoint)
}

/// Compare exact signs and canonical magnitudes without allocating or prewalking.
/// Metadata length mismatch requires no magnitude walk. Each actual word pair
/// reserves two reads before either iterator advances; the first mismatch stops
/// immediately. Initial/final checkpoints retain the caller's original refusal.
pub(crate) fn try_bigint_eq<E>(
    left: &JsBigInt,
    right: &JsBigInt,
    admit: &mut (impl FnMut(PrimitiveWork) -> Result<(), E> + ?Sized),
) -> PrimitiveResult<bool, E> {
    primitive_request(admit, PrimitiveWork::Checkpoint)?;
    if left.as_inner().sign() != right.as_inner().sign() {
        primitive_request(admit, PrimitiveWork::Checkpoint)?;
        return Ok(false);
    }
    let mut left = canonical_bigint_digits(left.as_inner(), admit)?;
    let mut right = canonical_bigint_digits(right.as_inner(), admit)?;
    if left.len() != right.len() {
        primitive_request(admit, PrimitiveWork::Checkpoint)?;
        return Ok(false);
    }
    for _ in 0..left.len() {
        primitive_request(admit, PrimitiveWork::BigIntRead { words: 2 })?;
        if left.next().expect("canonical digit count")
            != right.next().expect("canonical digit count")
        {
            primitive_request(admit, PrimitiveWork::Checkpoint)?;
            return Ok(false);
        }
    }
    primitive_request(admit, PrimitiveWork::Checkpoint)?;
    Ok(true)
}

fn mul_quote<E>(a: u64, b: u64) -> PrimitiveResult<u64, E> {
    let (x, y) = (words::<E>(a)?, words::<E>(b)?);
    let result = words::<E>(checked(a.checked_add(b))?)?;
    checked(
        x.checked_mul(y)
            .and_then(|n| n.checked_add(x))
            .and_then(|n| n.checked_add(y))
            .and_then(|n| n.checked_add(result)),
    )
}

impl<F, E, const METERED: bool> PrimitiveContext<'_, F, METERED>
where
    F: FnMut(PrimitiveWork) -> Result<(), E>,
{
    fn work(&mut self, work: PrimitiveWork) -> PrimitiveResult<(), E> {
        if METERED {
            (self.admit)(work).map_err(PrimitiveError::Refused)?;
        }
        Ok(())
    }
    fn checkpoint(&mut self) -> PrimitiveResult<(), E> {
        self.work(PrimitiveWork::Checkpoint)
    }
    fn string<T>(
        &mut self,
        f: impl FnOnce(&mut dyn FnMut(StringWork) -> Result<(), E>) -> Result<T, StringError<E>>,
    ) -> PrimitiveResult<T, E> {
        f(&mut |work| (self.admit)(PrimitiveWork::String(work))).map_err(Into::into)
    }
    fn number(&mut self, value: &JsValue) -> PrimitiveResult<f64, E> {
        Ok(match value.variant() {
            JsVariant::Null => 0.0,
            JsVariant::Undefined => f64::NAN,
            JsVariant::Boolean(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
            JsVariant::String(s) => {
                if METERED {
                    self.string(|admit| s.as_str().try_to_number(admit))?
                } else {
                    s.to_number()
                }
            }
            JsVariant::Float64(n) => n,
            JsVariant::Integer32(n) => f64::from(n),
            JsVariant::Symbol(_) => {
                return Err(JsNativeError::typ()
                    .with_message("argument must not be a symbol")
                    .into());
            }
            JsVariant::BigInt(_) => {
                return Err(JsNativeError::typ()
                    .with_message("argument must not be a bigint")
                    .into());
            }
            JsVariant::Object(_) => unreachable!("context-free primitive operand"),
        })
    }
    fn numeric(&mut self, value: &JsValue) -> PrimitiveResult<Numeric, E> {
        if let Some(bigint) = value.as_bigint() {
            Ok(Numeric::BigInt(bigint))
        } else {
            self.number(value).map(Numeric::Number)
        }
    }
    fn text(&mut self, value: &JsValue) -> PrimitiveResult<JsString, E> {
        Ok(match value.variant() {
            JsVariant::Null => js_string!("null"),
            JsVariant::Undefined => js_string!("undefined"),
            JsVariant::Boolean(true) => js_string!("true"),
            JsVariant::Boolean(false) => js_string!("false"),
            JsVariant::Float64(n) => {
                if METERED {
                    self.string(|admit| JsString::try_from_number(n, admit))?
                } else {
                    JsString::from(n)
                }
            }
            JsVariant::Integer32(n) => {
                if METERED {
                    self.string(|admit| JsString::try_from_integer(n, admit))?
                } else {
                    JsString::from(n)
                }
            }
            JsVariant::String(s) => s,
            JsVariant::Symbol(_) => {
                return Err(JsNativeError::typ()
                    .with_message("can't convert symbol to string")
                    .into());
            }
            JsVariant::BigInt(n) => {
                if METERED {
                    // Decimal: D=ceil(bits/3) digits, padded provider temporary
                    // extent P=2*ceil((bits+192)/3). See pinned conversion producer.
                    let bits = n.as_inner().bits();
                    let w = words::<E>(bits)?;
                    let d = checked::<E>(bits.checked_add(2))? / 3;
                    let d = d.max(1);
                    let padded = checked::<E>(bits.checked_add(194))? / 3;
                    let units = checked(
                        w.checked_mul(d)
                            .and_then(|n| d.checked_mul(2).and_then(|d| n.checked_add(d)))
                            .and_then(|n| padded.checked_mul(2).and_then(|p| n.checked_add(p))),
                    )?;
                    self.work(PrimitiveWork::BigIntConvert { units })?;
                }
                let text = n.to_string();
                self.checkpoint()?;
                if METERED {
                    self.string(|admit| JsString::try_from_utf8(&text, admit))?
                } else {
                    text.into()
                }
            }
            JsVariant::Object(_) => unreachable!("context-free primitive operand"),
        })
    }
    fn concat(&mut self, x: &JsString, y: &JsString) -> PrimitiveResult<JsValue, E> {
        if METERED {
            self.string(|admit| JsString::try_concat(x.as_str(), y.as_str(), admit))
                .map(JsValue::from)
        } else {
            Ok(js_string!(x, y).into())
        }
    }
    fn parse_bigint(&mut self, text: &JsString) -> PrimitiveResult<Option<JsBigInt>, E> {
        if !METERED {
            return Ok(JsBigInt::from_js_string(text));
        }
        let text = self.string(|admit| text.as_str().try_to_std_string(admit))?;
        let Some(text) = text else {
            return Ok(None);
        };
        // A conservative radix<=16 Horner reference schedule, including all
        // UTF8 bytes (prefix/whitespace included). sum ceil(i/8), i=1..n,
        // computed in constant time rather than traversing input to price it.
        let n = u64::try_from(text.len()).map_err(|_| PrimitiveError::WorkOverflow)?;
        let (q, r) = (n / 8, n % 8);
        let sum = checked(
            q.checked_add(1)
                .and_then(|v| q.checked_mul(v))
                .and_then(|v| v.checked_mul(4))
                .and_then(|v| r.checked_mul(q + 1).and_then(|r| v.checked_add(r))),
        )?;
        self.work(PrimitiveWork::BigIntConvert {
            units: checked(n.checked_add(sum))?,
        })?;
        let result = JsBigInt::from_string(&text);
        self.checkpoint()?;
        Ok(result)
    }
    fn compare_bigint(&mut self, a: &JsBigInt, b: Option<&JsBigInt>) -> PrimitiveResult<(), E> {
        if METERED {
            let x = words::<E>(a.as_inner().bits())?;
            let y = if let Some(b) = b {
                words::<E>(b.as_inner().bits())?
            } else {
                34
            };
            self.work(PrimitiveWork::BigIntKernel {
                units: checked(x.checked_add(y))?,
            })?;
        }
        Ok(())
    }
    fn bigint(
        &mut self,
        op: PrimitiveBinaryOp,
        a: &JsBigInt,
        b: &JsBigInt,
    ) -> PrimitiveResult<JsValue, E> {
        use PrimitiveBinaryOp as Op;
        if matches!(op, Op::Pow) {
            return self.bigint_pow(a, b);
        }
        if METERED {
            let (ab, bb) = (a.as_inner().bits(), b.as_inner().bits());
            let (x, y) = (words::<E>(ab)?, words::<E>(bb)?);
            // Existing bridge clones the left raw magnitude for these APIs.
            self.work(PrimitiveWork::BigIntCopy { words: x })?;
            let units = match op {
                Op::Mul => mul_quote::<E>(ab, bb)?,
                Op::Div | Op::Rem => {
                    let n = checked::<E>(x.checked_add(y).and_then(|n| n.checked_add(1)))?;
                    checked(
                        n.checked_mul(n)
                            .and_then(|n| n.checked_add(x))
                            .and_then(|n| n.checked_add(y)),
                    )?
                }

                Op::Shl | Op::Shr => {
                    self.work(PrimitiveWork::BigIntConvert { units: y })?;
                    let shift = b.as_inner().to_i32();
                    let result = match shift {
                        Some(k)
                            if (matches!(op, Op::Shl) && k > 0)
                                || (matches!(op, Op::Shr) && k <= 0) =>
                        {
                            checked(ab.checked_add(u64::from(k.unsigned_abs())))?
                        }
                        Some(_) => ab,
                        None => 1, // native size guard or all-shifted-out shortcut
                    };
                    checked(
                        x.checked_add(y)
                            .and_then(|n| words::<E>(result).ok().and_then(|r| n.checked_add(r))),
                    )?
                }
                _ => checked(x.checked_add(y).and_then(|n| {
                    ab.max(bb)
                        .checked_add(1)
                        .and_then(|bits| words::<E>(bits).ok())
                        .and_then(|r| n.checked_add(r))
                }))?,
            };
            self.work(PrimitiveWork::BigIntKernel { units })?;
        }
        let result = match op {
            Op::Add => Ok(JsBigInt::add(a, b)),
            Op::Sub => Ok(JsBigInt::sub(a, b)),
            Op::Mul => Ok(JsBigInt::mul(a, b)),
            Op::Div | Op::Rem => {
                if b.is_zero() {
                    return Err(JsNativeError::range()
                        .with_message("BigInt division by zero")
                        .into());
                }
                Ok(if matches!(op, Op::Div) {
                    JsBigInt::div(a, b)
                } else {
                    JsBigInt::rem(a, b)
                })
            }
            Op::BitAnd => Ok(JsBigInt::bitand(a, b)),
            Op::BitOr => Ok(JsBigInt::bitor(a, b)),
            Op::BitXor => Ok(JsBigInt::bitxor(a, b)),
            Op::Shl => JsBigInt::shift_left(a, b),
            Op::Shr => JsBigInt::shift_right(a, b),
            _ => unreachable!("BigInt arithmetic operation"),
        };
        self.checkpoint()?;
        Ok(result?.into())
    }
    fn bigint_pow(&mut self, a: &JsBigInt, b: &JsBigInt) -> PrimitiveResult<JsValue, E> {
        if METERED {
            // Preparation performs the existing exponent conversion/copy and
            // float size guard once. Semantic guard errors precede any quote
            // for a kernel that would never execute.
            let y = words::<E>(b.as_inner().bits())?;
            self.work(PrimitiveWork::BigIntCopy { words: y })?;
            self.work(PrimitiveWork::BigIntConvert {
                units: checked(y.checked_add(34))?,
            })?;
        }
        let prepared = JsBigInt::prepare_pow(a, b);
        self.checkpoint()?;
        let prepared = prepared?;
        if METERED {
            let ab = a.as_inner().bits();
            let units = if ab == 0 {
                // Zero base accepts even an exponent too large for u64 under
                // the original float guard and provider's zero shortcut.
                1
            } else {
                let mut exp = prepared
                    .exponent_u64()
                    .ok_or(PrimitiveError::WorkOverflow)?;
                let mut base = ab;
                let mut accumulator = None;
                let mut price = 0u64;
                while exp != 0 {
                    if exp & 1 == 1 {
                        accumulator = Some(if let Some(acc) = accumulator {
                            price = checked(price.checked_add(mul_quote::<E>(acc, base)?))?;
                            checked(acc.checked_add(base))?
                        } else {
                            base
                        });
                    }
                    exp >>= 1;
                    if exp != 0 {
                        price = checked(price.checked_add(mul_quote::<E>(base, base)?))?;
                        base = checked(base.checked_add(base))?;
                    }
                }
                checked(price.checked_add(words::<E>(accumulator.unwrap_or(1))?))?
            };
            self.work(PrimitiveWork::BigIntCopy {
                words: words::<E>(ab)?,
            })?;
            self.work(PrimitiveWork::BigIntKernel { units })?;
        }
        let result = prepared.evaluate();
        self.checkpoint()?;
        Ok(result.into())
    }
    fn bigint_unary(&mut self, value: &JsBigInt, not: bool) -> PrimitiveResult<JsValue, E> {
        if METERED {
            let bits = value.as_inner().bits();
            let w = words::<E>(bits)?;
            self.work(PrimitiveWork::BigIntCopy { words: w })?;
            self.work(PrimitiveWork::BigIntKernel {
                units: checked(w.checked_add(words::<E>(checked(bits.checked_add(1))?)?))?,
            })?;
        }
        let result = if not {
            JsBigInt::not(value)
        } else {
            JsBigInt::neg(value)
        };
        self.checkpoint()?;
        Ok(result.into())
    }
}

impl JsValue {
    pub(crate) fn primitive_unary<E>(
        &self,
        op: PrimitiveUnaryOp,
        admit: &mut impl FnMut(PrimitiveWork) -> Result<(), E>,
    ) -> PrimitiveResult<Self, E> {
        self.primitive_unary_inner::<_, _, true>(op, &mut PrimitiveContext { admit })
    }
    pub(crate) fn primitive_binary<E>(
        &self,
        op: PrimitiveBinaryOp,
        other: &Self,
        admit: &mut impl FnMut(PrimitiveWork) -> Result<(), E>,
    ) -> PrimitiveResult<Self, E> {
        self.primitive_binary_inner::<_, _, true>(op, other, &mut PrimitiveContext { admit })
    }
    pub(crate) fn primitive_unary_unmetered(&self, op: PrimitiveUnaryOp) -> JsResult<Self> {
        unmetered(self.primitive_unary_inner::<_, Infallible, false>(
            op,
            &mut PrimitiveContext {
                admit: &mut |_| Ok(()),
            },
        ))
    }
    pub(crate) fn primitive_binary_unmetered(
        &self,
        op: PrimitiveBinaryOp,
        other: &Self,
    ) -> JsResult<Self> {
        unmetered(self.primitive_binary_inner::<_, Infallible, false>(
            op,
            other,
            &mut PrimitiveContext {
                admit: &mut |_| Ok(()),
            },
        ))
    }
    pub(super) fn primitive_number_unmetered(&self) -> JsResult<f64> {
        unmetered(
            PrimitiveContext::<_, false> {
                admit: &mut |_| Ok::<_, Infallible>(()),
            }
            .number(self),
        )
    }
    pub(super) fn primitive_text_unmetered(&self) -> JsResult<JsString> {
        unmetered(
            PrimitiveContext::<_, false> {
                admit: &mut |_| Ok::<_, Infallible>(()),
            }
            .text(self),
        )
    }
    fn primitive_unary_inner<F, E, const M: bool>(
        &self,
        op: PrimitiveUnaryOp,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        assert!(!self.is_object(), "context-free primitive operand");
        cx.checkpoint()?;
        Ok(match op {
            PrimitiveUnaryOp::Plus => Self::new(cx.number(self)?),
            PrimitiveUnaryOp::Not => Self::new(!self.to_boolean()),
            PrimitiveUnaryOp::TypeOf => Self::new(self.js_type_of()),
            PrimitiveUnaryOp::BitNot => match cx.numeric(self)? {
                Numeric::Number(n) => Number::not(n).into(),
                Numeric::BigInt(n) => return cx.bigint_unary(&n, true),
            },
            PrimitiveUnaryOp::Minus => match self.variant() {
                // Preserve the existing public neg producer's Symbol behavior.
                JsVariant::Symbol(_) | JsVariant::Undefined => Self::nan(),
                JsVariant::String(s) => Self::new(-cx.number(&Self::from(s))?),
                JsVariant::Float64(n) => Self::new(-n),
                JsVariant::Integer32(0) | JsVariant::Boolean(false) | JsVariant::Null => {
                    Self::new(-0.0)
                }
                JsVariant::Integer32(n) => Self::new(-n),
                JsVariant::Boolean(true) => Self::new(-1),
                JsVariant::BigInt(n) => return cx.bigint_unary(&n, false),
                JsVariant::Object(_) => unreachable!("context-free primitive operand"),
            },
        })
    }
    fn primitive_binary_inner<F, E, const M: bool>(
        &self,
        op: PrimitiveBinaryOp,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        assert!(
            !self.is_object() && !other.is_object(),
            "context-free primitive operands"
        );
        cx.checkpoint()?;
        use PrimitiveBinaryOp as Op;
        match op {
            Op::Add => self.primitive_add(other, cx),
            Op::Sub => self.primitive_sub(other, cx),
            Op::Mul => self.primitive_mul(other, cx),
            Op::Div => self.primitive_div(other, cx),
            Op::Rem => self.primitive_rem(other, cx),
            Op::Pow => self.primitive_pow(other, cx),
            Op::BitAnd => self.primitive_bitand(other, cx),
            Op::BitOr => self.primitive_bitor(other, cx),
            Op::BitXor => self.primitive_bitxor(other, cx),
            Op::Shl => self.primitive_shl(other, cx),
            Op::Shr => self.primitive_shr(other, cx),
            Op::Ushr => self.primitive_ushr(other, cx),
            Op::Eq | Op::NotEq => self
                .primitive_equals(other, cx)
                .map(|r| Self::new(if matches!(op, Op::NotEq) { !r } else { r })),
            Op::StrictEq | Op::StrictNotEq => self
                .primitive_strict_equals(other, cx)
                .map(|r| Self::new(if matches!(op, Op::StrictNotEq) { !r } else { r })),
            Op::Lt | Op::Le | Op::Gt | Op::Ge => {
                let relation = if matches!(op, Op::Le | Op::Gt) {
                    other.primitive_relation(self, cx)?
                } else {
                    self.primitive_relation(other, cx)?
                };
                Ok(Self::new(if matches!(op, Op::Le | Op::Ge) {
                    matches!(relation, AbstractRelation::False)
                } else {
                    matches!(relation, AbstractRelation::True)
                }))
            }
        }
    }
}

fn unmetered<T>(value: PrimitiveResult<T, Infallible>) -> JsResult<T> {
    match value {
        Ok(value) => Ok(value),
        Err(PrimitiveError::Abrupt(error)) => Err(error),
        Err(PrimitiveError::Refused(never)) => match never {},
        Err(PrimitiveError::WorkOverflow) => {
            unreachable!("unmetered primitive has no work quote")
        }
        Err(PrimitiveError::Allocation(error)) => {
            unreachable!("unmetered primitive has no admitted allocation: {error}")
        }
    }
}

impl JsValue {
    fn primitive_add<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            // Numeric add
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => x
                .checked_add(y)
                .map_or_else(|| Self::new(f64::from(x) + f64::from(y)), Self::new),
            (JsVariant::Float64(x), JsVariant::Float64(y)) => Self::new(x + y),
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => Self::new(f64::from(x) + y),
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => Self::new(x + f64::from(y)),
            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                cx.bigint(PrimitiveBinaryOp::Add, &x, &y)?
            }

            // String concat
            (JsVariant::String(x), JsVariant::String(y)) => cx.concat(&x, &y)?,

            // Slow path:
            (_, _) => {
                let x = self.clone();
                let y = other.clone();
                match (x.variant(), y.variant()) {
                    (JsVariant::String(x), _) => {
                        let y = cx.text(&y)?;
                        cx.concat(&x, &y)?
                    }
                    (_, JsVariant::String(y)) => {
                        let x = cx.text(&x)?;
                        cx.concat(&x, &y)?
                    }
                    (_, _) => {
                        match (cx.numeric(&x)?, cx.numeric(&y)?) {
                            (Numeric::Number(x), Numeric::Number(y)) => Self::new(x + y),
                            (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                                cx.bigint(PrimitiveBinaryOp::Add, x, y)?
                            }
                            (_, _) => return Err(JsNativeError::typ()
                                .with_message(
                                    "cannot mix BigInt and other types, use explicit conversions",
                                )
                                .into()),
                        }
                    }
                }
            }
        })
    }
    fn primitive_sub<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => x
                .checked_sub(y)
                .map_or_else(|| Self::new(f64::from(x) - f64::from(y)), Self::new),
            (JsVariant::Float64(x), JsVariant::Float64(y)) => Self::new(x - y),
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => Self::new(f64::from(x) - y),
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => Self::new(x - f64::from(y)),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                cx.bigint(PrimitiveBinaryOp::Sub, &x, &y)?
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(a), Numeric::Number(b)) => Self::new(a - b),
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    cx.bigint(PrimitiveBinaryOp::Sub, x, y)?
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
    fn primitive_mul<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => x
                .checked_mul(y)
                // Check for the special case of `0 * -N` which must produce `-0.0`
                .filter(|v| *v != 0 || i32::min(x, y) >= 0)
                .map_or_else(|| Self::new(f64::from(x) * f64::from(y)), Self::new),
            (JsVariant::Float64(x), JsVariant::Float64(y)) => Self::new(x * y),
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => Self::new(f64::from(x) * y),
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => Self::new(x * f64::from(y)),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                cx.bigint(PrimitiveBinaryOp::Mul, &x, &y)?
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(a), Numeric::Number(b)) => Self::new(a * b),
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    cx.bigint(PrimitiveBinaryOp::Mul, x, y)?
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
    fn primitive_div<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => x
                .checked_div(y)
                .filter(|div| y * div == x)
                .map_or_else(|| Self::new(f64::from(x) / f64::from(y)), Self::new),
            (JsVariant::Float64(x), JsVariant::Float64(y)) => Self::new(x / y),
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => Self::new(f64::from(x) / y),
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => Self::new(x / f64::from(y)),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                if y.is_zero() {
                    return Err(JsNativeError::range()
                        .with_message("BigInt division by zero")
                        .into());
                }
                cx.bigint(PrimitiveBinaryOp::Div, &x, &y)?
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(a), Numeric::Number(b)) => Self::new(a / b),
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    if y.is_zero() {
                        return Err(JsNativeError::range()
                            .with_message("BigInt division by zero")
                            .into());
                    }
                    cx.bigint(PrimitiveBinaryOp::Div, x, y)?
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
    fn primitive_rem<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => {
                if y == 0 {
                    Self::nan()
                } else {
                    match x % y {
                        rem if rem == 0 && x < 0 => Self::new(-0.0),
                        rem => Self::new(rem),
                    }
                }
            }
            (JsVariant::Float64(x), JsVariant::Float64(y)) => Self::new((x % y).copysign(x)),
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => {
                let x = f64::from(x);
                Self::new((x % y).copysign(x))
            }

            (JsVariant::Float64(x), JsVariant::Integer32(y)) => {
                Self::new((x % f64::from(y)).copysign(x))
            }

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                if y.is_zero() {
                    return Err(JsNativeError::range()
                        .with_message("BigInt division by zero")
                        .into());
                }
                cx.bigint(PrimitiveBinaryOp::Rem, &x, &y)?
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(a), Numeric::Number(b)) => Self::new(a % b),
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    if y.is_zero() {
                        return Err(JsNativeError::range()
                            .with_message("BigInt division by zero")
                            .into());
                    }
                    cx.bigint(PrimitiveBinaryOp::Rem, x, y)?
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
    fn primitive_pow<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => u32::try_from(y)
                .ok()
                .and_then(|y| x.checked_pow(y))
                .map_or_else(|| Self::new(f64::from(x).powi(y)), Self::new),
            (JsVariant::Float64(x), JsVariant::Float64(y)) => {
                if Number::equal(x.abs(), 1.0) && y.is_infinite() {
                    Self::nan()
                } else {
                    Self::new(x.powf(y))
                }
            }
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => {
                if x.wrapping_abs() == 1 && y.is_infinite() {
                    Self::nan()
                } else {
                    Self::new(f64::from(x).powf(y))
                }
            }
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => Self::new(x.powi(y)),
            (JsVariant::BigInt(a), JsVariant::BigInt(b)) => {
                cx.bigint(PrimitiveBinaryOp::Pow, &a, &b)?
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(a), Numeric::Number(b)) => {
                    if Number::equal(a.abs(), 1.0) && b.is_infinite() {
                        Self::nan()
                    } else {
                        Self::new(a.powf(b))
                    }
                }
                (Numeric::BigInt(ref a), Numeric::BigInt(ref b)) => {
                    cx.bigint(PrimitiveBinaryOp::Pow, a, b)?
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
    fn primitive_bitand<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => Self::new(x & y),
            (JsVariant::Float64(x), JsVariant::Float64(y)) => {
                Self::new(f64_to_int32(x) & f64_to_int32(y))
            }
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => Self::new(x & f64_to_int32(y)),
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => Self::new(f64_to_int32(x) & y),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                cx.bigint(PrimitiveBinaryOp::BitAnd, &x, &y)?
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(a), Numeric::Number(b)) => {
                    Self::new(f64_to_int32(a) & f64_to_int32(b))
                }
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    cx.bigint(PrimitiveBinaryOp::BitAnd, x, y)?
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
    fn primitive_bitor<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => Self::new(x | y),
            (JsVariant::Float64(x), JsVariant::Float64(y)) => {
                Self::new(f64_to_int32(x) | f64_to_int32(y))
            }
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => Self::new(x | f64_to_int32(y)),
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => Self::new(f64_to_int32(x) | y),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                cx.bigint(PrimitiveBinaryOp::BitOr, &x, &y)?
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(a), Numeric::Number(b)) => {
                    Self::new(f64_to_int32(a) | f64_to_int32(b))
                }
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    cx.bigint(PrimitiveBinaryOp::BitOr, x, y)?
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
    fn primitive_bitxor<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => Self::new(x ^ y),
            (JsVariant::Float64(x), JsVariant::Float64(y)) => {
                Self::new(f64_to_int32(x) ^ f64_to_int32(y))
            }
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => Self::new(x ^ f64_to_int32(y)),
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => Self::new(f64_to_int32(x) ^ y),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                cx.bigint(PrimitiveBinaryOp::BitXor, &x, &y)?
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(a), Numeric::Number(b)) => {
                    Self::new(f64_to_int32(a) ^ f64_to_int32(b))
                }
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    cx.bigint(PrimitiveBinaryOp::BitXor, x, y)?
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
    fn primitive_shl<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => {
                Self::new(x.wrapping_shl(y as u32))
            }
            (JsVariant::Float64(x), JsVariant::Float64(y)) => {
                Self::new(f64_to_int32(x).wrapping_shl(f64_to_uint32(y)))
            }
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => {
                Self::new(x.wrapping_shl(f64_to_uint32(y)))
            }
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => {
                Self::new(f64_to_int32(x).wrapping_shl(y as u32))
            }

            (JsVariant::BigInt(a), JsVariant::BigInt(b)) => {
                cx.bigint(PrimitiveBinaryOp::Shl, &a, &b)?
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(x), Numeric::Number(y)) => {
                    Self::new(f64_to_int32(x).wrapping_shl(f64_to_uint32(y)))
                }
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    cx.bigint(PrimitiveBinaryOp::Shl, x, y)?
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
    fn primitive_shr<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => {
                Self::new(x.wrapping_shr(y as u32))
            }
            (JsVariant::Float64(x), JsVariant::Float64(y)) => {
                Self::new(f64_to_int32(x).wrapping_shr(f64_to_uint32(y)))
            }
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => {
                Self::new(x.wrapping_shr(f64_to_uint32(y)))
            }
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => {
                Self::new(f64_to_int32(x).wrapping_shr(y as u32))
            }

            (JsVariant::BigInt(a), JsVariant::BigInt(b)) => {
                cx.bigint(PrimitiveBinaryOp::Shr, &a, &b)?
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(x), Numeric::Number(y)) => {
                    Self::new(f64_to_int32(x).wrapping_shr(f64_to_uint32(y)))
                }
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    cx.bigint(PrimitiveBinaryOp::Shr, x, y)?
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
    fn primitive_ushr<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<Self, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => {
                Self::new((x as u32).wrapping_shr(y as u32))
            }
            (JsVariant::Float64(x), JsVariant::Float64(y)) => {
                Self::new(f64_to_uint32(x).wrapping_shr(f64_to_uint32(y)))
            }
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => {
                Self::new((x as u32).wrapping_shr(f64_to_uint32(y)))
            }
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => {
                Self::new(f64_to_uint32(x).wrapping_shr(y as u32))
            }

            // Slow path:
            (_, _) => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(x), Numeric::Number(y)) => {
                    Self::new(f64_to_uint32(x).wrapping_shr(f64_to_uint32(y)))
                }
                (Numeric::BigInt(_), Numeric::BigInt(_)) => {
                    return Err(JsNativeError::typ()
                        .with_message("BigInts have no unsigned right shift, use >> instead")
                        .into());
                }
                (_, _) => {
                    return Err(JsNativeError::typ()
                        .with_message("cannot mix BigInt and other types, use explicit conversions")
                        .into());
                }
            },
        })
    }
}

impl JsValue {
    pub(super) fn primitive_relation_unmetered(&self, other: &Self) -> JsResult<AbstractRelation> {
        unmetered(self.primitive_relation::<_, Infallible, false>(
            other,
            &mut PrimitiveContext {
                admit: &mut |_| Ok(()),
            },
        ))
    }
    pub(super) fn primitive_strict_equals_unmetered(&self, other: &Self) -> bool {
        unmetered(self.primitive_strict_equals::<_, Infallible, false>(
            other,
            &mut PrimitiveContext {
                admit: &mut |_| Ok(()),
            },
        ))
        .expect("strict equality is infallible")
    }
    fn primitive_strict_equals<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<bool, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        if self.get_type() != other.get_type() {
            return Ok(false);
        }
        let result = match (self.variant(), other.variant()) {
            (JsVariant::String(a), JsVariant::String(b)) => {
                if M {
                    return cx.string(|admit| a.as_str().try_eq(b.as_str(), admit));
                }
                a == b
            }
            (JsVariant::BigInt(a), JsVariant::BigInt(b)) => {
                cx.compare_bigint(&a, Some(&b))?;
                JsBigInt::equal(&a, &b)
            }
            _ => self.strict_equals_scalar(other),
        };
        cx.checkpoint()?;
        Ok(result)
    }
    fn primitive_equals<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<bool, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        if self.get_type() == other.get_type() {
            return self.primitive_strict_equals(other, cx);
        }
        let result = match (self.variant(), other.variant()) {
            (JsVariant::Null, JsVariant::Undefined) | (JsVariant::Undefined, JsVariant::Null) => {
                true
            }
            (
                JsVariant::Integer32(_) | JsVariant::Float64(_),
                JsVariant::String(_) | JsVariant::Boolean(_),
            )
            | (JsVariant::String(_), JsVariant::Integer32(_) | JsVariant::Float64(_)) => {
                Number::equal(cx.number(self)?, cx.number(other)?)
            }
            (JsVariant::BigInt(a), JsVariant::String(b))
            | (JsVariant::String(b), JsVariant::BigInt(a)) => {
                let Some(b) = cx.parse_bigint(&b)? else {
                    return Ok(false);
                };
                cx.compare_bigint(&a, Some(&b))?;
                JsBigInt::equal(&a, &b)
            }
            (JsVariant::Boolean(b), _) => {
                return other.primitive_equals(&Self::new(i32::from(b)), cx);
            }
            (_, JsVariant::Boolean(b)) => {
                return self.primitive_equals(&Self::new(i32::from(b)), cx);
            }
            (JsVariant::BigInt(a), JsVariant::Float64(b))
            | (JsVariant::Float64(b), JsVariant::BigInt(a)) => {
                cx.compare_bigint(&a, None)?;
                if M {
                    cx.work(PrimitiveWork::BigIntConvert { units: 34 })?;
                }
                a == b
            }
            (JsVariant::BigInt(a), JsVariant::Integer32(b))
            | (JsVariant::Integer32(b), JsVariant::BigInt(a)) => {
                cx.compare_bigint(&a, None)?;
                if M {
                    cx.work(PrimitiveWork::BigIntConvert { units: 1 })?;
                }
                a == b
            }
            _ => false,
        };
        cx.checkpoint()?;
        Ok(result)
    }
    fn primitive_relation<F, E, const M: bool>(
        &self,
        other: &Self,
        cx: &mut PrimitiveContext<'_, F, M>,
    ) -> PrimitiveResult<AbstractRelation, E>
    where
        F: FnMut(PrimitiveWork) -> Result<(), E>,
    {
        let result = match (self.variant(), other.variant()) {
            (JsVariant::Integer32(x), JsVariant::Integer32(y)) => (x < y).into(),
            (JsVariant::Integer32(x), JsVariant::Float64(y)) => Number::less_than(f64::from(x), y),
            (JsVariant::Float64(x), JsVariant::Integer32(y)) => Number::less_than(x, f64::from(y)),
            (JsVariant::Float64(x), JsVariant::Float64(y)) => Number::less_than(x, y),
            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                cx.compare_bigint(&x, Some(&y))?;
                (x < y).into()
            }
            (JsVariant::String(x), JsVariant::String(y)) => {
                if M {
                    cx.string(|admit| x.as_str().try_cmp(y.as_str(), admit))?
                        .is_lt()
                        .into()
                } else {
                    (x < y).into()
                }
            }
            (JsVariant::BigInt(x), JsVariant::String(y)) => {
                let Some(y) = cx.parse_bigint(&y)? else {
                    return Ok(AbstractRelation::Undefined);
                };
                cx.compare_bigint(&x, Some(&y))?;
                (x < y).into()
            }
            (JsVariant::String(x), JsVariant::BigInt(y)) => {
                let Some(x) = cx.parse_bigint(&x)? else {
                    return Ok(AbstractRelation::Undefined);
                };
                cx.compare_bigint(&x, Some(&y))?;
                (x < y).into()
            }
            _ => match (cx.numeric(self)?, cx.numeric(other)?) {
                (Numeric::Number(x), Numeric::Number(y)) => Number::less_than(x, y),
                (Numeric::BigInt(x), Numeric::BigInt(y)) => {
                    cx.compare_bigint(&x, Some(&y))?;
                    (x < y).into()
                }
                (Numeric::BigInt(x), Numeric::Number(y)) => {
                    if y.is_nan() {
                        return Ok(AbstractRelation::Undefined);
                    }
                    if y.is_infinite() {
                        return Ok(y.is_sign_positive().into());
                    }
                    cx.compare_bigint(&x, None)?;
                    if M {
                        cx.work(PrimitiveWork::BigIntConvert {
                            units: checked(words::<E>(x.as_inner().bits())?.checked_add(34))?,
                        })?;
                    }
                    if let Ok(y) = JsBigInt::try_from(y) {
                        (x < y).into()
                    } else {
                        (x.to_f64() < y).into()
                    }
                }
                (Numeric::Number(x), Numeric::BigInt(y)) => {
                    if x.is_nan() {
                        return Ok(AbstractRelation::Undefined);
                    }
                    if x.is_infinite() {
                        return Ok(x.is_sign_negative().into());
                    }
                    cx.compare_bigint(&y, None)?;
                    if M {
                        cx.work(PrimitiveWork::BigIntConvert {
                            units: checked(words::<E>(y.as_inner().bits())?.checked_add(34))?,
                        })?;
                    }
                    if let Ok(x) = JsBigInt::try_from(x) {
                        (x < y).into()
                    } else {
                        (x < y.to_f64()).into()
                    }
                }
            },
        };
        cx.checkpoint()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binary(op: PrimitiveBinaryOp, a: JsValue, b: JsValue) -> JsValue {
        a.primitive_binary(op, &b, &mut |_| Ok::<_, ()>(()))
            .unwrap()
    }

    #[test]
    fn primitive_arithmetic_and_bitwise_have_language_results() {
        use PrimitiveBinaryOp as Op;
        for (op, a, b, expected) in [
            (Op::Add, 7, 3, 10),
            (Op::Sub, 7, 3, 4),
            (Op::Mul, 7, 3, 21),
            (Op::Div, 8, 2, 4),
            (Op::Rem, 7, 3, 1),
            (Op::Pow, 2, 5, 32),
            (Op::BitAnd, 6, 3, 2),
            (Op::BitOr, 6, 3, 7),
            (Op::BitXor, 6, 3, 5),
            (Op::Shl, 3, 2, 12),
            (Op::Shr, -12, 2, -3),
            (Op::Ushr, 12, 2, 3),
        ] {
            assert!(binary(op, a.into(), b.into()).strict_equals(&expected.into()));
            let big = JsValue::from(JsBigInt::from(a));
            let rhs = JsValue::from(JsBigInt::from(b));
            if !matches!(op, Op::Ushr) {
                assert!(binary(op, big, rhs).strict_equals(&JsBigInt::from(expected).into()));
            }
        }
        assert!(
            binary(Op::Mul, 0.into(), (-3).into())
                .as_number()
                .unwrap()
                .is_sign_negative()
        );
        assert_eq!(
            binary(Op::Add, js_string!("x").into(), 2.into())
                .as_string()
                .unwrap(),
            js_string!("x2")
        );
    }

    #[test]
    fn primitive_equality_and_relations_cover_conversions_and_nan() {
        use PrimitiveBinaryOp as Op;
        for (op, a, b, expected) in [
            (Op::Eq, JsValue::null(), JsValue::undefined(), true),
            (Op::StrictEq, JsValue::null(), JsValue::undefined(), false),
            (
                Op::NotEq,
                JsValue::new(3),
                JsValue::new(js_string!("3")),
                false,
            ),
            (
                Op::StrictNotEq,
                JsValue::new(3),
                JsValue::new(js_string!("3")),
                true,
            ),
            (
                Op::Eq,
                JsValue::new(JsBigInt::from(32)),
                JsValue::new(js_string!("0x20")),
                true,
            ),
            (
                Op::Eq,
                JsValue::new(JsBigInt::from(32)),
                JsValue::new(js_string!("bad")),
                false,
            ),
            (
                Op::Lt,
                JsValue::new(js_string!("10")),
                JsValue::new(js_string!("2")),
                true,
            ),
            (Op::Gt, JsValue::new(3), JsValue::new(2), true),
            (Op::Le, JsValue::new(3), JsValue::new(3), true),
            (Op::Ge, JsValue::new(3), JsValue::new(3), true),
            (Op::Le, JsValue::nan(), JsValue::new(3), false),
            (Op::Ge, JsValue::new(3), JsValue::nan(), false),
            (
                Op::Lt,
                JsValue::new(JsBigInt::from(3)),
                JsValue::new(3.5),
                true,
            ),
            (
                Op::Gt,
                JsValue::new(3.5),
                JsValue::new(JsBigInt::from(3)),
                true,
            ),
        ] {
            assert_eq!(binary(op, a, b).as_boolean(), Some(expected));
        }
    }

    #[test]
    fn primitive_unary_preserves_signed_zero_and_types() {
        let mut admit = |_| Ok::<_, ()>(());
        assert!(
            JsValue::new(0)
                .primitive_unary(PrimitiveUnaryOp::Minus, &mut admit)
                .unwrap()
                .as_number()
                .unwrap()
                .is_sign_negative()
        );
        assert_eq!(
            JsValue::new(js_string!(" 42 "))
                .primitive_unary(PrimitiveUnaryOp::Plus, &mut admit)
                .unwrap()
                .as_number(),
            Some(42.0)
        );
        assert_eq!(
            JsValue::null()
                .primitive_unary(PrimitiveUnaryOp::Not, &mut admit)
                .unwrap()
                .as_boolean(),
            Some(true)
        );
        assert!(
            JsValue::new(JsBigInt::from(2))
                .primitive_unary(PrimitiveUnaryOp::BitNot, &mut admit)
                .unwrap()
                .strict_equals(&JsBigInt::from(-3).into())
        );
        assert_eq!(
            JsValue::new(JsBigInt::from(2))
                .primitive_unary(PrimitiveUnaryOp::TypeOf, &mut admit)
                .unwrap()
                .as_string()
                .unwrap(),
            js_string!("bigint")
        );
    }

    #[test]
    fn bigint_multiplication_reserves_input_work_before_opaque_kernel() {
        let a = JsValue::from(JsBigInt::new(
            crate::bigint::RawBigInt::from(1u8) << 63usize,
        ));
        let b = JsValue::from(JsBigInt::new(
            crate::bigint::RawBigInt::from(1u8) << 95usize,
        ));
        let mut copied = 0;
        let result = a.primitive_binary(PrimitiveBinaryOp::Mul, &b, &mut |work| {
            match work {
                PrimitiveWork::BigIntCopy { words } => copied += words,
                // 2*3 + 2 + 3 + 5 canonical reference cells/extents.
                PrimitiveWork::BigIntKernel { units } => {
                    assert_eq!(units, 16);
                    return Err("refused product");
                }
                _ => {}
            }
            Ok(())
        });
        assert!(matches!(
            result,
            Err(PrimitiveError::Refused("refused product"))
        ));
        assert_eq!(copied, 2);
    }

    #[test]
    fn opaque_completion_checks_cancellation_before_returning_result() {
        let mut entered = false;
        let result = JsValue::from(JsBigInt::from(2)).primitive_binary(
            PrimitiveBinaryOp::Mul,
            &JsBigInt::from(3).into(),
            &mut |work| {
                match work {
                    PrimitiveWork::BigIntKernel { .. } => entered = true,
                    PrimitiveWork::Checkpoint if entered => return Err("cancelled"),
                    _ => {}
                }
                Ok(())
            },
        );
        assert!(matches!(result, Err(PrimitiveError::Refused("cancelled"))));
    }

    #[test]
    fn language_abrupt_and_host_refusal_remain_distinct() {
        let a = JsValue::from(JsBigInt::from(1));
        let zero = JsValue::from(JsBigInt::zero());
        assert!(matches!(
            a.primitive_binary(PrimitiveBinaryOp::Div, &zero, &mut |_| Ok::<_, ()>(())),
            Err(PrimitiveError::Abrupt(_))
        ));
        assert!(matches!(
            a.primitive_binary(
                PrimitiveBinaryOp::Add,
                &JsValue::new(1),
                &mut |_| Ok::<_, ()>(())
            ),
            Err(PrimitiveError::Abrupt(_))
        ));
        assert!(matches!(
            a.primitive_binary(PrimitiveBinaryOp::Add, &a, &mut |_| Err("stopped")),
            Err(PrimitiveError::Refused("stopped"))
        ));
        assert!(matches!(
            mul_quote::<()>(u64::MAX, 2),
            Err(PrimitiveError::WorkOverflow)
        ));
    }
    #[test]
    fn contextual_wrappers_preserve_left_to_right_conversion() {
        for operator in [
            "+", "-", "*", "/", "%", "**", "&", "|", "^", "<<", ">>", ">>>", "<", "<=", ">", ">=",
        ] {
            let source = format!(
                "(() => {{ let trace=''; const left={{valueOf(){{trace+='l';return 3;}}}}; const right={{valueOf(){{trace+='r';return 2;}}}}; const result=left {operator} right; return trace; }})()"
            );
            crate::run_test_actions([crate::TestAction::assert_eq(source, js_string!("lr"))]);
        }
    }
    #[test]
    fn power_native_guard_precedes_kernel_quote_for_huge_exponent() {
        let exponent = JsValue::from(JsBigInt::new(
            crate::bigint::RawBigInt::from(1u8) << 64usize,
        ));
        let mut copies = Vec::new();
        let result = JsValue::from(JsBigInt::one()).primitive_binary(
            PrimitiveBinaryOp::Pow,
            &exponent,
            &mut |work| {
                match work {
                    PrimitiveWork::BigIntCopy { words } => copies.push(words),
                    PrimitiveWork::BigIntKernel { .. } => return Err("kernel must not be entered"),
                    _ => {}
                }
                Ok(())
            },
        );
        let Err(PrimitiveError::Abrupt(error)) = result else {
            panic!("native size guard must remain a semantic error");
        };
        assert!(error.as_native().unwrap().is_range());
        assert_eq!(
            error.as_native().unwrap().message(),
            "Maximum BigInt size exceeded"
        );
        assert_eq!(copies, [3]);
        for (base, exponent, expected) in [
            (JsBigInt::zero(), exponent, JsBigInt::zero()),
            (JsBigInt::zero(), JsBigInt::zero().into(), JsBigInt::one()),
            (JsBigInt::from(7), JsBigInt::zero().into(), JsBigInt::one()),
        ] {
            assert!(
                binary(PrimitiveBinaryOp::Pow, base.into(), exponent)
                    .strict_equals(&expected.into())
            );
        }
    }
}

#[cfg(test)]
mod magnitude_tests {
    use super::*;
    use num_bigint::{BigInt as Raw, Sign};
    use std::{cell::Cell, rc::Rc};

    #[derive(Debug, PartialEq, Eq)]
    enum HashWrite {
        Sign(u8),
        Count(u64),
        Word(u32),
    }

    #[derive(Default)]
    struct RecordingHasher {
        writes: Vec<HashWrite>,
        refused: Rc<Cell<bool>>,
    }
    impl Hasher for RecordingHasher {
        fn finish(&self) -> u64 {
            0
        }
        fn write(&mut self, _: &[u8]) {
            panic!("canonical magnitude hash uses typed writes");
        }
        fn write_u8(&mut self, value: u8) {
            assert!(!self.refused.get(), "write after refusal");
            self.writes.push(HashWrite::Sign(value));
        }
        fn write_u64(&mut self, value: u64) {
            assert!(!self.refused.get(), "write after refusal");
            self.writes.push(HashWrite::Count(value));
        }
        fn write_u32(&mut self, value: u32) {
            assert!(!self.refused.get(), "write after refusal");
            self.writes.push(HashWrite::Word(value));
        }
    }

    fn large(words: usize) -> Raw {
        Raw::new(Sign::Plus, vec![u32::MAX; words])
    }

    #[test]
    fn canonical_hash_and_equality_preserve_sign_zero_and_constructed_values() {
        let positive = Raw::new(Sign::Plus, vec![7, 0, 1]);
        let equivalent = (Raw::from(1u8) << 64usize) + Raw::from(7u8);
        for (a, b, sign, digits) in [
            (positive.clone(), equivalent.clone(), 1, vec![7, 0, 1]),
            (-positive, -equivalent, 2, vec![7, 0, 1]),
            (
                Raw::from(0),
                Raw::new(Sign::Minus, vec![0, 0]),
                0,
                Vec::new(),
            ),
        ] {
            let (a, b) = (JsBigInt::new(a), JsBigInt::new(b));
            assert!(try_bigint_eq(&a, &b, &mut |_| Ok::<_, ()>(())).unwrap());
            let expected: Vec<_> = [HashWrite::Sign(sign), HashWrite::Count(digits.len() as u64)]
                .into_iter()
                .chain(digits.into_iter().map(HashWrite::Word))
                .collect();
            for value in [&a, &b] {
                let mut state = RecordingHasher::default();
                try_bigint_hash(value, &mut state, &mut |_| Ok::<_, ()>(())).unwrap();
                assert_eq!(state.writes, expected);
            }
        }
    }

    #[test]
    fn equality_stops_at_sign_length_and_first_different_word() {
        let positive = Raw::new(Sign::Plus, vec![7, 0, 1]);
        for (left, right, expected, reads) in [
            (Raw::from(0), Raw::from(0), true, 0),
            (positive.clone(), -positive.clone(), false, 0),
            (positive.clone(), Raw::from(7), false, 2),
            (
                positive.clone(),
                Raw::new(Sign::Plus, vec![8, 0, 1]),
                false,
                4,
            ),
            (
                positive.clone(),
                Raw::new(Sign::Plus, vec![7, 0, 2]),
                false,
                8,
            ),
            (positive.clone(), positive, true, 8),
        ] {
            let mut actual = 0;
            let result = try_bigint_eq(&JsBigInt::new(left), &JsBigInt::new(right), &mut |work| {
                if let PrimitiveWork::BigIntRead { words } = work {
                    actual += words;
                }
                Ok::<_, ()>(())
            })
            .unwrap();
            assert_eq!(result, expected);
            assert_eq!(actual, reads);
        }
    }

    #[test]
    fn hash_exact_and_one_short_read_limits_preserve_accepted_prefix() {
        let value = JsBigInt::new(large(513));
        for limit in [514, 513] {
            let mut accepted = 0;
            let mut state = RecordingHasher::default();
            let refused = state.refused.clone();
            let marker = Rc::new(());
            let result = try_bigint_hash(&value, &mut state, &mut |work| {
                if let PrimitiveWork::BigIntRead { words } = work {
                    assert!(words <= 256);
                    if words > limit - accepted {
                        refused.set(true);
                        return Err(marker.clone());
                    }
                    accepted += words;
                }
                Ok(())
            });
            assert_eq!(accepted, limit);
            let prefix = if limit == 514 { 513 } else { 512 };
            assert_eq!(state.writes.len(), prefix + 2);
            assert_eq!(state.writes[0], HashWrite::Sign(1));
            assert_eq!(state.writes[1], HashWrite::Count(513));
            assert!(
                state.writes[2..]
                    .iter()
                    .all(|write| *write == HashWrite::Word(u32::MAX))
            );
            if limit == 514 {
                assert!(result.is_ok());
            } else {
                let Err(PrimitiveError::Refused(error)) = result else {
                    panic!("read limit must refuse");
                };
                assert!(Rc::ptr_eq(&error, &marker));
            }
        }
    }

    #[test]
    fn raw_copy_preserves_values_and_admits_copy_and_payload_before_provider() {
        for raw in [Raw::from(0), Raw::from(-1), large(513), -large(513)] {
            let result = try_copy_raw_bigint(&raw, &mut |_| Ok::<_, ()>(())).unwrap();
            assert_eq!(result.as_inner(), &raw);
        }
        let raw = large(513);
        let descriptor = std::mem::size_of::<Raw>() as u64;
        let mut copies = 0;
        for capacity in [descriptor + 2056, descriptor + 2055] {
            let marker = Rc::new(());
            let result = try_copy_raw_bigint(&raw, &mut |work| {
                match work {
                    PrimitiveWork::BigIntCapacity { bytes } => {
                        assert_eq!(bytes, descriptor + 2056);
                        if bytes > capacity {
                            return Err(marker.clone());
                        }
                    }
                    PrimitiveWork::BigIntCopy { words } => {
                        assert_eq!(words, 513);
                        copies += 1;
                    }
                    _ => {}
                }
                Ok(())
            });
            if capacity == descriptor + 2056 {
                assert_eq!(result.unwrap().as_inner(), &raw);
            } else {
                let Err(PrimitiveError::Refused(error)) = result else {
                    panic!("payload limit must refuse");
                };
                assert!(Rc::ptr_eq(&error, &marker));
            }
        }
        assert_eq!(copies, 1);
        for limit in [513, 512] {
            let result = try_copy_raw_bigint(&raw, &mut |work| {
                if let PrimitiveWork::BigIntCopy { words } = work
                    && words > limit
                {
                    return Err("copy limit");
                }
                Ok(())
            });
            if limit == 513 {
                assert_eq!(result.unwrap().as_inner(), &raw);
            } else {
                assert!(matches!(result, Err(PrimitiveError::Refused("copy limit"))));
            }
        }
    }

    #[test]
    fn magnitude_producers_preserve_refusal_identity_at_every_boundary() {
        let raw = large(513);
        let value = JsBigInt::new(raw.clone());
        for denied in 0..6 {
            let mut state = RecordingHasher::default();
            let refused = state.refused.clone();
            let marker = Rc::new(());
            let mut calls = 0;
            let result = try_bigint_hash(&value, &mut state, &mut |_| {
                let current = calls;
                calls += 1;
                if current == denied {
                    refused.set(true);
                    Err(marker.clone())
                } else {
                    Ok(())
                }
            });
            let Err(PrimitiveError::Refused(error)) = result else {
                panic!("hash boundary must refuse");
            };
            assert!(Rc::ptr_eq(&error, &marker));
            assert_eq!(calls, denied + 1);
        }
        for denied in 0..5 {
            let marker = Rc::new(());
            let mut calls = 0;
            let result = try_copy_raw_bigint(&raw, &mut |_| {
                let current = calls;
                calls += 1;
                if current == denied {
                    Err(marker.clone())
                } else {
                    Ok(())
                }
            });
            let Err(PrimitiveError::Refused(error)) = result else {
                panic!("copy boundary must refuse");
            };
            assert!(Rc::ptr_eq(&error, &marker));
            assert_eq!(calls, denied + 1);
        }
        let small = JsBigInt::new(Raw::new(Sign::Plus, vec![7, 0, 1]));
        for denied in 0..7 {
            let marker = Rc::new(());
            let mut calls = 0;
            let result = try_bigint_eq(&small, &small, &mut |_| {
                let current = calls;
                calls += 1;
                if current == denied {
                    Err(marker.clone())
                } else {
                    Ok(())
                }
            });
            let Err(PrimitiveError::Refused(error)) = result else {
                panic!("equality boundary must refuse");
            };
            assert!(Rc::ptr_eq(&error, &marker));
            assert_eq!(calls, denied + 1);
        }
    }

    #[test]
    fn equality_exact_and_one_short_limits_admit_only_complete_word_pairs() {
        let value = JsBigInt::new(large(3));
        for limit in [8, 7] {
            let mut accepted = 0;
            let result = try_bigint_eq(&value, &value, &mut |work| {
                if let PrimitiveWork::BigIntRead { words } = work {
                    if words > limit - accepted {
                        return Err("pair limit");
                    }
                    accepted += words;
                }
                Ok(())
            });
            if limit == 8 {
                assert!(result.unwrap());
                assert_eq!(accepted, 8);
            } else {
                assert!(matches!(result, Err(PrimitiveError::Refused("pair limit"))));
                assert_eq!(accepted, 6);
            }
        }
    }
}
